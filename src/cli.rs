use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "archculars",
    version,
    about = "Modern TUI for Arch Linux + AUR package management",
    long_about = None,
)]
pub struct Args {
    /// Optional search term to populate the search bar on start.
    pub query: Option<String>,

    /// Limit repo searches to a comma-separated list of repos (e.g. core,extra).
    #[arg(short = 'r', long = "repos", value_delimiter = ',')]
    pub repos: Vec<String>,

    /// Open the upgrades view on start.
    #[arg(short = 'u', long = "upgrades")]
    pub upgrades: bool,

    /// Pre-select the "installed" filter on start.
    #[arg(short = 'i', long = "installed")]
    pub installed: bool,
}
