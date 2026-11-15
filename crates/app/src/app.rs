//! Uygulama durumu, eylemler, ana dongu.

use crate::theme::{self, Palette};
use crate::widgets;
use egui::{Align2, Color32, Id, Rect, RichText, Sense, Stroke, pos2, vec2};
use iskandil_core::format as fmt;
use iskandil_core::scan::{ScanOptions, ScanSession};
use iskandil_core::settings::{Settings, ThemeMode};
use iskandil_core::sys::{self, DriveInfo};
use iskandil_core::treemap::{self, Tile, TreemapOptions};
use iskandil_core::{Category, NONE, Tree};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const APP_TITLE: &str = "İskandil — disk derinlik ölçer";
pub const SEARCH_ID: &str = "iskandil-search";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Start,
    Scanning,
    Workspace,
}

#[derive(Clone, Copy, Debug)]
pub enum UiAction {
    Select(u32),
    View(u32),
    Toggle(u32),
    OpenExplorer(u32),
    CopyPath(u32),
    AskDelete(u32),
}

pub struct RowsCache {
    pub generation: u64,
    pub expanded_rev: u64,
    pub rows: Vec<(u32, u16)>,
}

pub struct SearchCache {
    pub query: String,
    pub generation: u64,
    pub hits: Vec<u32>,
}

pub struct FolderStats {
    pub root: u32,
    pub generation: u64,
    pub types: Vec<(Category, u64, u64)>,
    pub top: Vec<u32>,
}

pub struct TilesCache {
    pub root: u32,
    pub generation: u64,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Warning,
    Error,
}

pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub at: Instant,
}

impl Toast {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ToastKind::Info,
            at: Instant::now(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ToastKind::Warning,
            at: Instant::now(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ToastKind::Error,
            at: Instant::now(),
        }
    }
}

pub enum Modal {
    Delete {
        id: u32,
        path: String,
        size: u64,
        is_dir: bool,
    },
    Error {
        title: String,
        body: String,
    },
}

pub struct ShotPlan {
    pub path: PathBuf,
    pub wait_for_scan: bool,
    pub start: Instant,
    pub requested: bool,
    pub done: Arc<AtomicBool>,
}

pub struct LaunchOptions {
    pub scan_path: Option<String>,
    pub shot: Option<PathBuf>,
    pub shot_now: bool,
    pub theme: Option<ThemeMode>,
    pub view: Option<String>,
    pub search: Option<String>,
    pub ask_delete_largest: bool,
}

pub struct IskandilApp {
    pub p: Palette,
    pub settings: Settings,
    pub drives: Vec<DriveInfo>,
    pub path_input: String,
    pub session: Option<ScanSession>,
    pub tree: Option<Tree>,
    pub view_root: u32,
    pub selected: Option<u32>,
    pub expanded: HashSet<u32>,
    pub expanded_rev: u64,
    pub rows_cache: Option<RowsCache>,
    pub search: String,
    pub search_cache: Option<SearchCache>,
    pub generation: u64,
    pub folder_stats: Option<FolderStats>,
    pub tiles_cache: Option<TilesCache>,
    pub toast: Option<Toast>,
    pub modal: Option<Modal>,
    pub scan_error: Option<String>,
    pub shot: Option<ShotPlan>,
    pub frame: u64,
    pub last_scan_path: Option<String>,
    pub pending_view: Option<String>,
    pub pending_search: Option<String>,
    pub pending_delete_largest: bool,
}

