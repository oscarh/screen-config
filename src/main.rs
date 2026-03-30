mod app;
mod monitor_canvas;
mod sway;
mod types;

use app::App;

fn theme(_: &App) -> iced::Theme {
    iced::Theme::Dark
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("sc - Screen Config")
        .subscription(App::subscription)
        .window_size(iced::Size::new(600.0, 500.0))
        .theme(theme)
        .run()
}
