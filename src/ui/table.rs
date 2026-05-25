use humansize::{format_size, DECIMAL};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

use crate::app::Focus;
use crate::core::models::{Filter, Package};
use crate::theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    packages: &[Package],
    state: &mut TableState,
    focus: Focus,
    filter: Filter,
) {
    let border = if focus == Focus::Table {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };

    let header = Row::new(vec!["Package", "Version", "Source", "Size", "Description", "Inst."])
        .style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = packages
        .iter()
        .map(|p| {
            let size = p
                .installed_size
                .or(p.download_size)
                .map(|s| format_size(s, DECIMAL))
                .unwrap_or_else(|| "—".into());
            let source_label = p.source_label();
            let source_style = Style::default().fg(theme::source_color(source_label));
            let inst = if p.installed {
                if p.needs_upgrade() {
                    Span::styled("↑", Style::default().fg(theme::WARNING).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("✓", Style::default().fg(theme::INSTALLED))
                }
            } else {
                Span::styled("—", Style::default().fg(theme::MUTED))
            };
            let desc = truncate(&p.description, 80);
            Row::new(vec![
                Cell::from(p.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(p.version.clone()),
                Cell::from(source_label.to_string()).style(source_style),
                Cell::from(size),
                Cell::from(desc),
                Cell::from(inst),
            ])
        })
        .collect();

    let title = format!(" Packages · {} · {} hits ", filter.label(), packages.len());

    let table = Table::new(
        rows,
        [
            Constraint::Length(28),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(5),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(title),
    )
    .row_highlight_style(
        Style::default()
            .bg(theme::ACCENT)
            .fg(ratatui::style::Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, state);
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}
