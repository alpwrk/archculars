# archculars

A modern terminal UI for browsing and managing Arch Linux + AUR packages.
Built as a feature-rich successor to [pacseek](https://github.com/moson-mo/pacseek),
written in Rust on top of [ratatui](https://ratatui.rs/) and
[libalpm](https://man.archlinux.org/man/libalpm.3.en).

```
 ─────────────────────────────────────────────────────────────
   archculars   moderne TUI für Arch + AUR
 ─────────────────────────────────────────────────────────────
 / Suche [enter] linux
 ─────────────────────── Pakete · All · 47 Treffer ─────────
 ▶ linux            6.12.1.arch1-1   core    142 MB  ✓
   linux-zen        6.12.1.zen1-1    extra   148 MB  ✓
   linux-cachyos    6.12.1-1         AUR     151 MB  —
   linux-firmware   20250108.1-1     core     1.4 GB ✓
 ───────────────────────────────────────────────────────────
 / Suche  ↑↓ Nav  Enter Install/Remove  d Deps  p PKGBUILD …
```

## Features

- **Async parallel search** in repos (via libalpm) and AUR (via raur), with a 1h
  on-disk cache so repeated queries are instant.
- **Modern UI** with mouse support, red accent theme, smooth scrolling, focused
  borders, and a responsive split-pane layout.
- **Rich details panel** showing version, source, install size, license,
  maintainer, AUR votes/popularity, out-of-date flag and recent dependencies.
- **Dependency tree** modal (`d`) that walks `depends`, `makedepends` and
  `optdepends` up to 4 levels deep.
- **PKGBUILD viewer** modal (`p`) with syntect syntax highlighting — fetched
  live from the AUR.
- **Upgrade screen** (`u`) using `alpm::vercmp` to find packages whose sync
  version is newer than what is installed.
- **Stats view** (`s`) with the 30 largest installed packages and a heuristic
  orphan list.
- **Arch news feed** (`n`) via the official RSS.
- **Install / remove** through `pkexec pacman -S/-Rns` for repo packages or
  via the detected AUR helper (`paru` ⟶ `yay`) for AUR builds, with live
  output streamed into a modal.
- **Filter cycling** (`f`): all → installed → repos → aur → upgrades.
- **CLI-args compatible with pacseek**: `-r core,extra`, `-u`, `-i`, positional
  search term.

## Build

Requires the `libalpm` headers (ship with `pacman`) and a recent Rust toolchain.

```bash
cd archculars
cargo build --release
./target/release/archculars
```

The release binary is a single ~9 MB self-contained executable.

## Keybindings

| Key | Action |
|---|---|
| `/` | Focus the search bar |
| `Esc` | Leave the search bar / close the active modal |
| `Enter` | Install (or remove if already installed) the selected package |
| `↑` `↓` `j` `k` | Move selection / scroll modal |
| `PageUp` `PageDown` | Page through the table or modal |
| `Home` `End` | Jump to first/last row |
| `f` | Cycle filter (All → Installed → Repos → AUR → Upgrades) |
| `d` | Open dependency tree for the selected package |
| `p` | Open PKGBUILD viewer (AUR only) |
| `u` | Open upgrades screen |
| `n` | Open Arch news feed |
| `s` | Open stats (largest packages + orphans) |
| `r` | Force-refresh the current view (invalidate search cache) |
| `Ctrl+Q` | Quit from anywhere |
| `q` | Quit when the search bar is empty |

## Stack

- [`ratatui`](https://ratatui.rs/) + [`crossterm`](https://github.com/crossterm-rs/crossterm) — TUI rendering and input
- [`alpm`](https://crates.io/crates/alpm) + [`pacmanconf`](https://crates.io/crates/pacmanconf) — local + sync database access
- [`raur`](https://crates.io/crates/raur) — AUR RPC v5 client
- [`tokio`](https://tokio.rs/) (current-thread) — async runtime
- [`reqwest`](https://crates.io/crates/reqwest) — HTTPS for PKGBUILDs and news
- [`feed-rs`](https://crates.io/crates/feed-rs) — Arch news feed parser
- [`syntect`](https://crates.io/crates/syntect) — PKGBUILD syntax highlighting
- [`clap`](https://crates.io/crates/clap) — argument parsing

## License

MIT
