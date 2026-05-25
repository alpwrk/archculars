use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use super::models::{Package, Source};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Install,
    Remove,
}

#[derive(Debug, Clone)]
pub enum LogLine {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

/// Choose an AUR helper binary that is available on the system.
pub fn detect_aur_helper() -> Option<&'static str> {
    for tool in ["paru", "yay"] {
        if which(tool) {
            return Some(tool);
        }
    }
    None
}

fn which(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|p| p.join(name).is_file())
        })
        .unwrap_or(false)
}

fn use_pkexec_for_root() -> bool {
    // If we're root already, no escalation needed; otherwise prefer pkexec
    // which gives a graphical prompt and stays out of our TUI.
    if unsafe { libc_geteuid() } == 0 {
        return false;
    }
    which("pkexec")
}

// libc::geteuid without bringing in the libc crate
unsafe fn libc_geteuid() -> u32 {
    extern "C" {
        fn geteuid() -> u32;
    }
    geteuid()
}

/// Build the command we'd run for an Install/Remove on `pkg`. Streams output
/// line-by-line over `tx`. Returns the spawned child so the caller can keep
/// the handle around for cancellation.
pub fn spawn(pkg: &Package, action: Action, tx: mpsc::Sender<LogLine>) -> Result<Child> {
    let mut cmd = build_command(pkg, action);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());
    let mut child = cmd.spawn().context("spawn package manager")?;

    if let Some(stdout) = child.stdout.take() {
        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_out.send(LogLine::Stdout(line)).await;
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let tx_err = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_err.send(LogLine::Stderr(line)).await;
            }
        });
    }
    Ok(child)
}

/// Drive a spawned child to completion and emit the final Exit line.
pub async fn wait(mut child: Child, tx: mpsc::Sender<LogLine>) -> Result<i32> {
    let status = child.wait().await.context("wait pkg-manager exit")?;
    let code = status.code().unwrap_or(-1);
    let _ = tx.send(LogLine::Exit(code)).await;
    Ok(code)
}

fn build_command(pkg: &Package, action: Action) -> Command {
    let is_aur = matches!(pkg.source, Source::Aur);
    let helper = if is_aur {
        detect_aur_helper()
    } else {
        None
    };

    // AUR install via helper (runs makepkg as user, then sudo for pacman -U).
    if matches!(action, Action::Install) && is_aur {
        if let Some(h) = helper {
            let mut cmd = Command::new(h);
            cmd.arg("-S").arg("--noconfirm").arg(&pkg.name);
            return cmd;
        }
    }

    // Repo install / any remove → pacman, possibly via pkexec.
    let mut cmd = if use_pkexec_for_root() {
        let mut c = Command::new("pkexec");
        c.arg("pacman");
        c
    } else {
        Command::new("pacman")
    };

    match action {
        Action::Install => {
            cmd.arg("-S").arg("--noconfirm").arg(&pkg.name);
        }
        Action::Remove => {
            cmd.arg("-Rns").arg("--noconfirm").arg(&pkg.name);
        }
    }
    cmd
}

/// One-shot helper to fetch a freshly-rendered changelog for a sync package.
pub async fn changelog(pkg_name: &str) -> Result<Option<String>> {
    let output = Command::new("pacman")
        .arg("-Qc")
        .arg(pkg_name)
        .output()
        .await
        .context("pacman -Qc")?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}
