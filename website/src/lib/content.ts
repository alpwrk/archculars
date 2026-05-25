export interface Feature {
	title: string;
	short: string;
	body: string;
	icon: string;
}

export interface Keybinding {
	key: string;
	action: string;
}

export interface CliArg {
	flag: string;
	long: string;
	description: string;
}

export interface ArchModule {
	path: string;
	purpose: string;
}

export interface StackEntry {
	name: string;
	role: string;
	url: string;
}

export const META = {
	name: 'archculars',
	tagline: 'Modern TUI for Arch Linux + AUR package management',
	taglineDe: 'Moderne TUI für Arch + AUR',
	version: '0.1.0',
	license: 'MIT',
	repo: 'https://github.com/alpwrk/archculars',
	binarySize: '~9 MB',
	language: 'Rust 2021',
	author: 'Alp <sinn1os@proton.me>'
};

export const FEATURES: Feature[] = [
	{
		title: 'Async parallel search',
		short: 'Repos + AUR gleichzeitig',
		body: 'Sucht parallel in den Sync-DBs (über libalpm) und in der AUR (über raur). Eine On-Disk-Cache mit 1h TTL macht wiederholte Anfragen sofort verfügbar.',
		icon: 'search'
	},
	{
		title: 'Modern UI',
		short: 'Maus, Akzente, Borders',
		body: 'Mauseingabe, roter Akzent-Theme, weiches Scrolling, fokussierte Borders und ein responsives Split-Pane-Layout — alles im Terminal.',
		icon: 'sparkle'
	},
	{
		title: 'Rich details panel',
		short: 'Alles auf einen Blick',
		body: 'Zeigt Version, Quelle, Install-Größe, Lizenz, Maintainer, AUR Votes/Popularity, Out-of-Date Flag und aktuelle Dependencies.',
		icon: 'panel'
	},
	{
		title: 'Dependency tree',
		short: 'Modal mit »d«',
		body: 'Geht depends, makedepends und optdepends bis zu 4 Ebenen tief — mit Zykluserkennung. Geöffnet per d-Taste.',
		icon: 'tree'
	},
	{
		title: 'PKGBUILD viewer',
		short: 'Modal mit »p«',
		body: 'Lädt den PKGBUILD live von aur.archlinux.org und rendert ihn mit syntect-Syntax-Highlighting.',
		icon: 'code'
	},
	{
		title: 'Upgrade screen',
		short: 'Modal mit »u«',
		body: 'Nutzt alpm::vercmp um Pakete zu finden, deren Sync-Version neuer ist als die installierte.',
		icon: 'upgrade'
	},
	{
		title: 'Stats view',
		short: 'Modal mit »s«',
		body: 'Top-30 größte installierte Pakete und eine heuristische Orphan-Liste über Reverse-Deps.',
		icon: 'stats'
	},
	{
		title: 'Arch news feed',
		short: 'Modal mit »n«',
		body: 'Lädt das offizielle archlinux.org RSS-Feed und zeigt die letzten 20 Nachrichten direkt im TUI.',
		icon: 'news'
	},
	{
		title: 'Install / remove',
		short: 'Native pacman',
		body: 'Pkexec pacman -S/-Rns für Repo-Pakete, paru oder yay für AUR. Live-Output wird in ein Modal gestreamt.',
		icon: 'install'
	},
	{
		title: 'Filter cycling',
		short: 'f drücken',
		body: 'Zyklus zwischen all → installed → repos → aur → upgrades. Schnellster Weg zwischen den Views.',
		icon: 'filter'
	}
];

export const KEYBINDINGS: Keybinding[] = [
	{ key: '/', action: 'Focus the search bar' },
	{ key: 'Esc', action: 'Leave search / close active modal' },
	{ key: 'Enter', action: 'Install (or remove) selected package' },
	{ key: '↑ ↓ / j k', action: 'Move selection or scroll modal' },
	{ key: 'PageUp / PageDown', action: 'Page through table or modal' },
	{ key: 'Home / End', action: 'Jump to first / last row' },
	{ key: 'f', action: 'Cycle filter (All → Installed → Repos → AUR → Upgrades)' },
	{ key: 'd', action: 'Open dependency tree for selected package' },
	{ key: 'p', action: 'Open PKGBUILD viewer (AUR only)' },
	{ key: 'u', action: 'Open upgrades screen' },
	{ key: 'n', action: 'Open Arch news feed' },
	{ key: 's', action: 'Open stats (largest packages + orphans)' },
	{ key: 'r', action: 'Force-refresh the current view (invalidate cache)' },
	{ key: 'Ctrl+Q', action: 'Quit from anywhere' },
	{ key: 'q', action: 'Quit when the search bar is empty' }
];

