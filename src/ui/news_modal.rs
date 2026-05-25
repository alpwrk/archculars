use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::core::news::NewsItem;
use crate::theme;
use crate::ui::install_modal::centered;

pub fn render(f: &mut Frame, area: Rect, items: &[NewsItem], scroll: u16, loading: bool) {
    let popup = centered(area, 100, 30);
    f.render_widget(Clear, popup);

    let title = if loading {
        " Arch News · loading … "
    } else {
        " Arch News "
    };

    let mut lines: Vec<Line<'_>> = Vec::new();
    for item in items {
        lines.push(Line::from(Span::styled(
            item.title.clone(),
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        )));
        if let Some(p) = &item.published {
            lines.push(Line::from(Span::styled(
                p.clone(),
                Style::default().fg(theme::MUTED),
            )));
        }
        if !item.summary.is_empty() {
            lines.push(Line::from(item.summary.clone()));
        }
        if let Some(l) = &item.link {
            lines.push(Line::from(Span::styled(
                l.clone(),
                Style::default().fg(theme::MUTED),
            )));
        }
        lines.push(Line::raw(""));
    }

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT))
                .title(title),
        );
    f.render_widget(p, popup);
}
