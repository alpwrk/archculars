use chrono::{DateTime, Utc};
use humansize::{format_size, DECIMAL};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::core::models::Package;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, pkg: Option<&Package>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::unfocused_border())
        .title(" Details ");
    let content: Vec<Line> = match pkg {
        Some(p) => render_pkg(p),
        None => vec![Line::from(Span::styled(
            "Select a package from the list",
            Style::default().fg(theme::MUTED),
        ))],
    };
    f.render_widget(
        Paragraph::new(content).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_pkg(p: &Package) -> Vec<Line<'_>> {
    let mut lines: Vec<Line<'_>> = Vec::new();
    lines.push(Line::from(Span::styled(
        p.name.clone(),
        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::raw(""));

    let source_color = theme::source_color(p.source_label());
    lines.push(field("Version", &p.version));
    lines.push(Line::from(vec![
        label("Source"),
        Span::styled(p.source_label().to_string(), Style::default().fg(source_color)),
    ]));
    lines.push(Line::from(vec![
        label("Status"),
        if p.installed {
            if p.needs_upgrade() {
                Span::styled("upgrade available", Style::default().fg(theme::WARNING))
            } else {
                Span::styled("installed", Style::default().fg(theme::INSTALLED))
            }
        } else {
            Span::styled("not installed", Style::default().fg(theme::MUTED))
        },
    ]));

    if let Some(iv) = &p.installed_version {
        if iv != &p.version {
            lines.push(field("Installed", iv));
        }
    }
    if let Some(size) = p.installed_size {
        lines.push(field("Size", &format_size(size, DECIMAL)));
    }
    if let Some(size) = p.download_size {
        lines.push(field("Download", &format_size(size, DECIMAL)));
    }
    if let Some(m) = &p.maintainer {
        lines.push(field("Maintainer", m));
    }
    if let Some(url) = &p.url {
        lines.push(field("URL", url));
    }
    if !p.licenses.is_empty() {
        lines.push(field("License", &p.licenses.join(", ")));
    }
    if let Some(v) = p.votes {
        let pop = p
            .popularity
            .map(|f| format!("  pop {f:.2}"))
            .unwrap_or_default();
        lines.push(field("AUR", &format!("{v} votes{pop}")));
    }
    if let Some(ts) = p.out_of_date {
        if let Some(dt) = ts_to_string(ts) {
            lines.push(Line::from(vec![
                label("Out of date"),
                Span::styled(dt, Style::default().fg(theme::OUT_OF_DATE)),
            ]));
        }
    }
    if let Some(ts) = p.last_modified {
        if let Some(dt) = ts_to_string(ts) {
            lines.push(field("Last update", &dt));
        }
    }

    if !p.description.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::raw(p.description.clone())));
    }

    if !p.depends.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            format!("Dependencies ({})", p.depends.len()),
            Style::default().fg(theme::ACCENT),
        )));
        for d in p.depends.iter().take(8) {
            lines.push(Line::from(format!("  • {d}")));
        }
        if p.depends.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("  … {} more — press [d] for full tree", p.depends.len() - 8),
                Style::default().fg(theme::MUTED),
            )));
        }
    }

    if !p.opt_depends.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            format!("Optional ({})", p.opt_depends.len()),
            Style::default().fg(theme::ACCENT),
        )));
        for d in p.opt_depends.iter().take(5) {
            lines.push(Line::from(format!("  • {d}")));
        }
    }

    lines
}

fn field<'a>(name: &'static str, value: &str) -> Line<'a> {
    Line::from(vec![label(name), Span::raw(value.to_string())])
}

fn label(name: &'static str) -> Span<'static> {
    let padded = format!("{name:>14}  ");
    Span::styled(padded, Style::default().fg(theme::MUTED))
}

fn ts_to_string(ts: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(ts, 0).map(|d| d.format("%Y-%m-%d").to_string())
}
