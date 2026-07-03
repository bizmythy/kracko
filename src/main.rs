mod app;
mod horizontal_scroll;
mod style;
mod terminal;

use app::App;
use iced::Size;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .window_size(Size {
            width: 1280.0,
            height: 720.0,
        })
        .subscription(App::subscription)
        .font(terminal::FONT_BYTES)
        .run()
}
