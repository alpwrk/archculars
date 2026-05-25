use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::models::Package;

const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60); // 1 hour

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    inserted_at: u64,
    packages: Vec<Package>,
}

#[derive(Debug)]
pub struct AurCache {
    inner: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,
    file: Option<PathBuf>,
}

impl AurCache {
    pub fn new() -> Self {
        let file = dirs::cache_dir().map(|p| p.join("archculars").join("aur.json"));
        let inner = file
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<HashMap<String, CacheEntry>>(&s).ok())
            .unwrap_or_default();
        Self {
            inner: Mutex::new(inner),
            ttl: DEFAULT_TTL,
            file,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<Package>> {
        let guard = self.inner.lock().ok()?;
        let entry = guard.get(key)?;
        if now_seconds().saturating_sub(entry.inserted_at) < self.ttl.as_secs() {
            Some(entry.packages.clone())
        } else {
            None
        }
    }

    pub fn put(&self, key: String, packages: Vec<Package>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(
                key,
                CacheEntry {
                    inserted_at: now_seconds(),
                    packages,
                },
            );
            // Best-effort persistence; ignore errors.
            if let Some(file) = &self.file {
                let _ = persist(&*guard, file);
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.inner.lock() {
            g.clear();
            if let Some(file) = &self.file {
                let _ = std::fs::remove_file(file);
            }
        }
    }
}

impl Default for AurCache {
    fn default() -> Self {
        Self::new()
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn persist(map: &HashMap<String, CacheEntry>, file: &PathBuf) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).context("create cache dir")?;
    }
    let json = serde_json::to_string(map).context("serialize cache")?;
    std::fs::write(file, json).context("write cache")?;
    Ok(())
}
