pub mod install;

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use futures::StreamExt;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, TableState};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::cli::Args;
use crate::core::alpm::AlpmBackend;
use crate::core::aur::{dedup_against_repos, AurClient};
use crate::core::cache::AurCache;
use crate::core::deps::build_tree;
use crate::core::models::{DepNode, Filter, Package};
use crate::core::news::NewsItem;
use crate::core::orphans;
use crate::core::pacman::{self, Action, LogLine};
use crate::theme;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Search,
    Table,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    None,
    Install,
    Deps,
    PkgBuild,
    Updates,
    News,
    Stats,
}

pub struct App {
    args: Args,
    alpm: Arc<AlpmBackend>,
    aur: AurClient,

    focus: Focus,
    modal: Modal,
    filter: Filter,

    query: String,
    last_searched: String,
    search_loading: bool,
    search_task: Option<JoinHandle<()>>,

    aur_results: Vec<Package>,
    repo_results: Vec<Package>,
    visible: Vec<Package>,
    table_state: TableState,

    toast: Option<String>,

    install: Option<install::InstallState>,
    install_log_rx: Option<mpsc::Receiver<LogLine>>,

    deps_root: Option<DepNode>,
    deps_scroll: u16,

    pkgbuild_name: Option<String>,
    pkgbuild_body: Option<String>,
    pkgbuild_scroll: u16,
    pkgbuild_rx: Option<oneshot::Receiver<String>>,

    updates: Vec<Package>,
    updates_scroll: u16,
    updates_loading: bool,
    updates_rx: Option<oneshot::Receiver<Vec<Package>>>,

    news: Vec<NewsItem>,
    news_scroll: u16,
    news_loading: bool,
    news_rx: Option<oneshot::Receiver<Vec<NewsItem>>>,

    largest: Vec<Package>,
    orphans: Vec<Package>,
    stats_scroll: u16,

    pending_aur_rx: Option<mpsc::Receiver<Vec<Package>>>,
}

impl App {
    pub fn new(args: Args, alpm: Arc<AlpmBackend>, aur: AurClient) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            args,
            alpm,
            aur,
            focus: Focus::Table,
            modal: Modal::None,
            filter: Filter::All,
            query: String::new(),
            last_searched: String::new(),
            search_loading: false,
            search_task: None,
            aur_results: Vec::new(),
            repo_results: Vec::new(),
            visible: Vec::new(),
            table_state,
            toast: None,
            install: None,
            install_log_rx: None,
            deps_root: None,
            deps_scroll: 0,
            pkgbuild_name: None,
            pkgbuild_body: None,
            pkgbuild_scroll: 0,
            pkgbuild_rx: None,
            updates: Vec::new(),
            updates_scroll: 0,
            updates_loading: false,
            updates_rx: None,
            news: Vec::new(),
            news_scroll: 0,
            news_loading: false,
            news_rx: None,
            largest: Vec::new(),
            orphans: Vec::new(),
            stats_scroll: 0,
            pending_aur_rx: None,
        }
    }

    pub fn selected(&self) -> Option<&Package> {
        self.table_state
            .selected()
            .and_then(|i| self.visible.get(i))
    }

    fn refresh_visible(&mut self) {
        let mut merged: Vec<Package> = self.repo_results.clone();
        merged.extend(self.aur_results.iter().cloned());
        merged.sort_by(|a, b| ranking(a).cmp(&ranking(b)).then(a.name.cmp(&b.name)));
        merged.retain(|p| self.filter.matches(p));
        self.visible = merged;
        let last = self.visible.len().saturating_sub(1);
        match self.table_state.selected() {
            Some(i) if i > last => {
                self.table_state.select(if self.visible.is_empty() {
                    None
                } else {
                    Some(last)
                });
            }
            None if !self.visible.is_empty() => {
                self.table_state.select(Some(0));
            }
            _ => {}
        }
    }

    fn enrich_with_installed(&mut self) {
        if let Ok(idx) = self.alpm.installed_index() {
            for p in self
                .repo_results
                .iter_mut()
                .chain(self.aur_results.iter_mut())
            {
                if let Some(v) = idx.get(&p.name) {
                    p.installed = true;
                    p.installed_version = Some(v.clone());
                }
            }
        }
    }
}

fn ranking(p: &Package) -> u8 {
    match (p.is_aur(), p.installed) {
        (false, true) => 0,
        (false, false) => 1,
        (true, true) => 2,
        (true, false) => 3,
    }
}

