//! Smoke test for the non-TUI backends. Run with `cargo run --example smoke`.

use archculars::core::alpm::AlpmBackend;
use archculars::core::aur::AurClient;
use archculars::core::cache::AurCache;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    println!("─── ALPM ────────────────────────────");
    let alpm = AlpmBackend::new()?;
    let installed = alpm.installed()?;
    println!("Installed packages: {}", installed.len());
    println!("Repos:              {:?}", alpm.repo_names());

    println!("\nSearching 'linux' in sync DBs:");
    let hits = alpm.search_sync("linux", &[])?;
    for p in hits.iter().take(5) {
        let inst = if p.installed { "✓" } else { "—" };
        println!(
            "  {} {:<30}  {:<22} [{}]",
            inst, p.name, p.version, p.source_label()
        );
    }
    println!("  … {} more", hits.len().saturating_sub(5));

    println!("\nUpgrades:");
    let ups = alpm.upgrades()?;
    if ups.is_empty() {
        println!("  System is up to date");
    } else {
        for u in ups.iter().take(5) {
            println!(
                "  {} : {} → {}",
                u.name,
                u.installed_version.as_deref().unwrap_or("?"),
                u.version
            );
        }
    }

    println!("\n─── AUR ─────────────────────────────");
    let cache = Arc::new(AurCache::new());
    let aur = AurClient::new(cache);
    let hits = aur.search("paru").await?;
    println!("AUR search 'paru': {} hits", hits.len());
    for p in hits.iter().take(3) {
        println!(
            "  {:<30}  {:<20}  votes {}  pop {:.2}",
            p.name,
            p.version,
            p.votes.unwrap_or(0),
            p.popularity.unwrap_or(0.0)
        );
    }

    Ok(())
}
