use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::Focus;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, query: &str, focus: Focus, loading: bool) {
    let border = if focus == Focus::Search {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };

    let title = if loading {
        " Search … (loading) "
    } else if focus == Focus::Search {
        " Search (Esc → list) "
    } else {
        " Search  [/]  "
    };

    let display: Line = if query.is_empty() && focus != Focus::Search {
        Line::from(Span::styled(
            "Search packages … e.g. linux, firefox, hyprland",
            Style::default().fg(theme::MUTED),
        ))
    } else {
        Line::from(Span::raw(query.to_string()))
    };

    let p = Paragraph::new(display)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(border));
    f.render_widget(p, area);
}