pub async fn run(terminal: &mut DefaultTerminal, args: Args) -> Result<()> {
    let cache = Arc::new(AurCache::new());
    let alpm = Arc::new(AlpmBackend::new()?);
    let aur = AurClient::new(cache);

    let mut app = App::new(args.clone(), alpm.clone(), aur);

    // Preload installed packages so the empty/installed view is non-empty.
    let installed = alpm.installed().unwrap_or_default();
    app.repo_results = installed;
    app.enrich_with_installed();

    if args.installed {
        app.filter = Filter::Installed;
    }
    if args.upgrades {
        app.filter = Filter::Upgrades;
        app.modal = Modal::Updates;
        spawn_load_upgrades(&mut app);
    }
    if let Some(q) = &args.query {
        app.query = q.clone();
        trigger_search(&mut app);
    }
    app.refresh_visible();

    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(150));

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        tokio::select! {
            biased;

            ev = events.next() => {
                match ev {
                    Some(Ok(event)) => {
                        if should_quit(&event) {
                            return Ok(());
                        }
                        handle_event(&mut app, event).await;
                    }
                    Some(Err(_)) | None => return Ok(()),
                }
            }

            _ = ticker.tick() => {
                drain_async_channels(&mut app);
            }
        }
    }
}

fn should_quit(event: &Event) -> bool {
    if let Event::Key(k) = event {
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('q') {
            return true;
        }
    }
    false
}

