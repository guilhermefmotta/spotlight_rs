#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align2, Color32, ColorImage, CornerRadius, FontId, Key, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::{Code, HotKey, Modifiers}};
use image::DynamicImage;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const W: f32 = 640.0;
const SEARCH_H: f32 = 64.0;
const ITEM_H: f32 = 46.0;
const ICON_SIZE: f32 = 28.0;
const MAX_ITEMS: usize = 8;
const RADIUS: CornerRadius = CornerRadius::same(12);

fn main() -> eframe::Result {
    // Set up global hotkey manager on the main thread.
    let hotkey_manager = GlobalHotKeyManager::new().ok();
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyD);
    if let Some(manager) = &hotkey_manager {
        let _ = manager.register(hotkey);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Spotlight")
            .with_inner_size([W, SEARCH_H])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_resizable(false)
            .with_visible(false), // Start hidden
        ..Default::default()
    };
    let hotkey_id = hotkey.id();
    eframe::run_native(
        "Spotlight",
        options,
        Box::new(move |cc| Ok(Box::new(Spotlight::new(cc, hotkey_manager, hotkey_id)))),
    )
}

#[derive(Clone)]
struct App {
    name: String,
    exec: String,
    comment: Option<String>,
    icon: Option<String>,
}

struct Spotlight {
    query: String,
    all: Vec<App>,
    results: Vec<App>,
    selected: usize,
    need_focus: bool,
    positioned: bool,
    visible: bool,
    _hotkey_manager: Option<GlobalHotKeyManager>,
    hotkey_id: u32,
    icons: HashMap<String, TextureHandle>,
}

impl Spotlight {
    fn new(_cc: &eframe::CreationContext, hotkey_manager: Option<GlobalHotKeyManager>, hotkey_id: u32) -> Self {
        Self {
            query: String::new(),
            all: scan_desktop_files(),
            results: Vec::new(),
            selected: 0,
            need_focus: true,
            positioned: false,
            visible: false,
            _hotkey_manager: hotkey_manager,
            hotkey_id,
            icons: HashMap::new(),
        }
    }

    fn filter(&mut self) {
        let q = self.query.to_ascii_lowercase();
        self.results = self
            .all
            .iter()
            .filter(|a| a.name.to_ascii_lowercase().contains(&q))
            .cloned()
            .collect();
        self.selected = 0;
    }

    fn launch(&self) {
        if let Some(app) = self.results.get(self.selected) {
            launch_exec(&app.exec);
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.filter();
        self.need_focus = true;
        self.positioned = false;
    }

    fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
    }

    fn get_icon_by_ref(&mut self, ctx: &egui::Context, icon_ref: String) -> Option<&TextureHandle> {
        let key = icon_ref.clone();
        let entry = self.icons.entry(key.clone());
        match entry {
            std::collections::hash_map::Entry::Occupied(o) => Some(o.into_mut()),
            std::collections::hash_map::Entry::Vacant(v) => {
                let path = resolve_icon_path(&icon_ref)?;
                let image = load_icon_image(&path)?;
                let texture = ctx.load_texture(key, image, TextureOptions::NEAREST);
                Some(v.insert(texture))
            }
        }
    }
}

