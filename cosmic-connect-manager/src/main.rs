mod dbus_client;

use clap::Parser;
use cosmic::{
    app::{Core, Task},
    iced::{Alignment, Length, Size},
    theme,
    widget::{button, column, container, icon, row, scrollable, text, vertical_space},
    Application, Element,
};

/// Navigation pages in the manager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Devices,
    MediaPlayers,
    Transfers,
    History,
    Settings,
}

impl Page {
    fn title(&self) -> &'static str {
        match self {
            Page::Devices => "Devices",
            Page::MediaPlayers => "Media",
            Page::Transfers => "Transfers",
            Page::History => "History",
            Page::Settings => "Settings",
        }
    }

    fn icon_name(&self) -> &'static str {
        match self {
            Page::Devices => "computer-symbolic",
            Page::MediaPlayers => "multimedia-player-symbolic",
            Page::Transfers => "folder-download-symbolic",
            Page::History => "document-open-recent-symbolic",
            Page::Settings => "preferences-system-symbolic",
        }
    }
}

use dbus_client::{DbusClient, DeviceConfig, DeviceInfo};
use std::collections::HashMap;

const APP_ID: &str = "com.system76.CosmicConnectManager";

#[derive(Parser, Debug, Clone)]
#[command(name = "cosmic-connect-manager")]
#[command(about = "COSMIC Connect Device Manager")]
pub struct Args {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub action: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeviceAction {
    SendFile,
    Ping,
    Find,
    ScreenShare,
    Settings,
}

impl DeviceAction {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "send-file" => Some(DeviceAction::SendFile),
            "ping" => Some(DeviceAction::Ping),
            "find" => Some(DeviceAction::Find),
            "screen-share" => Some(DeviceAction::ScreenShare),
            "settings" => Some(DeviceAction::Settings),
            _ => None,
        }
    }
}

fn device_icon_name(device_type: &str) -> &'static str {
    match device_type {
        "phone" => "phone-symbolic",
        "tablet" => "tablet-symbolic",
        "desktop" | "laptop" => "computer-symbolic",
        _ => "network-wireless-symbolic",
    }
}

fn connection_status(device: &DeviceInfo) -> &'static str {
    if device.is_connected {
        "Connected"
    } else if device.is_reachable {
        "Available"
    } else {
        "Offline"
    }
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let settings = cosmic::app::Settings::default().size(Size::new(900.0, 700.0));
    cosmic::app::run::<CosmicConnectManager>(settings, args)
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    SelectDevice(String),
    DevicesUpdated(HashMap<String, DeviceInfo>),
    DeviceConfigLoaded(String, DeviceConfig),
    ExecuteAction(String, DeviceAction),
    DbusReady(DbusClient),
    MediaPlayPause(String),
    MediaNext(String),
    MediaPrevious(String),
    CancelTransfer(String),
    None,
}

pub struct CosmicConnectManager {
    core: Core,
    active_page: Page,
    dbus_client: Option<DbusClient>,
    devices: HashMap<String, DeviceInfo>,
    device_configs: HashMap<String, DeviceConfig>,
    selected_device: Option<String>,
    initial_device: Option<String>,
    initial_action: Option<DeviceAction>,
    dbus_ready: bool,
}

impl CosmicConnectManager {
    fn sidebar_view(&self) -> Element<Message> {
        let pages = [
            Page::Devices,
            Page::MediaPlayers,
            Page::Transfers,
            Page::History,
            Page::Settings,
        ];

        let mut nav_items = column::with_capacity(pages.len() + 1)
            .spacing(theme::active().cosmic().space_xxs());

        for page in pages {
            let is_active = self.active_page == page;
            let item_icon = icon::from_name(page.icon_name()).size(20);
            let item_label = text(page.title()).size(14);

            let item_content = row::with_capacity(2)
                .spacing(theme::active().cosmic().space_s())
                .align_y(Alignment::Center)
                .push(item_icon)
                .push(item_label);

            let item_container = container(item_content)
                .padding(theme::active().cosmic().space_s())
                .width(Length::Fill);

            let nav_button = if is_active {
                button::custom(item_container)
                    .class(theme::Button::Suggested)
            } else {
                button::custom(item_container)
                    .class(theme::Button::Text)
            };

            nav_items = nav_items.push(
                nav_button
                    .on_press(Message::NavigateTo(page))
                    .padding(0)
                    .width(Length::Fill)
            );
        }

        container(
            column::with_capacity(2)
                .push(
                    text("COSMIC Connect")
                        .size(18)
                )
                .push(vertical_space().height(theme::active().cosmic().space_m()))
                .push(nav_items)
        )
        .padding(theme::active().cosmic().space_m())
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .into()
    }

