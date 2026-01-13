use cosmic::{app::Core, applet, iced::window, Application, Element};

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    applet::run::<KdeConnectApplet>(())
}

struct KdeConnectApplet {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for KdeConnectApplet {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    const APP_ID: &'static str = "com.system76.CosmicAppletKdeConnect";

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::iced::Command<Message>) {
        (Self { core }, cosmic::iced::Command::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, _message: Self::Message) -> cosmic::iced::Command<Self::Message> {
        cosmic::iced::Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core.applet.icon_button("phone-symbolic").into()
    }
}
