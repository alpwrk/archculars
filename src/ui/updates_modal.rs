use humansize::{format_size, DECIMAL};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::core::models::Package;
use crate::theme;
use crate::ui::install_modal::centered;

pub fn render(f: &mut Frame, area: Rect, upgrades: &[Package], scroll: u16, loading: bool) {
    let popup = centered(area, 100, 30);
    f.render_widget(Clear, popup);

    let title = if loading {
        " Upgrades · loading … ".to_string()
    } else {
        format!(" Upgrades · {} available ", upgrades.len())
    };

    let mut lines: Vec<Line<'_>> = Vec::new();
    if upgrades.is_empty() && !loading {
        lines.push(Line::from(Span::styled(
            "✨ System is up to date — no updates",
            Style::default().fg(theme::SUCCESS),
        )));
    }
    for p in upgrades {
        let from = p
            .installed_version
            .clone()
            .unwrap_or_else(|| "?".to_string());
        let size = p
            .download_size
            .map(|s| format_size(s, DECIMAL))
            .unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled(
                p.name.clone(),
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(from, Style::default().fg(theme::MUTED)),
            Span::styled(" → ", Style::default().fg(theme::WARNING)),
            Span::raw(p.version.clone()),
            Span::raw("  "),
            Span::styled(format!("[{}]", p.source_label()), Style::default().fg(theme::source_color(p.source_label()))),
            Span::raw("  "),
            Span::styled(size, Style::default().fg(theme::MUTED)),
        ]));
        if !p.description.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  {}", p.description),
                Style::default().fg(theme::MUTED),
            )));
        }
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
