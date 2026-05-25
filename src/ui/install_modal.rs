use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::install::InstallState;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, state: &InstallState) {
    let popup = centered(area, 80, 22);
    f.render_widget(Clear, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(popup);

    let title = format!(
        " {} {} ",
        match state.action {
            crate::core::pacman::Action::Install => "Install",
            crate::core::pacman::Action::Remove => "Remove",
        },
        state.package_name
    );
    let header = Paragraph::new(Line::from(Span::styled(
        format!("Command: {}", state.command_preview),
        Style::default().fg(theme::MUTED),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT))
            .title(title),
    );
    f.render_widget(header, layout[0]);

    let body_lines: Vec<Line<'_>> = state
        .log
        .iter()
        .map(|entry| match entry {
            crate::core::pacman::LogLine::Stdout(s) => Line::from(s.clone()),
            crate::core::pacman::LogLine::Stderr(s) => {
                Line::from(Span::styled(s.clone(), Style::default().fg(theme::OUT_OF_DATE)))
            }
            crate::core::pacman::LogLine::Exit(code) => {
                let color = if *code == 0 { theme::SUCCESS } else { theme::OUT_OF_DATE };
                Line::from(Span::styled(
                    format!("→ Exit {code}"),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))
            }
        })
        .collect();

    let log = Paragraph::new(body_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::MUTED))
                .title(" Output "),
        )
        .scroll((state.scroll, 0));
    f.render_widget(log, layout[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" close   ", Style::default().fg(theme::MUTED)),
        Span::styled(
            " PgUp/PgDn ",
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll", Style::default().fg(theme::MUTED)),
    ]));
    f.render_widget(footer, layout[2]);
}

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}
