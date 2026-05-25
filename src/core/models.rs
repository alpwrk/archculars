use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub source: Source,
    pub description: String,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub installed_size: Option<u64>,
    pub download_size: Option<u64>,
    pub maintainer: Option<String>,
    pub url: Option<String>,
    pub licenses: Vec<String>,
    pub depends: Vec<String>,
    pub make_depends: Vec<String>,
    pub opt_depends: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    /// AUR votes
    pub votes: Option<i32>,
    /// AUR popularity score
    pub popularity: Option<f64>,
    /// AUR out-of-date flag (unix timestamp)
    pub out_of_date: Option<i64>,
    /// AUR last modification unix timestamp
    pub last_modified: Option<i64>,
}

impl Package {
    pub fn is_aur(&self) -> bool {
        matches!(self.source, Source::Aur)
    }

    pub fn source_label(&self) -> &str {
        self.source.label()
    }

    pub fn needs_upgrade(&self) -> bool {
        match &self.installed_version {
            Some(v) => v != &self.version,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Source {
    Repo(String),
    Aur,
}

impl Source {
    pub fn label(&self) -> &str {
        match self {
            Source::Repo(s) => s.as_str(),
            Source::Aur => "AUR",
        }
    }
}

/// Filter state for the main table view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    All,
    Installed,
    Repos,
    Aur,
    Upgrades,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Installed => "Installed",
            Filter::Repos => "Repos",
            Filter::Aur => "AUR",
            Filter::Upgrades => "Upgrades",
        }
    }

    pub fn matches(self, pkg: &Package) -> bool {
        match self {
            Filter::All => true,
            Filter::Installed => pkg.installed,
            Filter::Repos => !pkg.is_aur(),
            Filter::Aur => pkg.is_aur(),
            Filter::Upgrades => pkg.installed && pkg.needs_upgrade(),
        }
    }
}

/// One node in the dependency tree shown in the deps modal.
#[derive(Debug, Clone)]
pub struct DepNode {
    pub name: String,
    pub kind: DepKind,
    pub installed: bool,
    pub children: Vec<DepNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Required,
    Make,
    Optional,
}

impl DepKind {
    pub fn label(self) -> &'static str {
        match self {
            DepKind::Required => "depends",
            DepKind::Make => "makedepends",
            DepKind::Optional => "optdepends",
        }
    }
}
