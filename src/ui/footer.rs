use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme;

pub fn render(f: &mut Frame, area: Rect, hint: Option<&str>) {
    let line = match hint {
        Some(msg) => Line::from(Span::styled(
            format!(" 💡 {msg} "),
            Style::default().fg(ratatui::style::Color::Black).bg(theme::WARNING),
        )),
        None => Line::from(vec![
            key(" / "),
            label(" Search  "),
            key(" ↑↓ "),
            label(" Nav  "),
            key(" Enter "),
            label(" Install/Remove  "),
            key(" d "),
            label(" Deps  "),
            key(" p "),
            label(" PKGBUILD  "),
            key(" u "),
            label(" Upgrades  "),
            key(" n "),
            label(" News  "),
            key(" f "),
            label(" Filter  "),
            key(" Ctrl+Q "),
            label(" Quit"),
        ]),
    };
    f.render_widget(Paragraph::new(line), area);
}

fn key(s: &'static str) -> Span<'static> {
    Span::styled(
        s,
        Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )
}

fn label(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}
