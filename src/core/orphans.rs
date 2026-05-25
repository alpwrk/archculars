use std::collections::HashSet;

use super::deps::reverse_deps;
use super::models::Package;

/// Heuristic orphan detection: packages that are installed-as-dependency
/// (not explicitly requested) and have nothing depending on them.
pub fn detect(installed: &[Package]) -> Vec<Package> {
    // We don't have install-reason in our model yet, so fall back to a pure
    // reverse-deps scan. Packages that nothing else depends on are candidates.
    let names: HashSet<&str> = installed.iter().map(|p| p.name.as_str()).collect();
    installed
        .iter()
        .filter(|p| {
            let rd = reverse_deps(&p.name, installed);
            // Skip if anything in our installed set depends on it.
            !rd.iter().any(|r| names.contains(r.as_str()))
        })
        .cloned()
        .collect()
}

/// Top-N installed packages by `installed_size`.
pub fn largest(installed: &[Package], n: usize) -> Vec<Package> {
    let mut sorted: Vec<Package> = installed.to_vec();
    sorted.sort_by(|a, b| b.installed_size.unwrap_or(0).cmp(&a.installed_size.unwrap_or(0)));
    sorted.into_iter().take(n).collect()
}
