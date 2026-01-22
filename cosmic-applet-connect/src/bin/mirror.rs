use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget::{button, column, text};
use cosmic::Element;
use std::env;

struct MirrorApp {
    core: Core,
    device_id: String,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    Close,
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

        let app = Self {
            core,
            device_id,
            status: "Waiting for stream...".to_string(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column()
            .push(text(format!("Mirroring: {}", self.device_id)).size(24))
            .push(text(&self.status))
            .push(button::text("Close").on_press(Message::Close))
            .padding(20)
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::Alignment::Center)
            .into()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<MirrorApp>(cosmic::app::Settings::default(), ())
}