impl eframe::App for Spotlight {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for global hotkey events.
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id() == self.hotkey_id && event.state() == HotKeyState::Pressed {
                if self.visible {
                    self.hide();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                } else {
                    self.show();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                }
            }
        }

        // If hidden, don't render anything
        if !self.visible {
            return;
        }

        // Position window on first frame: horizontally centered, ~28% from top
        if !self.positioned {
            if let Some(mon) = ctx.input(|i| i.viewport().monitor_size) {
                let x = (mon.x - W) / 2.0;
                let y = mon.y * 0.28;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, y)));
            }
            self.positioned = true;
        }

        let mut close = false;
        let mut do_launch = false;

        // consume_key intercepts before TextEdit can swallow Escape/Enter
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
                close = true;
            }
            if i.consume_key(egui::Modifiers::NONE, Key::Enter) {
                do_launch = true;
            }
            if i.key_pressed(Key::ArrowDown) && self.selected + 1 < self.results.len() {
                self.selected += 1;
            }
            if i.key_pressed(Key::ArrowUp) && self.selected > 0 {
                self.selected -= 1;
            }
        });

        if do_launch {
            self.launch();
            close = true;
        }
        if close {
            self.hide();
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        // Resize window to fit results
        let visible = self.results.len().min(MAX_ITEMS);
        let target_h = SEARCH_H + visible as f32 * ITEM_H;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(W, target_h)));

        let bg = Color32::from_rgba_premultiplied(22, 22, 22, 240);
        let border_col = Color32::from_gray(55);
        let sel_col = Color32::from_rgba_premultiplied(74, 144, 226, 190);
        let hover_col = Color32::from_rgba_premultiplied(60, 60, 60, 90);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();

                ui.painter().rect_filled(full, RADIUS, bg);
                ui.painter().rect_stroke(
                    full,
                    RADIUS,
                    Stroke::new(1.0, border_col),
                    StrokeKind::Outside,
                );

                // ── Search bar ───────────────────────────────────────
                let search_rect =
                    egui::Rect::from_min_size(full.min, Vec2::new(W, SEARCH_H));
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(search_rect), |ui| {
                    ui.set_min_height(SEARCH_H);
                    ui.horizontal_centered(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("🔍")
                                .size(20.0)
                                .color(Color32::from_gray(130)),
                        );
                        ui.add_space(10.0);
                        let edit = egui::TextEdit::singleline(&mut self.query)
                            .frame(false)
                            .hint_text("Search applications...")
                            .font(FontId::proportional(20.0))
                            .text_color(Color32::WHITE)
                            .desired_width(f32::INFINITY);
                        let resp = ui.add(edit);
                        if self.need_focus {
                            resp.request_focus();
                            self.need_focus = false;
                        }
                        if resp.changed() {
                            self.filter();
                        }
                    });
                });

                if self.results.is_empty() {
                    return;
                }

                // Divider
                ui.painter().hline(
                    (full.min.x + 12.0)..=(full.max.x - 12.0),
                    full.min.y + SEARCH_H,
                    Stroke::new(1.0, border_col),
                );

                // ── Dropdown results ─────────────────────────────────
                let list_rect = egui::Rect::from_min_max(
                    egui::pos2(full.min.x, full.min.y + SEARCH_H),
                    full.max,
                );

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(list_rect), |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(MAX_ITEMS as f32 * ITEM_H)
                        .show(ui, |ui| {
                            let w = ui.available_width();
                            let max_results = self.results.len().min(MAX_ITEMS);
                            for i in 0..max_results {
                                let app = self.results[i].clone();
                                let is_sel = i == self.selected;
                                let (row, resp) =
                                    ui.allocate_exact_size(Vec2::new(w, ITEM_H), Sense::click());

                                if is_sel {
                                    ui.painter()
                                        .rect_filled(row, CornerRadius::same(6), sel_col);
                                } else if resp.hovered() {
                                    ui.painter()
                                        .rect_filled(row, CornerRadius::same(6), hover_col);
                                }

                                if resp.hovered() {
                                    self.selected = i;
                                }
                                if resp.clicked() {
                                    launch_exec(&self.results[i].exec);
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }

                                let icon_center = row.min + Vec2::new(12.0 + ICON_SIZE / 2.0, ITEM_H / 2.0);
                                let icon_rect = egui::Rect::from_center_size(icon_center, Vec2::splat(ICON_SIZE));
                                let icon_texture = if let Some(icon_ref) = app.icon.clone() {
                                    self.get_icon_by_ref(ctx, icon_ref)
                                } else {
                                    None
                                };
                                if let Some(texture) = icon_texture {
                                    ui.painter().rect_filled(icon_rect, CornerRadius::same((ICON_SIZE / 2.0) as u8), Color32::from_gray(30));
                                    ui.painter().image(
                                        texture.id(),
                                        icon_rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        Color32::WHITE,
                                    );
                                } else {
                                    ui.painter().circle_filled(icon_center, ICON_SIZE / 2.0, Color32::from_gray(50));
                                    let icon_text = app
                                        .name
                                        .chars()
                                        .next()
                                        .map(|c| c.to_string())
                                        .unwrap_or_else(|| "?".to_string());
                                    ui.painter().text(
                                        icon_center,
                                        Align2::CENTER_CENTER,
                                        icon_text,
                                        FontId::proportional(14.0),
                                        Color32::WHITE,
                                    );
                                }

                                // App name
                                ui.painter().text(
                                    row.min + Vec2::new(12.0 + ICON_SIZE + 16.0, 8.0),
                                    Align2::LEFT_TOP,
                                    &app.name,
                                    FontId::proportional(16.0),
                                    Color32::WHITE,
                                );

                                // Description
                                if let Some(c) = &app.comment {
                                    ui.painter().text(
                                        row.min + Vec2::new(12.0 + ICON_SIZE + 16.0, 28.0),
                                        Align2::LEFT_TOP,
                                        c,
                                        FontId::proportional(12.0),
                                        Color32::from_gray(115),
                                    );
                                }
                            }
                        });
                });
            });
    }
}

// ── App discovery ────────────────────────────────────────────────────────────

fn scan_desktop_files() -> Vec<App> {
    #[cfg(target_os = "linux")]
    {
        scan_linux_apps()
    }
    #[cfg(target_os = "windows")]
    {
        scan_windows_apps()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
fn scan_linux_apps() -> Vec<App> {
    let home = std::env::var("HOME").unwrap_or_default();
    let static_dirs: &[&str] = &[
        "/usr/share/applications",
        "/usr/local/share/applications",
        "/var/lib/flatpak/exports/share/applications",
        "/var/lib/snapd/desktop/applications",
    ];
    let home_dirs = [
        format!("{home}/.local/share/applications"),
        format!("{home}/.local/share/flatpak/exports/share/applications"),
    ];

    let mut apps = Vec::new();
    for dir in static_dirs
        .iter()
        .copied()
        .chain(home_dirs.iter().map(|s| s.as_str()))
    {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(a) = parse_desktop(&path) {
                        apps.push(a);
                    }
                }
            }
        }
    }

    apps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