impl IskandilApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: LaunchOptions) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let settings = Settings::load();
        let mode = opts.theme.unwrap_or(settings.theme);
        theme::apply(&cc.egui_ctx, mode);
        let drives = sys::drives();
        let path_input = opts
            .scan_path
            .clone()
            .or_else(|| settings.last_path.clone())
            .unwrap_or_else(default_start_path);
        let shot = opts.shot.map(|path| ShotPlan {
            path,
            wait_for_scan: opts.scan_path.is_some() && !opts.shot_now,
            start: Instant::now(),
            requested: false,
            done: Arc::new(AtomicBool::new(false)),
        });
        let mut app = Self {
            p: theme::palette(mode),
            settings,
            drives,
            path_input,
            session: None,
            tree: None,
            view_root: 0,
            selected: None,
            expanded: HashSet::new(),
            expanded_rev: 0,
            rows_cache: None,
            search: String::new(),
            search_cache: None,
            generation: 0,
            folder_stats: None,
            tiles_cache: None,
            toast: None,
            modal: None,
            scan_error: None,
            shot,
            frame: 0,
            last_scan_path: None,
            pending_view: opts.view,
            pending_search: opts.search,
            pending_delete_largest: opts.ask_delete_largest,
        };
        if let Some(path) = opts.scan_path {
            app.start_scan(&cc.egui_ctx, &path);
        }
        app
    }

    pub fn mode(&self) -> Mode {
        if self.session.is_some() {
            Mode::Scanning
        } else if self.tree.is_some() {
            Mode::Workspace
        } else {
            Mode::Start
        }
    }

    pub fn node_path(&self, id: u32) -> String {
        self.tree
            .as_ref()
            .map(|t| t.path_of(id))
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------
    // Tarama
    // -----------------------------------------------------------------

    pub fn start_scan(&mut self, ctx: &egui::Context, input: &str) {
        self.scan_error = None;
        self.modal = None;
        if let Some(s) = &self.session {
            s.cancel();
        }
        self.session = None;
        let normalized = match iskandil_core::scan::normalize_display_path(input) {
            Ok(p) => p,
            Err(e) => {
                self.scan_error = Some(e);
                return;
            }
        };
        let volume_total = if iskandil_core::scan::is_drive_root(&normalized) {
            sys::volume_root(&normalized).and_then(|root| {
                self.drives
                    .iter()
                    .find(|d| d.root.eq_ignore_ascii_case(&root))
                    .map(|d| d.total)
            })
        } else {
            None
        };
        let options = ScanOptions::default();
        match ScanSession::start(&normalized, volume_total, options) {
            Ok(session) => {
                self.last_scan_path = Some(session.root_display.clone());
                self.settings.last_path = Some(session.root_display.clone());
                self.settings.save();
                self.session = Some(session);
                self.path_input = normalized;
            }
            Err(e) => {
                self.scan_error = Some(e.clone());
                self.toast = Some(Toast::error(e));
            }
        }
        ctx.request_repaint();
    }

    fn poll_scan(&mut self, ctx: &egui::Context) {
        let mut finished = None;
        if let Some(session) = self.session.as_mut() {
            if let Some(tree) = session.try_finish() {
                finished = Some(tree);
            } else if session.is_running() {
                ctx.request_repaint_after(Duration::from_millis(80));
            }
        }
        if let Some(tree) = finished {
            self.session = None;
            self.adopt_tree(tree);
        } else if let Some(session) = &self.session {
            if session.is_running() {
                ctx.request_repaint_after(Duration::from_millis(80));
            }
        }
    }

    fn adopt_tree(&mut self, tree: Tree) {
        let partial = tree.partial;
        let files = tree.total_files();
        let dirs = tree.total_dirs();
        let bytes = tree.total_bytes();
        self.view_root = tree.root;
        self.selected = Some(tree.root);
        self.expanded.clear();
        self.expanded.insert(tree.root);
        self.expanded_rev += 1;
        self.generation += 1;
        self.rows_cache = None;
        self.search_cache = None;
        self.folder_stats = None;
        self.tiles_cache = None;
        self.path_input = tree.root_path.clone();
        self.last_scan_path = Some(tree.root_path.clone());
        self.tree = Some(tree);
        self.drives = sys::drives();
        if let Some(query) = self.pending_search.take() {
            self.search = query;
            self.search_cache = None;
            self.ensure_search();
        }
        if let Some(rel) = self.pending_view.take() {
            if let Some(tree) = &self.tree {
                if let Some(id) = find_relative(tree, &rel) {
                    self.navigate(id);
                }
            }
        }
        if self.pending_delete_largest {
            self.pending_delete_largest = false;
            let info = self.tree.as_ref().and_then(|tree| {
                let id = *tree.top_files(tree.root, 1).first()?;
                let node = tree.node(id);
                Some((id, tree.path_of(id), node.size))
            });
            if let Some((id, path, size)) = info {
                self.selected = Some(id);
                self.modal = Some(Modal::Delete {
                    id,
                    path,
                    size,
                    is_dir: false,
                });
            }
        }
        if self.shot.is_some() {
            // Ekran goruntusunde bildirim balonu gorunmesin.
            self.toast = None;
            return;
        }
        self.toast = Some(if partial {
            Toast::warning(format!(
                "Tarama iptal edildi · kısmi sonuç · {} · {}",
                fmt::count(files),
                fmt::bytes(bytes)
            ))
        } else {
            Toast::info(format!(
                "Tarama bitti · {} dosya · {} klasör · {}",
                fmt::count(files),
                fmt::count(dirs),
                fmt::bytes(bytes)
            ))
        });
    }

    pub fn cancel_scan(&mut self) {
        if let Some(s) = &self.session {
            s.cancel();
        }
    }

    // -----------------------------------------------------------------
    // Gezinme ve secim
    // -----------------------------------------------------------------

    pub fn select(&mut self, id: u32) {
        self.selected = Some(id);
    }

    /// Görünümü verilen klasöre taşır ve atalarını açar.
    pub fn navigate(&mut self, id: u32) {
        let Some(tree) = &self.tree else {
            return;
        };
        if !tree.node(id).is_dir {
            self.selected = Some(id);
            return;
        }
        let root = tree.root;
        let mut ancestors = Vec::new();
        let mut cur = id;
        while cur != NONE && cur != root {
            ancestors.push(cur);
            cur = tree.parent_of(cur);
        }
        self.view_root = id;
        self.selected = Some(id);
        self.expanded.insert(root);
        for a in ancestors {
            self.expanded.insert(a);
        }
        self.expanded_rev += 1;
        self.folder_stats = None;
    }

    pub fn toggle_expand(&mut self, id: u32) {
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
        self.expanded_rev += 1;
    }

    pub fn parent_id(&self, id: u32) -> Option<u32> {
        let tree = self.tree.as_ref()?;
        let parent = tree.parent_of(id);
        (parent != NONE).then_some(parent)
    }

    // -----------------------------------------------------------------
    // Onbellekler
    // -----------------------------------------------------------------

    pub fn ensure_rows(&mut self) {
        let stale = match &self.rows_cache {
            Some(c) => c.generation != self.generation || c.expanded_rev != self.expanded_rev,
            None => true,
        };
        if !stale {
            return;
        }
        let Some(tree) = &self.tree else {
            self.rows_cache = None;
            return;
        };
        let mut rows: Vec<(u32, u16)> = Vec::new();
        let mut stack: Vec<(u32, u16)> = vec![(tree.root, 0)];
        while let Some((id, depth)) = stack.pop() {
            rows.push((id, depth));
            if self.expanded.contains(&id) {
                let kids = tree.children_sorted(id);
                for &kid in kids.iter().rev() {
                    stack.push((kid, depth + 1));
                }
            }
        }
        self.rows_cache = Some(RowsCache {
            generation: self.generation,
            expanded_rev: self.expanded_rev,
            rows,
        });
    }

    pub fn ensure_search(&mut self) {
        let stale = match &self.search_cache {
            Some(c) => c.query != self.search || c.generation != self.generation,
            None => true,
        };
        if !stale {
            return;
        }
        let hits = if self.search.trim().is_empty() {
            Vec::new()
        } else if let Some(tree) = &self.tree {
            tree.find(&self.search, 300)
        } else {
            Vec::new()
        };
        self.search_cache = Some(SearchCache {
            query: self.search.clone(),
            generation: self.generation,
            hits,
        });
    }

    pub fn ensure_stats(&mut self) {
        let stale = match &self.folder_stats {
            Some(s) => s.root != self.view_root || s.generation != self.generation,
            None => true,
        };
        if !stale {
            return;
        }
        let Some(tree) = &self.tree else {
            self.folder_stats = None;
            return;
        };
        let types = tree.type_breakdown(self.view_root);
        let top = tree.top_files(self.view_root, 16);
        self.folder_stats = Some(FolderStats {
            root: self.view_root,
            generation: self.generation,
            types,
            top,
        });
    }

    pub fn ensure_tiles(&mut self, rect: Rect) {
        let width = rect.width().round().max(1.0) as i32;
        let height = rect.height().round().max(1.0) as i32;
        let stale = match &self.tiles_cache {
            Some(c) => {
                c.root != self.view_root
                    || c.generation != self.generation
                    || c.width != width
                    || c.height != height
            }
            None => true,
        };
        if !stale {
            return;
        }
        let tiles = if let Some(tree) = &self.tree {
            let r = treemap::TRect::new(0.0, 0.0, width as f32, height as f32);
            treemap::build(tree, self.view_root, r, TreemapOptions::default())
        } else {
            Vec::new()
        };
        self.tiles_cache = Some(TilesCache {
            root: self.view_root,
            generation: self.generation,
            width,
            height,
            tiles,
        });
    }

    // -----------------------------------------------------------------
    // Eylemler
    // -----------------------------------------------------------------

    pub fn perform(&mut self, action: UiAction, ctx: &egui::Context) {
        match action {
            UiAction::Select(id) => self.select(id),
            UiAction::View(id) => self.navigate(id),
            UiAction::Toggle(id) => self.toggle_expand(id),
            UiAction::OpenExplorer(id) => {
                let Some(tree) = &self.tree else { return };
                let path = tree.path_of(id);
                let is_dir = tree.node(id).is_dir;
                if let Err(e) = open_in_explorer(&path, is_dir) {
                    self.toast = Some(Toast::error(e));
                }
            }
            UiAction::CopyPath(id) => {
                let path = self.node_path(id);
                ctx.copy_text(path);
                self.toast = Some(Toast::info("Yol kopyalandı."));
            }
            UiAction::AskDelete(id) => {
                let Some(tree) = &self.tree else { return };
                if id == tree.root {
                    self.toast = Some(Toast::warning("Tarama kökü silinemez."));
                    return;
                }
                let path = tree.path_of(id);
                let node = tree.node(id);
                self.modal = Some(Modal::Delete {
                    id,
                    path,
                    size: node.size,
                    is_dir: node.is_dir,
                });
            }
        }
    }

    pub fn delete_node(&mut self, id: u32, ctx: &egui::Context) {
        let Some(tree) = &self.tree else { return };
        if id == tree.root {
            return;
        }
        let path = tree.path_of(id);
        let parent = tree.parent_of(id);
        let inside_view = tree.is_ancestor(id, self.view_root);
        let inside_selection = self
            .selected
            .map(|s| tree.is_ancestor(id, s))
            .unwrap_or(false);

        match sys::recycle(&[path]) {
            Ok(()) => {
                let Some(tree) = self.tree.as_mut() else {
                    return;
                };
                let removed = tree.remove(id).unwrap_or(0);
                self.generation += 1;
                self.expanded.remove(&id);
                self.expanded_rev += 1;
                self.rows_cache = None;
                self.search_cache = None;
                self.folder_stats = None;
                self.tiles_cache = None;
                self.modal = None;
                if inside_view {
                    self.view_root = if parent != NONE { parent } else { tree.root };
                }
                if inside_selection || self.selected == Some(id) {
                    self.selected = Some(if parent != NONE { parent } else { tree.root });
                }
                self.toast = Some(Toast::info(format!(
                    "Geri dönüşüme taşındı · {}",
                    fmt::bytes(removed)
                )));
                ctx.request_repaint();
            }
            Err(e) => {
                self.modal = Some(Modal::Error {
                    title: "Taşınamadı".into(),
                    body: e,
                });
            }
        }
    }

    pub fn toggle_theme(&mut self, ctx: &egui::Context) {
        let next = match self.p.mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.p = theme::palette(next);
        self.settings.theme = next;
        self.settings.save();
        theme::apply(ctx, next);
        self.tiles_cache = None;
    }
    // -----------------------------------------------------------------
    // Kisayollar, ekran goruntusu
    // -----------------------------------------------------------------

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.memory(|m| m.focused().is_some());
        let mut rescan = false;
        let mut escape = false;
        let mut enter = false;
        let mut parent = false;
        let mut focus_search = false;
        ctx.input_mut(|i| {
            rescan = i.consume_key(egui::Modifiers::NONE, egui::Key::F5);
            escape = i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            enter = i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            parent = i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp);
            focus_search = i.consume_key(egui::Modifiers::CTRL, egui::Key::F);
        });
        if focus_search {
            ctx.memory_mut(|m| m.request_focus(Id::unique(SEARCH_ID)));
        }
        if escape {
            if self.modal.is_some() {
                self.modal = None;
            } else if self.session.is_some() {
                self.cancel_scan();
            } else if !self.search.is_empty() {
                self.search.clear();
                self.search_cache = None;
            } else if let Some(sel) = self.selected {
                if sel != self.view_root {
                    if let Some(p) = self.parent_id(sel) {
                        self.navigate(p);
                    }
                }
            }
        }
        if rescan {
            if let Some(path) = self.last_scan_path.clone() {
                self.start_scan(ctx, &path);
            }
        }
        if enter && !typing {
            if let Some(sel) = self.selected {
                self.navigate(sel);
            }
        }
        if parent && !typing {
            if let Some(p) = self.parent_id(self.view_root) {
                self.navigate(p);
            }
        }
    }

    fn handle_shot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else { return };
        if shot.start.elapsed() > Duration::from_secs(120) {
            self.shot = None;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if shot.requested {
            if shot.done.load(Ordering::SeqCst) {
                self.shot = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ctx.request_repaint_after(Duration::from_millis(40));
            }
            return;
        }
        let ready = if shot.wait_for_scan {
            self.tree.is_some()
                || (self.session.is_none() && shot.start.elapsed() > Duration::from_secs(3))
        } else {
            true
        };
        if !ready || shot.start.elapsed() < Duration::from_millis(900) {
            ctx.request_repaint_after(Duration::from_millis(80));
            return;
        }
        shot.requested = true;
        let path = shot.path.clone();
        let done = shot.done.clone();
        ctx.request_screenshot(move |image| {
            if let Err(e) = save_png(&path, &image) {
                eprintln!("ekran goruntusu kaydedilemedi: {e}");
            }
            done.store(true, Ordering::SeqCst);
        });
        ctx.request_repaint();
    }

    // -----------------------------------------------------------------
    // Toast ve modal
    // -----------------------------------------------------------------

    fn toast_ui(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else { return };
        let age = toast.at.elapsed().as_secs_f32();
        if age > 4.0 {
            self.toast = None;
            return;
        }
        let fade = if age > 3.4 { (4.0 - age) / 0.6 } else { 1.0 };
        let kind_color = match toast.kind {
            ToastKind::Info => self.p.ink_muted,
            ToastKind::Warning => self.p.warning,
            ToastKind::Error => self.p.danger,
        };
        let text = toast.text.clone();
        let p = self.p;
        egui::Area::new(Id::unique("iskandil-toast"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -56.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_max_width(560.0);
                egui::Frame::NONE
                    .fill(p.panel.gamma_multiply(fade))
                    .stroke(Stroke::new(1.0, p.hairline.gamma_multiply(fade)))
                    .inner_margin(egui::Margin::symmetric(14, 9))
                    .corner_radius(egui::CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (bar, _) = ui.allocate_exact_size(vec2(2.0, 16.0), Sense::hover());
                            ui.painter()
                                .rect_filled(bar, 0.0, kind_color.gamma_multiply(fade));
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(text)
                                    .font(theme::font_mono(10.5))
                                    .color(p.ink.gamma_multiply(fade)),
                            );
                        });
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(80));
    }

    fn modal_ui(&mut self, ctx: &egui::Context) {
        let Some(modal) = &self.modal else { return };
        let p = self.p;
        match modal {
            Modal::Delete {
                id,
                path,
                size,
                is_dir,
            } => {
                let id = *id;
                let path = path.clone();
                let size = *size;
                let is_dir = *is_dir;
                let mut decision: Option<bool> = None;
                egui::Modal::new(Id::unique("iskandil-delete"))
                    .backdrop_color(Color32::from_black_alpha(120))
                    .frame(
                        egui::Frame::NONE
                            .fill(p.panel)
                            .stroke(Stroke::new(1.0, p.hairline_strong))
                            .inner_margin(egui::Margin::same(22))
                            .corner_radius(egui::CornerRadius::same(2)),
                    )
                    .show(ctx, |ui| {
                        ui.set_width(430.0);
                        widgets::section_label(ui, &p, "GERİ DÖNÜŞÜM KUTUSU");
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(if is_dir {
                                "Klasörü geri dönüşüme taşı"
                            } else {
                                "Dosyayı geri dönüşüme taşı"
                            })
                            .font(theme::font_display(21.0))
                            .color(p.ink),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(&path)
                                .font(theme::font_mono(10.0))
                                .color(p.ink_muted),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!(
                                "{} · kalıcı olarak silinmez, Geri Dönüşüm Kutusu'na gider.",
                                fmt::bytes(size)
                            ))
                            .font(theme::font_body(12.0))
                            .color(p.ink_muted),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if widgets::ghost_button(ui, "VAZGEÇ", &p).clicked() {
                                decision = Some(false);
                            }
                            ui.add_space(6.0);
                            let resp = widgets::text_button(ui, "TAŞI →", &p, true);
                            if resp.clicked() {
                                decision = Some(true);
                            }
                        });
                    });
                if let Some(go) = decision {
                    if go {
                        self.delete_node(id, ctx);
                    } else {
                        self.modal = None;
                    }
                }
            }
            Modal::Error { title, body } => {
                let title = title.clone();
                let body = body.clone();
                let mut close = false;
                egui::Modal::new(Id::unique("iskandil-error"))
                    .backdrop_color(Color32::from_black_alpha(120))
                    .frame(
                        egui::Frame::NONE
                            .fill(p.panel)
                            .stroke(Stroke::new(1.0, p.hairline_strong))
                            .inner_margin(egui::Margin::same(22))
                            .corner_radius(egui::CornerRadius::same(2)),
                    )
                    .show(ctx, |ui| {
                        ui.set_width(420.0);
                        widgets::section_label(ui, &p, "HATA");
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(title)
                                .font(theme::font_display(20.0))
                                .color(p.ink),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(body)
                                .font(theme::font_body(12.5))
                                .color(p.ink_muted),
                        );
                        ui.add_space(14.0);
                        if widgets::ghost_button(ui, "KAPAT", &p).clicked() {
                            close = true;
                        }
                    });
                if close {
                    self.modal = None;
                }
            }
        }
    }

    pub(crate) fn chrome_ui(&mut self, ui: &mut egui::Ui) {
        let p = self.p;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, p.panel);
        painter.hline(
            rect.x_range(),
            rect.bottom() - 0.5,
            Stroke::new(1.0, p.hairline),
        );

        let left = rect.left() + 20.0;
        painter.text(
            pos2(left, rect.center().y - 1.0),
            Align2::LEFT_CENTER,
            "İSKANDİL",
            theme::font_display(15.0),
            p.ink,
        );
        let brand_w = painter
            .layout_no_wrap("İSKANDİL".to_owned(), theme::font_display(15.0), p.ink)
            .size()
            .x;
        painter.text(
            pos2(left + brand_w + 12.0, rect.center().y + 1.0),
            Align2::LEFT_CENTER,
            "disk derinlik ölçer",
            theme::font_mono(8.5),
            p.ink_faint,
        );

        // Sag taraf: yeniden tara / durdur + tema
        let mut actions: Vec<&str> = Vec::new();
        if self.session.is_some() {
            actions.push("DURDUR");
        } else if self.tree.is_some() {
            actions.push("YENİDEN TARA");
        }
        let mut right = rect.right() - 20.0;
        let theme_label = match p.mode {
            ThemeMode::Light => "KOYU",
            ThemeMode::Dark => "AÇIK",
        };
        let theme_rect =
            Rect::from_min_size(pos2(right - 46.0, rect.center().y - 11.0), vec2(46.0, 22.0));
        let theme_resp = ui.interact(
            theme_rect,
            ui.scope_id().with("theme-toggle"),
            Sense::click(),
        );
        ui.painter().text(
            pos2(right - 23.0, rect.center().y),
            Align2::CENTER_CENTER,
            theme_label,
            theme::font_mono(9.0),
            if theme_resp.hovered() {
                p.ink
            } else {
                p.ink_muted
            },
        );
        if theme_resp.hovered() {
            ui.painter().hline(
                right - 46.0..=right,
                rect.center().y + 9.0,
                Stroke::new(1.0, p.accent),
            );
        }
        if theme_resp.clicked() {
            self.toggle_theme(ui.ctx());
        }
        right -= 62.0;

        for label in actions.into_iter().rev() {
            let font = theme::font_mono(9.5);
            let w = ui
                .painter()
                .layout_no_wrap(label.to_owned(), font.clone(), p.ink)
                .size()
                .x;
            let btn_rect =
                Rect::from_min_size(pos2(right - w, rect.center().y - 12.0), vec2(w, 24.0));
            let resp = ui.interact(btn_rect, ui.scope_id().with(label), Sense::click());
            if resp.hovered() {
                ui.painter().text(
                    pos2(right - w * 0.5, rect.center().y),
                    Align2::CENTER_CENTER,
                    label,
                    font.clone(),
                    p.accent,
                );
                ui.painter().hline(
                    right - w..=right,
                    rect.center().y + 10.0,
                    Stroke::new(1.0, p.accent),
                );
            } else {
                ui.painter().text(
                    pos2(right - w * 0.5, rect.center().y),
                    Align2::CENTER_CENTER,
                    label,
                    font,
                    p.ink_muted,
                );
            }
            if resp.clicked() {
                if label == "DURDUR" {
                    self.cancel_scan();
                } else if let Some(path) = self.last_scan_path.clone() {
                    self.start_scan(ui.ctx(), &path);
                }
            }
            right -= w + 20.0;
        }
    }

    fn save_settings(&mut self) {
        self.settings.theme = self.p.mode;
        if let Some(path) = &self.last_scan_path {
            self.settings.last_path = Some(path.clone());
        }
        self.settings.save();
    }
}

