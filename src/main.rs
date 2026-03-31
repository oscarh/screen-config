mod app;
mod monitor_canvas;
mod sway;
mod types;

use app::App;

pub const APP_ID: &str = "simple_sway_screen_config";

fn theme(_: &App) -> iced::Theme {
    iced::Theme::Dark
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Screen Config")
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(600.0, 500.0),
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: APP_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .theme(theme)
        .run()
}