#[cfg(target_os = "windows")]
fn scan_windows_apps() -> Vec<App> {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    let mut apps = Vec::new();

    // Scan uninstall registry keys for installed applications
    let keys = [
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for key_path in &keys {
        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(key_path) {
            for subkey_name in key.enum_keys().flatten() {
                if let Ok(subkey) = key.open_subkey(&subkey_name) {
                    // Try to get DisplayName
                    if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                        // Try to get executable path - look for common registry values
                        let raw_icon = subkey.get_value::<String, _>("DisplayIcon").ok();
                        let exec = raw_icon
                            .clone()
                            .or_else(|| subkey.get_value::<String, _>("InstallLocation").ok())
                            .and_then(|path| {
                                // Clean up paths: remove quotes and extract just the executable path
                                let clean_path = path.trim_matches('"').to_string();
                                Some(clean_path)
                            });

                        if let Some(exec_path) = exec {
                            // Skip entries without executables or system entries
                            if !exec_path.is_empty() && !display_name.contains("Update") {
                                let app = App {
                                    name: display_name,
                                    exec: exec_path,
                                    comment: None,
                                    icon: raw_icon.map(|i| i.trim_matches('"').to_string()),
                                };
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }
    }

    apps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

#[cfg(target_os = "linux")]
fn parse_desktop(path: &PathBuf) -> Option<App> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut comment: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut in_entry = false;
    let mut is_app = false;
    let mut skip = false;

    for line in text.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_entry = false;
            continue;
        }
        if !in_entry {
            continue;
        }

        if let Some(v) = line.strip_prefix("Type=") {
            if v == "Application" {
                is_app = true;
            }
        } else if let Some(v) = line.strip_prefix("Name=") {
            if name.is_none() {
                name = Some(v.to_string());
            }
        } else if let Some(v) = line.strip_prefix("Exec=") {
            exec = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("Icon=") {
            if icon.is_none() {
                icon = Some(v.to_string());
            }
        } else if let Some(v) = line.strip_prefix("Comment=") {
            if comment.is_none() {
                comment = Some(v.to_string());
            }
        } else if line == "NoDisplay=true" || line == "Hidden=true" {
            skip = true;
        }
    }

    if !is_app || skip {
        return None;
    }
    Some(App {
        name: name?,
        exec: exec?,
        comment,
        icon,
    })
}

fn launch_exec(exec: &str) {
    let args: Vec<&str> = exec
        .split_whitespace()
        .filter(|s| !s.starts_with('%'))
        .collect();
    if let Some((cmd, rest)) = args.split_first() {
        Command::new(cmd).args(rest).spawn().ok();
    }
}

fn load_icon_image(path: &Path) -> Option<ColorImage> {
    let data = std::fs::read(path).ok()?;
    let image = image::load_from_memory(&data).ok()?;
    let image = match image {
        DynamicImage::ImageRgba8(img) => img,
        img => img.to_rgba8(),
    };
    let resized = image::imageops::resize(&image, ICON_SIZE as u32, ICON_SIZE as u32, image::imageops::FilterType::Lanczos3);
    let samples = resized.as_flat_samples();
    let pixels = samples.as_slice();
    Some(ColorImage::from_rgba_unmultiplied([ICON_SIZE as usize, ICON_SIZE as usize], pixels))
}

fn resolve_icon_path(icon_ref: &str) -> Option<PathBuf> {
    let raw = icon_ref.trim_matches('"');
    let raw = raw.split(',').next().unwrap_or(raw).trim();
    let path = Path::new(raw);
    if path.exists() && path.is_file() {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
            let supported = ["png", "ico", "jpg", "jpeg", "bmp"];
            if supported.contains(&ext.as_str()) {
                return Some(path.to_path_buf());
            }
            if ext == "exe" {
                let icon_candidate = path.with_extension("ico");
                if icon_candidate.exists() {
                    return Some(icon_candidate);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        find_linux_icon(raw)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn find_linux_icon(name: &str) -> Option<PathBuf> {
    let base_dirs = [
        "/usr/share/icons/hicolor/256x256/apps",
        "/usr/share/icons/hicolor/128x128/apps",
        "/usr/share/icons/hicolor/48x48/apps",
        "/usr/share/pixmaps",
    ];
    let extensions = ["png", "ico", "xpm"];

    if name.contains('/') {
        let path = Path::new(name);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    for dir in base_dirs {
        for ext in extensions {
            let candidate = Path::new(dir).join(format!("{}.{}", name, ext));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}
