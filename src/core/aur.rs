use anyhow::{Context, Result};
use raur::{Handle, Raur, SearchBy};
use std::collections::HashSet;
use std::sync::Arc;

use super::cache::AurCache;
use super::models::{Package, Source};

#[derive(Clone)]
pub struct AurClient {
    handle: Handle,
    cache: Arc<AurCache>,
}

impl AurClient {
    pub fn new(cache: Arc<AurCache>) -> Self {
        Self {
            handle: Handle::new(),
            cache,
        }
    }

    /// Search the AUR by name+description. Cached for 1h per query.
    pub async fn search(&self, query: &str) -> Result<Vec<Package>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let key = format!("search:{query}");
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }
        let results = self
            .handle
            .search_by(query, SearchBy::NameDesc)
            .await
            .context("AUR search request failed")?;
        let packages: Vec<Package> = results.into_iter().map(aur_to_package).collect();
        self.cache.put(key, packages.clone());
        Ok(packages)
    }

    /// Fetch full info for a list of package names (max ~150 per request).
    pub async fn info(&self, names: &[String]) -> Result<Vec<Package>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let key = format!(
            "info:{}",
            {
                let mut sorted: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
                sorted.sort();
                sorted.join(",")
            }
        );
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }
        let mut out: Vec<Package> = Vec::with_capacity(names.len());
        for chunk in names.chunks(150) {
            let res = self
                .handle
                .info(chunk)
                .await
                .context("AUR info request failed")?;
            out.extend(res.into_iter().map(aur_to_package));
        }
        self.cache.put(key, out.clone());
        Ok(out)
    }

    /// Drop all cached AUR responses so the next query hits the network.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Fetch a single PKGBUILD from the AUR git mirror.
    pub async fn pkgbuild(&self, name: &str) -> Result<String> {
        let url = format!("https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h={name}");
        let resp = reqwest::get(&url).await.context("fetch PKGBUILD")?;
        if !resp.status().is_success() {
            anyhow::bail!("PKGBUILD fetch returned {}", resp.status());
        }
        resp.text().await.context("read PKGBUILD body")
    }
}

fn aur_to_package(p: raur::Package) -> Package {
    Package {
        name: p.name,
        version: p.version,
        source: Source::Aur,
        description: p.description.unwrap_or_default(),
        installed: false,
        installed_version: None,
        installed_size: None,
        download_size: None,
        maintainer: p.maintainer,
        url: p.url,
        licenses: p.license,
        depends: p.depends,
        make_depends: p.make_depends,
        opt_depends: p.opt_depends,
        provides: p.provides,
        conflicts: p.conflicts,
        votes: Some(p.num_votes as i32),
        popularity: Some(p.popularity),
        out_of_date: p.out_of_date,
        last_modified: Some(p.last_modified),
    }
}

/// Deduplicate AUR results that already exist in the repo list (by name).
pub fn dedup_against_repos(aur: Vec<Package>, repo_names: &HashSet<String>) -> Vec<Package> {
    aur.into_iter()
        .filter(|p| !repo_names.contains(&p.name))
        .collect()
}
