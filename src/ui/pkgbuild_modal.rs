use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::theme;
use crate::ui::install_modal::centered;

pub fn render(f: &mut Frame, area: Rect, name: &str, body: Option<&str>, scroll: u16) {
    let popup = centered(area, 100, 30);
    f.render_widget(Clear, popup);

    let title = format!(" PKGBUILD · {name} ");
    let lines: Vec<Line<'_>> = match body {
        Some(b) => highlight(b),
        None => vec![Line::from(Span::styled(
            "Loading PKGBUILD …",
            Style::default().fg(theme::MUTED),
        ))],
    };

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

fn highlight(body: &str) -> Vec<Line<'static>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-eighties.dark"];
    let syntax = ss
        .find_syntax_by_extension("sh")
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, theme);

    let mut lines: Vec<Line<'static>> = Vec::new();
    for line in LinesWithEndings::from(body) {
        let ranges = h.highlight_line(line, &ss).unwrap_or_default();
        let spans: Vec<Span<'static>> = ranges
            .into_iter()
            .map(|(style, text)| {
                let cleaned = text.trim_end_matches('\n').to_string();
                Span::styled(cleaned, to_ratatui_style(style))
            })
            .filter(|s| !s.content.is_empty())
            .collect();
        lines.push(Line::from(spans));
    }
    lines
}

fn to_ratatui_style(s: SynStyle) -> Style {
    let mut out = Style::default().fg(Color::Rgb(
        s.foreground.r,
        s.foreground.g,
        s.foreground.b,
    ));
    if s.font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        out = out.add_modifier(Modifier::BOLD);
    }
    if s.font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        out = out.add_modifier(Modifier::ITALIC);
    }
    out
}