impl eframe::App for IskandilApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame += 1;
        let ctx = ui.ctx().clone();
        // Kok zemin: panellerin arkasinda siyah clear rengi kalmasin.
        ui.painter().rect_filled(ui.max_rect(), 0.0, self.p.bg);
        self.poll_scan(&ctx);
        self.handle_shortcuts(&ctx);
        self.handle_shot(&ctx);

        let marker = match self.mode() {
            Mode::Start => Some(0.02),
            Mode::Scanning => {
                let t = ctx.input(|i| i.time) as f32;
                Some((t * 0.09).fract())
            }
            Mode::Workspace => None,
        };
        egui::Panel::left("iskandil-ruler")
            .exact_size(18.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(self.p.bg))
            .show(ui, |ui| {
                widgets::ruler(ui, &self.p, marker);
            });

        match self.mode() {
            Mode::Start => self.start_ui(ui, &ctx),
            Mode::Scanning => self.scanning_ui(ui, &ctx),
            Mode::Workspace => self.workspace_ui(ui, &ctx),
        }

        self.toast_ui(&ctx);
        self.modal_ui(&ctx);
    }

    fn on_exit(&mut self) {
        self.save_settings();
        if let Some(s) = &self.session {
            s.cancel();
        }
    }
}

fn find_relative(tree: &Tree, rel: &str) -> Option<u32> {
    let mut current = tree.root;
    for part in rel.split(['\\', '/']).filter(|p| !p.is_empty()) {
        let needle = fmt::fold_tr(part);
        let mut found = None;
        for child in tree.children(current) {
            if fmt::fold_tr(&tree.node(child).name) == needle {
                found = Some(child);
                break;
            }
        }
        current = found?;
    }
    Some(current)
}

fn default_start_path() -> String {
    if let Some(home) = std::env::var_os("USERPROFILE") {
        let downloads = PathBuf::from(home).join("Downloads");
        if downloads.is_dir() {
            return downloads.display().to_string();
        }
    }
    "C:\\".to_string()
}

fn open_in_explorer(path: &str, is_dir: bool) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let arg = if is_dir {
        format!("\"{path}\"")
    } else {
        format!("/select,\"{path}\"")
    };
    std::process::Command::new("explorer.exe")
        .raw_arg(arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Gezgin açılamadı: {e}"))
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let [w, h] = image.size;
    let mut raw = Vec::with_capacity(w * h * 4);
    for px in &image.pixels {
        raw.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
    }
    let buffer =
        image::RgbaImage::from_raw(w as u32, h as u32, raw).ok_or("görüntü tamponu geçersiz")?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    buffer.save(path).map_err(|e| e.to_string())
}
