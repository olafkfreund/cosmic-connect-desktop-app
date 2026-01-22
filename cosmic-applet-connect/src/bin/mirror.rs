use cosmic::app::{Core, Task};
use cosmic::iced::widget::image;
use cosmic::iced::Length;
use cosmic::widget::{button, column, container, text};
use cosmic::Element;
use std::env;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use cosmic_applet_connect::dbus_client::DbusClient;
use cosmic_connect_protocol::plugins::screenshare::decoder::VideoDecoder;
use cosmic_connect_protocol::plugins::screenshare::stream_receiver::StreamReceiver;

struct MirrorApp {
    core: Core,
    device_id: String,
    status: String,
    frame: Option<image::Handle>,
    receiver_rx: Arc<Mutex<mpsc::Receiver<Message>>>,
    dbus: Option<DbusClient>,
}

#[derive(Debug, Clone)]
enum Message {
    Close,
    StatusUpdate(String),
    FrameReceived(image::Handle),
    Error(String),
    Connected,
    Loop(Box<Message>),
    DbusConnected(DbusClient),
}

impl cosmic::Application for MirrorApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicConnect.Mirror";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let args: Vec<String> = env::args().collect();
        let device_id = args.get(1).cloned().unwrap_or_else(|| "unknown".to_string());

        let (tx, rx) = mpsc::channel(10);
        let receiver_rx = Arc::new(Mutex::new(rx));
        let dev_id = device_id.clone();

        // Spawn logic task
        tokio::spawn(async move {
            let _ = tx.send(Message::StatusUpdate("Connecting to daemon...".into())).await;
            
            let (client, _) = match DbusClient::connect().await {
                Ok((c, r)) => {
                    let _ = tx.send(Message::DbusConnected(c.clone())).await;
                    (c, r)
                }
                Err(e) => {
                    let _ = tx.send(Message::Error(format!("DBus connect failed: {}", e))).await;
                    return;
                }
            };
            
            let _ = tx.send(Message::StatusUpdate("Starting listener...".into())).await;
            
            let mut receiver = StreamReceiver::new();
            let port = match receiver.listen().await {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx.send(Message::Error(format!("Listen failed: {}", e))).await;
                    return;
                }
            };
            
            let _ = tx.send(Message::StatusUpdate(format!("Listening on port {}. Requesting stream...", port))).await;
            
            if let Err(e) = client.start_screen_share(&dev_id, port).await {
                let _ = tx.send(Message::Error(format!("StartScreenShare failed: {}", e))).await;
                return;
            }
            
            let _ = tx.send(Message::StatusUpdate("Waiting for connection...".into())).await;
            
            if let Err(e) = receiver.accept().await {
                let _ = tx.send(Message::Error(format!("Accept failed: {}", e))).await;
                return;
            }
            
            let _ = tx.send(Message::Connected).await;
            
            let decoder = match VideoDecoder::new() {
                Ok(d) => d,
                Err(e) => {
                    let _ = tx.send(Message::Error(format!("Decoder init failed: {}", e))).await;
                    return;
                }
            };
            
            if let Err(e) = decoder.start() {
                let _ = tx.send(Message::Error(format!("Decoder start failed: {}", e))).await;
                return;
            }
            
            loop {
                match receiver.next_frame().await {
                    Ok((_type, _ts, payload)) => {
                        if let Err(e) = decoder.push_frame(&payload) {
                            let _ = tx.send(Message::Error(format!("Decode push error: {}", e))).await;
                            break;
                        }
                        
                        match decoder.pull_frame() {
                            Ok(Some((data, width, height))) => {
                                let handle = image::Handle::from_rgba(width, height, data);
                                if let Err(_) = tx.send(Message::FrameReceived(handle)).await {
                                    break;
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let _ = tx.send(Message::Error(format!("Decode pull error: {}", e))).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Stream error: {}", e))).await;
                        break;
                    }
                }
            }
        });

        let app = Self {
            core,
            device_id,
            status: "Initializing...".to_string(),
            frame: None,
            receiver_rx: receiver_rx.clone(),
            dbus: None,
        };
        
        let task = Task::perform(
            wait_for_message(receiver_rx),
            |msg| cosmic::Action::App(Message::Loop(Box::new(msg)))
        );
        
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loop(inner) => {
                let task = self.update(*inner);
                let next = Task::perform(
                    wait_for_message(self.receiver_rx.clone()),
                    |msg| cosmic::Action::App(Message::Loop(Box::new(msg)))
                );
                Task::batch(vec![task, next])
            }
            Message::Close => {
                std::process::exit(0);
            }
            Message::StatusUpdate(s) => {
                self.status = s;
                Task::none()
            }
            Message::Error(e) => {
                self.status = format!("Error: {}", e);
                Task::none()
            }
            Message::FrameReceived(handle) => {
                self.frame = Some(handle);
                Task::none()
            }
            Message::Connected => {
                self.status = "Connected".to_string();
                Task::none()
            }
            Message::DbusConnected(client) => {
                self.dbus = Some(client);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if let Some(handle) = &self.frame {
            container(
                image::viewer(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            container(
                column()
                    .push(text(format!("Mirroring: {}", self.device_id)).size(24))
                    .push(text(&self.status))
                    .push(button::text("Close").on_press(Message::Close))
                    .padding(20)
                    .spacing(10)
                    .align_x(cosmic::iced::Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::Alignment::Center)
            .align_y(cosmic::iced::Alignment::Center)
            .into()
        }
    }
}

async fn wait_for_message(rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Message {
    let mut rx = rx.lock().await;
    rx.recv().await.unwrap_or(Message::Close)
}

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<MirrorApp>(cosmic::app::Settings::default(), ())
}
