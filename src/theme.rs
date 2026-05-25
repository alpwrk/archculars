use ratatui::style::{Color, Style};

pub const ACCENT: Color = Color::LightRed;
pub const AUR: Color = Color::Yellow;
pub const REPO: Color = Color::Green;
pub const MULTILIB: Color = Color::Blue;
pub const INSTALLED: Color = Color::Green;
pub const MUTED: Color = Color::DarkGray;
pub const OUT_OF_DATE: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;
pub const SUCCESS: Color = Color::Green;

pub fn source_color(label: &str) -> Color {
    match label {
        "AUR" => AUR,
        "multilib" | "multilib-testing" => MULTILIB,
        "core" | "extra" | "core-testing" | "extra-testing" => REPO,
        _ => Color::Cyan,
    }
}

pub fn focused_border() -> Style {
    Style::default().fg(ACCENT)
}

pub fn unfocused_border() -> Style {
    Style::default().fg(MUTED)
}
