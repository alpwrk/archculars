use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::core::models::{DepKind, DepNode};
use crate::theme;
use crate::ui::install_modal::centered;

pub fn render(f: &mut Frame, area: Rect, root: &DepNode, scroll: u16) {
    let popup = centered(area, 90, 26);
    f.render_widget(Clear, popup);

    let mut lines: Vec<Line<'_>> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("{}  (dependency tree)", root.name),
        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::raw(""));
    render_node(root, 0, true, &mut lines);

    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT))
                .title(" Deps "),
        );
    f.render_widget(body, popup);
}

fn render_node(node: &DepNode, depth: usize, root: bool, out: &mut Vec<Line<'_>>) {
    if !root {
        let prefix = "  ".repeat(depth);
        let bullet = match node.kind {
            DepKind::Required => "├─",
            DepKind::Make => "├─[make]",
            DepKind::Optional => "├─[opt]",
        };
        let style = if node.installed {
            Style::default().fg(theme::INSTALLED)
        } else {
            Style::default().fg(theme::MUTED)
        };
        let mark = if node.installed { "✓" } else { "—" };
        out.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{bullet} "), Style::default().fg(theme::MUTED)),
            Span::styled(node.name.clone(), style),
            Span::raw(" "),
            Span::styled(format!("[{mark}]"), Style::default().fg(theme::MUTED)),
        ]));
    }
    for child in &node.children {
        render_node(child, depth + 1, false, out);
    }
}