async fn handle_event(app: &mut App, event: Event) {
    app.toast = None;
    match event {
        Event::Key(k) if k.kind == KeyEventKind::Press => handle_key(app, k),
        Event::Mouse(m) => handle_mouse(app, m),
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    if app.modal != Modal::None {
        match key.code {
            KeyCode::Esc => {
                close_modal(app);
                return;
            }
            KeyCode::PageDown => bump_modal_scroll(app, 5),
            KeyCode::PageUp => bump_modal_scroll(app, -5),
            KeyCode::Down | KeyCode::Char('j') => bump_modal_scroll(app, 1),
            KeyCode::Up | KeyCode::Char('k') => bump_modal_scroll(app, -1),
            _ => {}
        }
        return;
    }

    if app.focus == Focus::Search {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.focus = Focus::Table;
                if key.code == KeyCode::Enter {
                    trigger_search(app);
                }
            }
            KeyCode::Backspace => {
                app.query.pop();
                trigger_search(app);
            }
            KeyCode::Char(c) => {
                app.query.push(c);
                trigger_search(app);
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('/') => app.focus = Focus::Search,
        KeyCode::Char('q') if app.query.is_empty() => std::process::exit(0),
        KeyCode::Down | KeyCode::Char('j') => move_selection(app, 1),
        KeyCode::Up | KeyCode::Char('k') => move_selection(app, -1),
        KeyCode::PageDown => move_selection(app, 10),
        KeyCode::PageUp => move_selection(app, -10),
        KeyCode::Home if !app.visible.is_empty() => app.table_state.select(Some(0)),
        KeyCode::End if !app.visible.is_empty() => {
            app.table_state.select(Some(app.visible.len() - 1))
        }
        KeyCode::Char('f') => cycle_filter(app),
        KeyCode::Char('r') => {
            app.aur.clear_cache();
            app.last_searched.clear();
            app.toast = Some("Reloaded".into());
            trigger_search(app);
        }
        KeyCode::Char('d') => open_deps(app),
        KeyCode::Char('p') => open_pkgbuild(app),
        KeyCode::Char('u') => {
            app.modal = Modal::Updates;
            spawn_load_upgrades(app);
        }
        KeyCode::Char('n') => {
            app.modal = Modal::News;
            spawn_load_news(app);
        }
        KeyCode::Char('s') => {
            app.modal = Modal::Stats;
            load_stats(app);
        }
        KeyCode::Enter => spawn_install(app, infer_action(app)),
        _ => {}
    }
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    if app.modal != Modal::None {
        match mouse.kind {
            MouseEventKind::ScrollDown => bump_modal_scroll(app, 2),
            MouseEventKind::ScrollUp => bump_modal_scroll(app, -2),
            _ => {}
        }
        return;
    }
    match mouse.kind {
        MouseEventKind::ScrollDown => move_selection(app, 3),
        MouseEventKind::ScrollUp => move_selection(app, -3),
        _ => {}
    }
}

fn move_selection(app: &mut App, delta: i32) {
    if app.visible.is_empty() {
        return;
    }
    let current = app.table_state.selected().unwrap_or(0) as i32;
    let next = (current + delta).clamp(0, app.visible.len() as i32 - 1);
    app.table_state.select(Some(next as usize));
}

fn cycle_filter(app: &mut App) {
    app.filter = match app.filter {
        Filter::All => Filter::Installed,
        Filter::Installed => Filter::Repos,
        Filter::Repos => Filter::Aur,
        Filter::Aur => Filter::Upgrades,
        Filter::Upgrades => Filter::All,
    };
    app.toast = Some(format!("Filter: {}", app.filter.label()));
    app.refresh_visible();
}

fn trigger_search(app: &mut App) {
    if let Some(h) = app.search_task.take() {
        h.abort();
    }
    let query = app.query.clone();
    if query == app.last_searched && !query.is_empty() {
        return;
    }
    app.last_searched = query.clone();

    let repo_filter = app.args.repos.clone();
    if query.is_empty() {
        app.repo_results = app.alpm.installed().unwrap_or_default();
        app.aur_results.clear();
        app.enrich_with_installed();
        app.refresh_visible();
        app.search_loading = false;
        return;
    }

    app.repo_results = app
        .alpm
        .search_sync(&query, &repo_filter)
        .unwrap_or_default();

    // AUR search async
    let aur = app.aur.clone();
    let repo_names: HashSet<String> = app
        .repo_results
        .iter()
        .map(|p| p.name.clone())
        .collect();
    let (tx, rx) = mpsc::channel::<Vec<Package>>(1);
    let q = query.clone();
    app.search_task = Some(tokio::spawn(async move {
        let aur_pkgs = aur.search(&q).await.unwrap_or_default();
        let pruned = dedup_against_repos(aur_pkgs, &repo_names);
        let _ = tx.send(pruned).await;
    }));
    app.pending_aur_rx = Some(rx);
    app.search_loading = true;
    app.enrich_with_installed();
    app.refresh_visible();
}

fn close_modal(app: &mut App) {
    app.modal = Modal::None;
    app.deps_scroll = 0;
    app.pkgbuild_scroll = 0;
    app.updates_scroll = 0;
    app.news_scroll = 0;
    app.stats_scroll = 0;
}

fn bump_modal_scroll(app: &mut App, delta: i32) {
    let target = match app.modal {
        Modal::Deps => &mut app.deps_scroll,
        Modal::PkgBuild => &mut app.pkgbuild_scroll,
        Modal::Updates => &mut app.updates_scroll,
        Modal::News => &mut app.news_scroll,
        Modal::Stats => &mut app.stats_scroll,
        Modal::Install => {
            if let Some(state) = app.install.as_mut() {
                let current = state.scroll as i32;
                state.scroll = (current + delta).max(0) as u16;
            }
            return;
        }
        _ => return,
    };
    let current = *target as i32;
    *target = (current + delta).max(0) as u16;
}

fn open_deps(app: &mut App) {
    let Some(pkg) = app.selected().cloned() else { return; };
    let mut index: HashMap<String, Package> = HashMap::new();
    for p in &app.repo_results {
        index.insert(p.name.clone(), p.clone());
    }
    for p in &app.aur_results {
        index.insert(p.name.clone(), p.clone());
    }
    if let Ok(installed) = app.alpm.installed() {
        for p in installed {
            index.entry(p.name.clone()).or_insert(p);
        }
    }
    app.deps_root = Some(build_tree(&pkg, &index, 4));
    app.modal = Modal::Deps;
}

fn open_pkgbuild(app: &mut App) {
    let Some(pkg) = app.selected().cloned() else { return; };
    if !pkg.is_aur() {
        app.toast = Some("PKGBUILD viewer is AUR-only".into());
        return;
    }
    app.pkgbuild_name = Some(pkg.name.clone());
    app.pkgbuild_body = None;
    app.modal = Modal::PkgBuild;
    let aur = app.aur.clone();
    let name = pkg.name.clone();
    let (tx, rx) = oneshot::channel::<String>();
    app.pkgbuild_rx = Some(rx);
    tokio::spawn(async move {
        if let Ok(body) = aur.pkgbuild(&name).await {
            let _ = tx.send(body);
        }
    });
}

fn infer_action(app: &App) -> Action {
    match app.selected() {
        Some(p) if p.installed => Action::Remove,
        _ => Action::Install,
    }
}

fn spawn_install(app: &mut App, action: Action) {
    let Some(pkg) = app.selected().cloned() else { return; };
    let (tx, rx) = mpsc::channel::<LogLine>(64);
    let preview = describe_command(&pkg, action);
    let child = match pacman::spawn(&pkg, action, tx.clone()) {
        Ok(c) => c,
        Err(e) => {
            app.toast = Some(format!("Failed to start command: {e}"));
            return;
        }
    };
    let waiter = tokio::spawn(async move {
        let _ = pacman::wait(child, tx).await;
    });
    app.install = Some(install::InstallState {
        package_name: pkg.name.clone(),
        action,
        log: Vec::new(),
        scroll: 0,
        command_preview: preview,
        waiter: Some(waiter),
    });
    app.install_log_rx = Some(rx);
    app.modal = Modal::Install;
}

fn describe_command(pkg: &Package, action: Action) -> String {
    let mut parts: Vec<String> = Vec::new();
    if pkg.is_aur() && matches!(action, Action::Install) {
        let helper = pacman::detect_aur_helper().unwrap_or("yay");
        parts.push(helper.into());
        parts.push("-S".into());
    } else {
        parts.push("pkexec pacman".into());
        parts.push(match action {
            Action::Install => "-S".into(),
            Action::Remove => "-Rns".into(),
        });
    }
    parts.push("--noconfirm".into());
    parts.push(pkg.name.clone());
    parts.join(" ")
}

fn drain_async_channels(app: &mut App) {
    if let Some(rx) = app.install_log_rx.as_mut() {
        while let Ok(line) = rx.try_recv() {
            if let Some(state) = app.install.as_mut() {
                state.log.push(line);
                let max_scroll = state.log.len().saturating_sub(8) as u16;
                if state.scroll < max_scroll.saturating_sub(2) {
                    // user scrolled up; leave alone
                } else {
                    state.scroll = max_scroll;
                }
            }
        }
    }

    let mut drained_aur = false;
    if let Some(rx) = app.pending_aur_rx.as_mut() {
        if let Ok(pkgs) = rx.try_recv() {
            app.aur_results = pkgs;
            app.search_loading = false;
            drained_aur = true;
        }
    }
    if drained_aur {
        app.pending_aur_rx = None;
        app.enrich_with_installed();
        app.refresh_visible();
    }

    let mut got_pkgbuild = false;
    if let Some(rx) = app.pkgbuild_rx.as_mut() {
        if let Ok(body) = rx.try_recv() {
            app.pkgbuild_body = Some(body);
            got_pkgbuild = true;
        }
    }
    if got_pkgbuild {
        app.pkgbuild_rx = None;
    }

    let mut got_updates = false;
    if let Some(rx) = app.updates_rx.as_mut() {
        if let Ok(list) = rx.try_recv() {
            app.updates = list;
            app.updates_loading = false;
            got_updates = true;
        }
    }
    if got_updates {
        app.updates_rx = None;
    }

    let mut got_news = false;
    if let Some(rx) = app.news_rx.as_mut() {
        if let Ok(items) = rx.try_recv() {
            app.news = items;
            app.news_loading = false;
            got_news = true;
        }
    }
    if got_news {
        app.news_rx = None;
    }
}

fn spawn_load_upgrades(app: &mut App) {
    // alpm is !Send, so we just run synchronously on the main thread —
    // localdb scans typically finish in <100ms on a fully populated install.
    app.updates_loading = true;
    app.updates = app.alpm.upgrades().unwrap_or_default();
    app.updates_loading = false;
    app.updates_rx = None;
}

fn spawn_load_news(app: &mut App) {
    let (tx, rx) = oneshot::channel::<Vec<NewsItem>>();
    app.news_rx = Some(rx);
    app.news_loading = true;
    tokio::spawn(async move {
        let list = crate::core::news::fetch(20).await.unwrap_or_default();
        let _ = tx.send(list);
    });
}

fn load_stats(app: &mut App) {
    if let Ok(installed) = app.alpm.installed() {
        app.largest = orphans::largest(&installed, 30);
        app.orphans = orphans::detect(&installed);
    }
}

fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_title(f, outer[0]);
    ui::search::render(f, outer[1], &app.query, app.focus, app.search_loading);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(outer[2]);
    ui::table::render(
        f,
        main[0],
        &app.visible,
        &mut app.table_state,
        app.focus,
        app.filter,
    );
    ui::info::render(f, main[1], app.selected());

    let toast_ref = app.toast.as_deref();
    ui::footer::render(f, outer[3], toast_ref);

    match app.modal {
        Modal::Install => {
            if let Some(state) = &app.install {
                ui::install_modal::render(f, area, state);
            }
        }
        Modal::Deps => {
            if let Some(root) = &app.deps_root {
                ui::deps_modal::render(f, area, root, app.deps_scroll);
            }
        }
        Modal::PkgBuild => {
            let name = app.pkgbuild_name.as_deref().unwrap_or("");
            ui::pkgbuild_modal::render(
                f,
                area,
                name,
                app.pkgbuild_body.as_deref(),
                app.pkgbuild_scroll,
            );
        }
        Modal::Updates => {
            ui::updates_modal::render(f, area, &app.updates, app.updates_scroll, app.updates_loading);
        }
        Modal::News => {
            ui::news_modal::render(f, area, &app.news, app.news_scroll, app.news_loading);
        }
        Modal::Stats => {
            ui::stats_modal::render(f, area, &app.largest, &app.orphans, app.stats_scroll);
        }
        Modal::None => {}
    }
}

fn render_title(f: &mut ratatui::Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " archculars ",
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "modern TUI for Arch + AUR",
            Style::default().fg(theme::MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
