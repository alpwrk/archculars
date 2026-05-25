use humansize::{format_size, DECIMAL};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::core::models::Package;
use crate::theme;
use crate::ui::install_modal::centered;

pub fn render(
    f: &mut Frame,
    area: Rect,
    largest: &[Package],
    orphans: &[Package],
    scroll: u16,
) {
    let popup = centered(area, 100, 30);
    f.render_widget(Clear, popup);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(popup);

    let mut left_lines: Vec<Line<'_>> = Vec::new();
    for p in largest {
        left_lines.push(Line::from(vec![
            Span::styled(
                format_size(p.installed_size.unwrap_or(0), DECIMAL),
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(p.name.clone()),
        ]));
    }
    let left = Paragraph::new(left_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT))
                .title(" Largest packages "),
        );
    f.render_widget(left, layout[0]);

    let mut right_lines: Vec<Line<'_>> = Vec::new();
    if orphans.is_empty() {
        right_lines.push(Line::from(Span::styled(
            "No obvious orphans",
            Style::default().fg(theme::SUCCESS),
        )));
    }
    for p in orphans {
        right_lines.push(Line::from(vec![
            Span::raw(p.name.clone()),
            Span::raw("  "),
            Span::styled(
                format_size(p.installed_size.unwrap_or(0), DECIMAL),
                Style::default().fg(theme::MUTED),
            ),
        ]));
    }
    let right = Paragraph::new(right_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::MUTED))
                .title(format!(" Orphans ({}) ", orphans.len())),
        );
    f.render_widget(right, layout[1]);
}
