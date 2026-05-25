use alpm::{Alpm, SigLevel};
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;

use super::models::{Package, Source};

/// Thin wrapper around an `alpm::Alpm` handle.
///
/// `alpm::Alpm` is not `Send` — it owns raw callback pointers and an FFI
/// handle. We therefore keep it in a `RefCell` and consume the wrapper only
/// from the main thread. Long-running operations (downloads, AUR queries)
/// must live on a different runtime task, but everything in this module
/// stays on the current thread.
pub struct AlpmBackend {
    handle: RefCell<Alpm>,
    repo_order: Vec<String>,
}

impl AlpmBackend {
    pub fn new() -> Result<Self> {
        let conf = pacmanconf::Config::new().context("failed to read /etc/pacman.conf")?;
        let alpm = init_alpm(&conf).context("failed to initialise libalpm")?;
        let repo_order = conf.repos.iter().map(|r| r.name.clone()).collect();
        Ok(Self {
            handle: RefCell::new(alpm),
            repo_order,
        })
    }

    pub fn reload(&self) -> Result<()> {
        let conf = pacmanconf::Config::new().context("reload /etc/pacman.conf")?;
        let alpm = init_alpm(&conf).context("reload libalpm")?;
        *self.handle.borrow_mut() = alpm;
        Ok(())
    }

    pub fn repo_names(&self) -> &[String] {
        &self.repo_order
    }

    pub fn installed(&self) -> Result<Vec<Package>> {
        let guard = self.handle.borrow();
        let mut out: Vec<Package> = Vec::new();
        for pkg in guard.localdb().pkgs() {
            out.push(local_to_package(pkg));
        }
        Ok(out)
    }

    pub fn installed_index(&self) -> Result<HashMap<String, String>> {
        let guard = self.handle.borrow();
        let mut map: HashMap<String, String> = HashMap::new();
        for pkg in guard.localdb().pkgs() {
            map.insert(pkg.name().to_string(), pkg.version().to_string());
        }
        Ok(map)
    }

    pub fn search_sync(&self, query: &str, repo_filter: &[String]) -> Result<Vec<Package>> {
        let guard = self.handle.borrow();
        let installed = build_installed_index(&guard);
        let needle = query.to_string();
        let mut out: Vec<Package> = Vec::new();
        for db in guard.syncdbs() {
            let name = db.name().to_string();
            if !repo_filter.is_empty() && !repo_filter.iter().any(|r| r == &name) {
                continue;
            }
            let needles = vec![needle.clone()];
            let matches = match db.search(needles.iter().map(|s| s.as_str())) {
                Ok(m) => m,
                Err(_) => continue,
            };
            for pkg in matches.iter() {
                let installed_version = installed.get(pkg.name()).cloned();
                out.push(sync_to_package(pkg, name.clone(), installed_version));
            }
        }
        Ok(out)
    }

    pub fn find_sync(&self, name: &str, repo_filter: &[String]) -> Result<Option<Package>> {
        let guard = self.handle.borrow();
        let installed = build_installed_index(&guard);
        for db in guard.syncdbs() {
            let repo = db.name().to_string();
            if !repo_filter.is_empty() && !repo_filter.iter().any(|r| r == &repo) {
                continue;
            }
            if let Ok(pkg) = db.pkg(name) {
                let installed_version = installed.get(pkg.name()).cloned();
                return Ok(Some(sync_to_package(pkg, repo, installed_version)));
            }
        }
        Ok(None)
    }

    pub fn find_local(&self, name: &str) -> Result<Option<Package>> {
        let guard = self.handle.borrow();
        match guard.localdb().pkg(name) {
            Ok(pkg) => Ok(Some(local_to_package(pkg))),
            Err(_) => Ok(None),
        }
    }