    fn content_view(&self) -> Element<Message> {
        match self.active_page {
            Page::Devices => self.device_list_view(),
            Page::MediaPlayers => self.media_players_view(),
            Page::Transfers => self.transfers_view(),
            Page::History => self.placeholder_view("History", "document-open-recent-symbolic"),
            Page::Settings => self.placeholder_view("Settings", "preferences-system-symbolic"),
        }
    }

    fn placeholder_view(&self, title: &'static str, icon_name: &'static str) -> Element<'static, Message> {
        container(
            column::with_capacity(2)
                .spacing(theme::active().cosmic().space_s())
                .align_x(Alignment::Center)
                .push(icon::from_name(icon_name).size(64))
                .push(text(title).size(24))
                .push(text("Coming soon").size(14))
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn device_list_view(&self) -> Element<Message> {
        let mut connected_devices = Vec::new();
        let mut available_devices = Vec::new();
        let mut offline_devices = Vec::new();

        for (device_id, device) in &self.devices {
            let config = self.device_configs.get(device_id);
            let is_selected = self.selected_device.as_ref() == Some(device_id);
            let card = self.device_card(device_id, device, config, is_selected);

            if device.is_connected {
                connected_devices.push(card);
            } else if device.is_reachable {
                available_devices.push(card);
            } else {
                offline_devices.push(card);
            }
        }

        let mut sections = column::with_capacity(6)
            .spacing(theme::active().cosmic().space_m())
            .padding(theme::active().cosmic().space_m());

        if !connected_devices.is_empty() {
            sections = sections.push(text("Connected").size(14));
            for device in connected_devices {
                sections = sections.push(device);
            }
        }

        if !available_devices.is_empty() {
            sections = sections.push(text("Available").size(14));
            for device in available_devices {
                sections = sections.push(device);
            }
        }

        if !offline_devices.is_empty() {
            sections = sections.push(text("Offline").size(14));
            for device in offline_devices {
                sections = sections.push(device);
            }
        }

        if self.devices.is_empty() {
            sections = sections.push(
                container(
                    column::with_capacity(3)
                        .spacing(theme::active().cosmic().space_s())
                        .align_x(Alignment::Center)
                        .push(icon::from_name("network-wireless-offline-symbolic").size(64))
                        .push(text("No devices found").size(18))
                        .push(text("Make sure devices are on the same network").size(14))
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            );
        }

        container(sections)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn media_players_view(&self) -> Element<Message> {
        let mut sections = column::with_capacity(4)
            .spacing(theme::active().cosmic().space_m())
            .padding(theme::active().cosmic().space_m());

        sections = sections.push(text("Media Players").size(18));

        let placeholder_players = vec![
            ("firefox", "Firefox"),
            ("spotify", "Spotify"),
        ];

        if placeholder_players.is_empty() {
            sections = sections.push(
                container(
                    column::with_capacity(3)
                        .spacing(theme::active().cosmic().space_s())
                        .align_x(Alignment::Center)
                        .push(icon::from_name("multimedia-player-symbolic").size(64))
                        .push(text("No media players found").size(18))
                        .push(text("Play media on connected devices to see players").size(14))
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            );
        } else {
            for (player_id, player_name) in placeholder_players {
                sections = sections.push(self.media_player_card(player_id, player_name));
            }
        }

        container(sections)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn media_player_card(&self, player_id: &str, player_name: &str) -> Element<'static, Message> {
        let player_icon = icon::from_name("multimedia-player-symbolic").size(48);
        let name_text = text(player_name.to_string()).size(16);
        let track_info = text("No track playing").size(12);

        let info_column = column::with_capacity(2)
            .spacing(theme::active().cosmic().space_xxs())
            .push(name_text)
            .push(track_info);

        let header_row = row::with_capacity(2)
            .spacing(theme::active().cosmic().space_s())
            .align_y(Alignment::Center)
            .push(player_icon)
            .push(info_column);

        let prev_button = button::icon(icon::from_name("media-skip-backward-symbolic").size(16))
            .on_press(Message::MediaPrevious(player_id.to_string()))
            .padding(theme::active().cosmic().space_xxs());

        let play_pause_button = button::icon(icon::from_name("media-playback-start-symbolic").size(16))
            .on_press(Message::MediaPlayPause(player_id.to_string()))
            .padding(theme::active().cosmic().space_xxs());

        let next_button = button::icon(icon::from_name("media-skip-forward-symbolic").size(16))
            .on_press(Message::MediaNext(player_id.to_string()))
            .padding(theme::active().cosmic().space_xxs());

        let controls_row = row::with_capacity(3)
            .spacing(theme::active().cosmic().space_xs())
            .push(prev_button)
            .push(play_pause_button)
            .push(next_button);

        let card_content = column::with_capacity(2)
            .spacing(theme::active().cosmic().space_s())
            .push(header_row)
            .push(controls_row);

        container(card_content)
            .padding(theme::active().cosmic().space_s())
            .width(Length::Fill)
            .into()
    }

    fn transfers_view(&self) -> Element<Message> {
        let mut content = column::with_capacity(4)
            .spacing(theme::active().cosmic().space_m())
            .padding(theme::active().cosmic().space_m());

        content = content.push(text("Active Transfers (2)").size(16));

        let active_transfer_1 = self.transfer_card(
            "transfer_1",
            "document.pdf",
            "text-x-generic-symbolic",
            65,
            "2.3 MB/s",
            true,
        );
        content = content.push(active_transfer_1);

        let active_transfer_2 = self.transfer_card(
            "transfer_2",
            "photo.jpg",
            "image-x-generic-symbolic",
            35,
            "1.1 MB/s",
            true,
        );
        content = content.push(active_transfer_2);

        content = content.push(vertical_space().height(theme::active().cosmic().space_m()));
        content = content.push(text("Completed Today (5)").size(16));

        let completed_items = column::with_capacity(5)
            .spacing(theme::active().cosmic().space_xs())
            .push(self.completed_transfer_item("report.docx", "text-x-generic-symbolic", "2.1 MB", "10:32 AM"))
            .push(self.completed_transfer_item("vacation.jpg", "image-x-generic-symbolic", "4.5 MB", "09:15 AM"))
            .push(self.completed_transfer_item("presentation.pptx", "x-office-presentation-symbolic", "8.2 MB", "08:45 AM"))
            .push(self.completed_transfer_item("video.mp4", "video-x-generic-symbolic", "125 MB", "08:12 AM"))
            .push(self.completed_transfer_item("archive.zip", "package-x-generic-symbolic", "15.3 MB", "07:55 AM"));

        content = content.push(completed_items);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn transfer_card(
        &self,
        transfer_id: &str,
        filename: &str,
        icon_name: &str,
        progress: u8,
        speed: &str,
        is_active: bool,
    ) -> Element<Message> {
        let file_icon = icon::from_name(icon_name).size(24);
        let filename_text = text(filename.to_string()).size(14);

        let header_row = row::with_capacity(2)
            .spacing(theme::active().cosmic().space_s())
            .align_y(Alignment::Center)
            .push(file_icon)
            .push(filename_text);

        let progress_text = format!("{}%", progress);
        let progress_label = text(progress_text).size(12);
        let speed_label = text(speed.to_string()).size(12);

        let mut info_row = row::with_capacity(3)
            .spacing(theme::active().cosmic().space_s())
            .align_y(Alignment::Center)
            .push(progress_label)
            .push(text("·").size(12))
            .push(speed_label);

        if is_active {
            let cancel_button = button::text("Cancel")
                .on_press(Message::CancelTransfer(transfer_id.to_string()))
                .class(theme::Button::Destructive)
                .padding(theme::active().cosmic().space_xxs());

            info_row = info_row.push(cancel_button);
        }

        let card_content = column::with_capacity(2)
            .spacing(theme::active().cosmic().space_xs())
            .push(header_row)
            .push(info_row);

        container(card_content)
            .padding(theme::active().cosmic().space_s())
            .width(Length::Fill)
            .into()
    }

    fn completed_transfer_item(
        &self,
        filename: &str,
        icon_name: &str,
        size: &str,
        time: &str,
    ) -> Element<Message> {
        let file_icon = icon::from_name(icon_name).size(20);
        let filename_text = text(filename.to_string()).size(14);
        let size_text = text(size.to_string()).size(12);
        let time_text = text(time.to_string()).size(12);

        let item_row = row::with_capacity(5)
            .spacing(theme::active().cosmic().space_s())
            .align_y(Alignment::Center)
            .push(file_icon)
            .push(filename_text)
            .push(text("-").size(12))
            .push(size_text)
            .push(text("-").size(12))
            .push(time_text);

        container(item_row)
            .padding(theme::active().cosmic().space_xs())
            .width(Length::Fill)
            .into()
    }

    fn device_card<'a>(
        &self,
        device_id: &'a str,
        device: &'a DeviceInfo,
        config: Option<&'a DeviceConfig>,
        is_selected: bool,
    ) -> Element<'a, Message> {
        let device_icon = icon::from_name(device_icon_name(&device.device_type)).size(32);
        let display_name = config
            .and_then(|c| c.nickname.as_deref())
            .unwrap_or(&device.name);
        let name_text = text(display_name).size(16);
        let status_text = connection_status(device);
        let status_badge = text(status_text).size(12);

        let mut info_row = row::with_capacity(2)
            .spacing(theme::active().cosmic().space_s())
            .align_y(Alignment::Center)
            .push(device_icon)
            .push(
                column::with_capacity(2)
                    .spacing(theme::active().cosmic().space_xxs())
                    .push(name_text)
                    .push(status_badge)
            );

        if let Some(cfg) = config {
            if cfg.plugins.enable_battery.unwrap_or(true) {
                info_row = info_row.push(icon::from_name("battery-symbolic").size(16));
            }
        }

        let mut card_content = column::with_capacity(2)
            .spacing(theme::active().cosmic().space_s())
            .push(info_row);

        if device.is_connected {
            let ping_button = button::icon(icon::from_name("network-transmit-receive-symbolic").size(16))
                .on_press(Message::ExecuteAction(device_id.to_string(), DeviceAction::Ping))
                .padding(theme::active().cosmic().space_xxs());

            let send_file_button = button::icon(icon::from_name("document-send-symbolic").size(16))
                .on_press(Message::ExecuteAction(device_id.to_string(), DeviceAction::SendFile))
                .padding(theme::active().cosmic().space_xxs());

            let find_button = button::icon(icon::from_name("find-location-symbolic").size(16))
                .on_press(Message::ExecuteAction(device_id.to_string(), DeviceAction::Find))
                .padding(theme::active().cosmic().space_xxs());

            let actions_row = row::with_capacity(3)
                .spacing(theme::active().cosmic().space_xs())
                .push(ping_button)
                .push(send_file_button)
                .push(find_button);

            card_content = card_content.push(actions_row);
        }

        let card_container = container(card_content)
            .padding(theme::active().cosmic().space_s())
            .width(Length::Fill);

        let card_button = if is_selected {
            button::custom(card_container)
        } else {
            button::custom(card_container)
        };

        card_button
            .on_press(Message::SelectDevice(device_id.to_string()))
            .padding(0)
            .width(Length::Fill)
            .into()
    }
}

impl Application for CosmicConnectManager {
    type Executor = cosmic::executor::Default;
    type Flags = Args;
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let initial_device = flags.device.clone();
        let initial_action = flags.action.as_deref().and_then(DeviceAction::from_str);

        (
            CosmicConnectManager {
                core,
                active_page: Page::Devices,
                dbus_client: None,
                devices: HashMap::new(),
                device_configs: HashMap::new(),
                selected_device: initial_device.clone(),
                initial_device,
                initial_action,
                dbus_ready: false,
            },
            Task::none(),
        )
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    fn view(&self) -> Element<Self::Message> {
        let sidebar = self.sidebar_view();
        let content = scrollable(self.content_view())
            .width(Length::Fill)
            .height(Length::Fill);

        row::with_capacity(2)
            .push(sidebar)
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NavigateTo(page) => {
                self.active_page = page;
                Task::none()
            }
            Message::SelectDevice(device_id) => {
                self.selected_device = Some(device_id);
                Task::none()
            }
            Message::DevicesUpdated(devices) => {
                self.devices = devices;
                Task::none()
            }
            Message::DeviceConfigLoaded(device_id, config) => {
                self.device_configs.insert(device_id, config);
                Task::none()
            }
            Message::ExecuteAction(_device_id, _action) => Task::none(),
            Message::DbusReady(client) => {
                self.dbus_client = Some(client);
                self.dbus_ready = true;
                Task::none()
            }
            Message::MediaPlayPause(_device_id) => Task::none(),
            Message::MediaNext(_device_id) => Task::none(),
            Message::MediaPrevious(_device_id) => Task::none(),
            Message::CancelTransfer(_transfer_id) => Task::none(),
            Message::None => Task::none(),
        }
    }
}
