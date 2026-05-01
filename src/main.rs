#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Key, Sense, Stroke, StrokeKind, Vec2};
use std::path::PathBuf;
use std::process::Command;

const W: f32 = 640.0;
const SEARCH_H: f32 = 64.0;
const ITEM_H: f32 = 46.0;
const MAX_ITEMS: usize = 8;
const RADIUS: CornerRadius = CornerRadius::same(12);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Spotlight")
            .with_inner_size([W, SEARCH_H])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Spotlight",
        options,
        Box::new(|cc| Ok(Box::new(Spotlight::new(cc)))),
    )
}

#[derive(Clone)]
struct App {
    name: String,
    exec: String,
    comment: Option<String>,
}

struct Spotlight {
    query: String,
    all: Vec<App>,
    results: Vec<App>,
    selected: usize,
    need_focus: bool,
    positioned: bool,
}

impl Spotlight {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            query: String::new(),
            all: scan_desktop_files(),
            results: Vec::new(),
            selected: 0,
            need_focus: true,
            positioned: false,
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
}

impl eframe::App for Spotlight {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                            for (i, app) in self.results.iter().enumerate().take(MAX_ITEMS) {
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

                                // App name
                                ui.painter().text(
                                    row.min + Vec2::new(22.0, 8.0),
                                    Align2::LEFT_TOP,
                                    &app.name,
                                    FontId::proportional(16.0),
                                    Color32::WHITE,
                                );

                                // Description
                                if let Some(c) = &app.comment {
                                    ui.painter().text(
                                        row.min + Vec2::new(22.0, 28.0),
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

fn parse_desktop(path: &PathBuf) -> Option<App> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut comment: Option<String> = None;
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
