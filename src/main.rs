use anyhow::Result;
use archculars::{app, cli};
use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use std::io::stdout;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = cli::Args::parse();
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture).ok();

    let result = app::run(&mut terminal, args).await;

    execute!(stdout(), DisableMouseCapture).ok();
    ratatui::restore();

    if let Err(e) = &result {
        eprintln!("archculars error: {e:?}");
    }
    result
}
