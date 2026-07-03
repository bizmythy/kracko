//! Terminal creation and configuration.

use iced::Font;
use iced::font::{Family, Stretch, Weight};

pub const FONT_BYTES: &[u8] = include_bytes!(
    "../assets/fonts/JetBrains/JetBrainsMonoNerdFontMono-Bold.ttf"
);

const SHELL: &str = "nu";

pub fn settings() -> iced_term::settings::Settings {
    iced_term::settings::Settings {
        font: iced_term::settings::FontSettings {
            size: 14.0,
            font_type: Font {
                weight: Weight::Bold,
                family: Family::Name("JetBrainsMono Nerd Font Mono"),
                stretch: Stretch::Normal,
                ..Default::default()
            },
            ..Default::default()
        },
        theme: iced_term::settings::ThemeSettings::default(),
        backend: iced_term::settings::BackendSettings {
            program: SHELL.to_string(),
            ..Default::default()
        },
    }
}

pub fn spawn(
    id: u64,
    settings: iced_term::settings::Settings,
) -> iced_term::Terminal {
    iced_term::Terminal::new(id, settings)
        .expect("failed to create the new terminal instance")
}