    pub fn upgrades(&self) -> Result<Vec<Package>> {
        let guard = self.handle.borrow();
        let mut out: Vec<Package> = Vec::new();
        let local_pkgs: Vec<(String, String)> = guard
            .localdb()
            .pkgs()
            .iter()
            .map(|p| (p.name().to_string(), p.version().to_string()))
            .collect();
        for (lname, lver) in local_pkgs {
            if let Some((repo, sync_pkg)) = find_in_sync(&guard, &lname) {
                if alpm::vercmp(sync_pkg.version().as_str(), lver.as_str())
                    == std::cmp::Ordering::Greater
                {
                    out.push(sync_to_package(sync_pkg, repo, Some(lver)));
                }
            }
        }
        Ok(out)
    }
}

fn build_installed_index(alpm: &Alpm) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for pkg in alpm.localdb().pkgs() {
        map.insert(pkg.name().to_string(), pkg.version().to_string());
    }
    map
}

fn init_alpm(conf: &pacmanconf::Config) -> Result<Alpm> {
    let alpm = Alpm::new(&*conf.root_dir, &*conf.db_path).context("Alpm::new")?;
    for repo in &conf.repos {
        // `register_syncdb` returns `&Db` — pacman has already populated the
        // mirrorlist in /etc/pacman.d, so we don't need to call `add_server`
        // ourselves to search the cached sync DBs.
        let _ = alpm
            .register_syncdb(repo.name.clone(), SigLevel::USE_DEFAULT)
            .with_context(|| format!("register syncdb {}", repo.name))?;
    }
    Ok(alpm)
}

fn find_in_sync<'a>(alpm: &'a Alpm, name: &str) -> Option<(String, &'a alpm::Pkg)> {
    for db in alpm.syncdbs() {
        if let Ok(pkg) = db.pkg(name) {
            return Some((db.name().to_string(), pkg));
        }
    }
    None
}

fn local_to_package(pkg: &alpm::Package) -> Package {
    Package {
        name: pkg.name().to_string(),
        version: pkg.version().to_string(),
        source: detect_source(pkg.db().map(|d| d.name().to_string()).as_deref()),
        description: pkg.desc().unwrap_or_default().to_string(),
        installed: true,
        installed_version: Some(pkg.version().to_string()),
        installed_size: Some(pkg.isize() as u64),
        download_size: None,
        maintainer: pkg.packager().map(|s| s.to_string()),
        url: pkg.url().map(|s| s.to_string()),
        licenses: pkg.licenses().iter().map(|s| s.to_string()).collect(),
        depends: pkg.depends().iter().map(|d| d.to_string()).collect(),
        make_depends: Vec::new(),
        opt_depends: pkg.optdepends().iter().map(|d| d.to_string()).collect(),
        provides: pkg.provides().iter().map(|d| d.to_string()).collect(),
        conflicts: pkg.conflicts().iter().map(|d| d.to_string()).collect(),
        votes: None,
        popularity: None,
        out_of_date: None,
        last_modified: None,
    }
}

fn sync_to_package(
    pkg: &alpm::Pkg,
    repo: String,
    installed_version: Option<String>,
) -> Package {
    let installed = installed_version.is_some();
    Package {
        name: pkg.name().to_string(),
        version: pkg.version().to_string(),
        source: Source::Repo(repo),
        description: pkg.desc().unwrap_or_default().to_string(),
        installed,
        installed_version,
        installed_size: Some(pkg.isize() as u64),
        download_size: Some(pkg.size() as u64),
        maintainer: pkg.packager().map(|s| s.to_string()),
        url: pkg.url().map(|s| s.to_string()),
        licenses: pkg.licenses().iter().map(|s| s.to_string()).collect(),
        depends: pkg.depends().iter().map(|d| d.to_string()).collect(),
        make_depends: Vec::new(),
        opt_depends: pkg.optdepends().iter().map(|d| d.to_string()).collect(),
        provides: pkg.provides().iter().map(|d| d.to_string()).collect(),
        conflicts: pkg.conflicts().iter().map(|d| d.to_string()).collect(),
        votes: None,
        popularity: None,
        out_of_date: None,
        last_modified: None,
    }
}

fn detect_source(db: Option<&str>) -> Source {
    match db {
        Some("local") | None => Source::Repo("local".into()),
        Some(other) => Source::Repo(other.into()),
    }
}
