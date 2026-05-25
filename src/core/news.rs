use anyhow::{Context, Result};
use feed_rs::parser;
use serde::{Deserialize, Serialize};

const FEED_URL: &str = "https://archlinux.org/feeds/news/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub link: Option<String>,
    pub published: Option<String>,
    pub summary: String,
}

/// Fetch + parse the official Arch news feed. Returns up to `limit` items.
pub async fn fetch(limit: usize) -> Result<Vec<NewsItem>> {
    let resp = reqwest::get(FEED_URL).await.context("GET arch news feed")?;
    let bytes = resp.bytes().await.context("read feed body")?;
    let feed = parser::parse(bytes.as_ref()).context("parse RSS/Atom feed")?;
    let items: Vec<NewsItem> = feed
        .entries
        .into_iter()
        .take(limit)
        .map(|e| NewsItem {
            title: e.title.map(|t| t.content).unwrap_or_default(),
            link: e.links.first().map(|l| l.href.clone()),
            published: e.published.map(|d| d.to_rfc2822()),
            summary: e
                .summary
                .map(|s| strip_html(&s.content))
                .or_else(|| e.content.and_then(|c| c.body.map(|b| strip_html(&b))))
                .unwrap_or_default(),
        })
        .collect();
    Ok(items)
}

/// Crude HTML stripper for news summaries — good enough for a TUI preview.
fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