export const CLI_ARGS: CliArg[] = [
	{
		flag: '<query>',
		long: '',
		description: 'Optional search term that pre-fills the search bar on start.'
	},
	{
		flag: '-r',
		long: '--repos <names>',
		description: 'Limit repo searches to a comma-separated list (e.g. core,extra).'
	},
	{
		flag: '-u',
		long: '--upgrades',
		description: 'Open the upgrades view immediately on start.'
	},
	{
		flag: '-i',
		long: '--installed',
		description: 'Pre-select the "installed" filter on start.'
	}
];

export const ARCHITECTURE: ArchModule[] = [
	{ path: 'src/main.rs', purpose: 'Entry-point — sets up tracing, ratatui terminal, mouse capture and drives app::run.' },
	{ path: 'src/cli.rs', purpose: 'clap-derived CLI parsing — accepts a pacseek-compatible argument set.' },
	{ path: 'src/app.rs', purpose: 'Event loop, App state, render dispatcher, modal handling, async-channel draining.' },
	{ path: 'src/app/install.rs', purpose: 'InstallState — keeps log lines, scroll offset and the spawned waiter handle.' },
	{ path: 'src/core/alpm.rs', purpose: 'Wraps libalpm in a RefCell on the main thread — searches sync DBs, lists installed, finds upgrades via vercmp.' },
	{ path: 'src/core/aur.rs', purpose: 'raur RPC client with cache, dedup against repos, PKGBUILD fetch from aur.archlinux.org.' },
	{ path: 'src/core/cache.rs', purpose: 'Disk-persisted cache (~/.cache/archculars/aur.json) with 1h TTL for AUR queries.' },
	{ path: 'src/core/deps.rs', purpose: 'Dependency tree builder with cycle detection and reverse-dependency scanning.' },
	{ path: 'src/core/news.rs', purpose: 'feed-rs based parser for the archlinux.org news Atom feed.' },
	{ path: 'src/core/orphans.rs', purpose: 'Heuristic orphan detection + top-N largest packages by install size.' },
	{ path: 'src/core/pacman.rs', purpose: 'Spawns pacman / pkexec / paru / yay and streams stdout+stderr line-by-line over an mpsc channel.' },
	{ path: 'src/core/models.rs', purpose: 'Package, Source, Filter, DepNode, DepKind — the shared data model.' },
	{ path: 'src/ui/', purpose: 'Ratatui widgets: search bar, results table, info panel, deps/pkgbuild/install/updates/news/stats modals and the footer.' },
	{ path: 'src/theme.rs', purpose: 'Color palette: red accent, source-specific colors (AUR yellow, multilib blue, core/extra green).' }
];

export const STACK: StackEntry[] = [
	{ name: 'ratatui', role: 'TUI rendering', url: 'https://ratatui.rs/' },
	{ name: 'crossterm', role: 'Terminal input & mouse', url: 'https://github.com/crossterm-rs/crossterm' },
	{ name: 'alpm', role: 'libalpm bindings — local + sync DB', url: 'https://crates.io/crates/alpm' },
	{ name: 'pacmanconf', role: '/etc/pacman.conf parser', url: 'https://crates.io/crates/pacmanconf' },
	{ name: 'raur', role: 'AUR RPC v5 client', url: 'https://crates.io/crates/raur' },
	{ name: 'tokio', role: 'current-thread async runtime', url: 'https://tokio.rs/' },
	{ name: 'reqwest', role: 'HTTPS for PKGBUILDs & news', url: 'https://crates.io/crates/reqwest' },
	{ name: 'feed-rs', role: 'Atom/RSS parser for Arch news', url: 'https://crates.io/crates/feed-rs' },
	{ name: 'syntect', role: 'PKGBUILD syntax highlighting', url: 'https://crates.io/crates/syntect' },
	{ name: 'clap', role: 'CLI argument parsing', url: 'https://crates.io/crates/clap' }
];

export const TUI_PREVIEW = ` ─────────────────────────────────────────────────────────────
   archculars   moderne TUI für Arch + AUR
 ─────────────────────────────────────────────────────────────
 / Suche [enter] linux
 ─────────────────────── Pakete · All · 47 Treffer ─────────
 ▶ linux            6.12.1.arch1-1   core    142 MB  ✓
   linux-zen        6.12.1.zen1-1    extra   148 MB  ✓
   linux-cachyos    6.12.1-1         AUR     151 MB  —
   linux-firmware   20250108.1-1     core     1.4 GB ✓
 ───────────────────────────────────────────────────────────
 / Suche  ↑↓ Nav  Enter Install/Remove  d Deps  p PKGBUILD …`;
