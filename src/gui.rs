use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, Margin, RichText, Rounding, Stroke};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::engine::HotkeyEngine;
use crate::ipc::{DaemonProxyHandle, GuiControlEvent};
use crate::logger::{self, LogEntry};
use crate::model::{KeyCombo, KeybindRule, LogicalKey, Modifiers};
use crate::tray::{TrayCommand, TrayController};

const BG_BASE: Color32 = Color32::from_rgb(16, 19, 26);
const BG_PANEL: Color32 = Color32::from_rgb(22, 26, 35);
const BG_CARD: Color32 = Color32::from_rgb(28, 33, 45);
const BG_KEYCAP: Color32 = Color32::from_rgb(40, 47, 63);
const BORDER_SUBTLE: Color32 = Color32::from_rgb(52, 60, 78);
const BORDER_ACCENT: Color32 = Color32::from_rgb(82, 152, 245);
const COLOR_ACTIVE: Color32 = Color32::from_rgb(72, 207, 134);
const COLOR_PAUSED: Color32 = Color32::from_rgb(239, 108, 108);
const COLOR_WARN: Color32 = Color32::from_rgb(255, 193, 94);
const COLOR_INFO: Color32 = Color32::from_rgb(138, 196, 255);
const TEXT_MUTED: Color32 = Color32::from_rgb(142, 152, 172);

pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_BASE;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(12, 14, 20);
    visuals.faint_bg_color = BG_CARD;

    let rounding = Rounding::same(6.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);

    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.inactive.bg_fill = BG_KEYCAP;
    visuals.widgets.inactive.weak_bg_fill = BG_KEYCAP;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);

    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 62, 84);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(52, 62, 84);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER_ACCENT);

    visuals.widgets.active.rounding = rounding;
    visuals.widgets.active.bg_fill = Color32::from_rgb(62, 76, 104);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, BORDER_ACCENT);

    visuals.selection.bg_fill = Color32::from_rgb(47, 94, 166);
    visuals.selection.stroke = Stroke::new(1.0_f32, BORDER_ACCENT);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    ctx.set_style(style);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordingTarget {
    Trigger(Uuid),
    Target(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevelFilter {
    All,
    WarnAndError,
}

pub struct HotkeysApp {
    engine: HotkeyEngine,
    tray: Option<TrayController>,
    proxy: Option<DaemonProxyHandle>,
    recording: Option<RecordingTarget>,
    status_message: Option<String>,
    config_path_display: String,
    log_path_display: String,
    allow_quit: bool,
    search_query: String,
    show_logs: bool,
    log_filter: LogLevelFilter,
    cached_logs: Vec<LogEntry>,
    last_log_refresh: Instant,
}

impl HotkeysApp {
    pub fn new(
        engine: HotkeyEngine,
        tray: Option<TrayController>,
        proxy: Option<DaemonProxyHandle>,
        log_path: PathBuf,
    ) -> Self {
        let config_path_display = AppConfig::default_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "config.toml".to_string());
        let cached_logs = logger::read_log_file_tail(500);

        Self {
            engine,
            tray,
            proxy,
            recording: None,
            status_message: None,
            config_path_display,
            log_path_display: log_path.display().to_string(),
            allow_quit: false,
            search_query: String::new(),
            show_logs: true,
            log_filter: LogLevelFilter::All,
            cached_logs,
            last_log_refresh: Instant::now(),
        }
    }

    fn persist_and_sync(&mut self) {
        // Save config.toml BEFORE syncing so a separate daemon process reads the latest rules.
        match self.engine.config.save() {
            Ok(()) => {
                self.status_message = Some("Saved configuration.".to_string());
            }
            Err(err) => {
                let msg = format!("Failed to save config: {}", err);
                log::error!("{}", msg);
                self.status_message = Some(msg);
            }
        }
        if let Err(err) = self.engine.sync_backend() {
            let msg = format!("Backend sync error: {}", err);
            log::error!("{}", msg);
            self.status_message = Some(msg);
        }
        if let Some(tray) = &self.tray {
            tray.set_global_enabled(self.engine.config.global_enabled);
        }
    }

    fn handle_key_recording(&mut self, ctx: &egui::Context) {
        let Some(rec_target) = self.recording else {
            let _ = self.engine.set_recording_active(false);
            return;
        };

        let _ = self.engine.set_recording_active(true);

        let mut captured: Option<KeyCombo> = None;
        let mut cancel = false;

        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } = event
                {
                    if *key == egui::Key::Escape && modifiers.is_none() {
                        cancel = true;
                        break;
                    }
                    if let Some(logical_key) = LogicalKey::from_egui_key(*key) {
                        let mods = Modifiers {
                            ctrl: modifiers.ctrl,
                            alt: modifiers.alt,
                            shift: modifiers.shift,
                            meta: modifiers.mac_cmd || modifiers.command && !modifiers.ctrl,
                        };
                        captured = Some(KeyCombo::new(mods, logical_key));
                        break;
                    }
                }
            }
        });

        if cancel {
            self.recording = None;
            let _ = self.engine.set_recording_active(false);
            return;
        }

        if let Some(combo) = captured {
            match rec_target {
                RecordingTarget::Trigger(id) => {
                    if let Some(rule) = self.engine.config.rules.iter_mut().find(|r| r.id == id) {
                        log::info!("Updated trigger for '{}' to {}", rule.name, combo);
                        rule.trigger = combo;
                    }
                }
                RecordingTarget::Target(id) => {
                    if let Some(rule) = self.engine.config.rules.iter_mut().find(|r| r.id == id) {
                        log::info!("Updated target for '{}' to {}", rule.name, combo);
                        rule.target = combo;
                    }
                }
            }
            self.recording = None;
            let _ = self.engine.set_recording_active(false);
            self.persist_and_sync();
        }
    }

    fn handle_tray_commands(&mut self, ctx: &egui::Context) {
        if let Some(tray) = &self.tray {
            let cmds: Vec<_> = std::iter::from_fn(|| tray.try_recv()).collect();
            for cmd in cmds {
                match cmd {
                    TrayCommand::ShowWindow => {
                        log::info!("Restoring window from system tray");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    TrayCommand::ToggleGlobalEnabled => {
                        self.engine.config.global_enabled = !self.engine.config.global_enabled;
                        log::info!(
                            "Toggled global hotkeys from tray -> enabled={}",
                            self.engine.config.global_enabled
                        );
                        self.persist_and_sync();
                    }
                    TrayCommand::Quit => {
                        log::info!("Quit requested from system tray");
                        self.allow_quit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }

        if let Some(proxy) = &self.proxy {
            while let Ok(evt) = proxy.control_rx.try_recv() {
                match evt {
                    GuiControlEvent::Focus => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    GuiControlEvent::SetGlobalEnabled(enabled) => {
                        self.engine.config.global_enabled = enabled;
                    }
                }
            }
        }
    }

    fn render_combo_editor(
        ui: &mut egui::Ui,
        id_salt: &str,
        combo: &mut KeyCombo,
        is_recording: bool,
        on_toggle_record: &mut bool,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            let btn = if is_recording {
                egui::Button::new(
                    RichText::new("Press keys... (Esc)")
                        .color(COLOR_WARN)
                        .strong(),
                )
                .stroke(Stroke::new(1.5_f32, COLOR_WARN))
                .fill(Color32::from_rgb(56, 44, 24))
            } else {
                egui::Button::new(
                    RichText::new(combo.to_string())
                        .monospace()
                        .strong()
                        .color(Color32::from_rgb(232, 238, 250)),
                )
                .fill(BG_KEYCAP)
                .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
            };

            if ui
                .add_sized([185.0, 28.0], btn)
                .on_hover_text("Click and press any key combination on your keyboard")
                .clicked()
            {
                *on_toggle_record = true;
            }

            ui.menu_button("Edit", |ui| {
                ui.label(RichText::new("Modifiers").strong());
                ui.horizontal(|ui| {
                    changed |= ui.checkbox(&mut combo.modifiers.ctrl, "Ctrl").changed();
                    changed |= ui.checkbox(&mut combo.modifiers.alt, "Alt").changed();
                    changed |= ui.checkbox(&mut combo.modifiers.shift, "Shift").changed();
                    changed |= ui
                        .checkbox(&mut combo.modifiers.meta, Modifiers::meta_label())
                        .changed();
                });
                ui.separator();
                ui.label(RichText::new("Key (including Media/Volume)").strong());
                egui::ComboBox::from_id_salt(id_salt)
                    .selected_text(format!("{} ({})", combo.key.label(), combo.key.category()))
                    .width(230.0)
                    .show_ui(ui, |ui| {
                        let mut current_category = "";
                        for &k in LogicalKey::ALL {
                            if k.category() != current_category {
                                current_category = k.category();
                                ui.separator();
                                ui.label(RichText::new(current_category).strong().small());
                            }
                            if ui.selectable_value(&mut combo.key, k, k.label()).changed() {
                                changed = true;
                            }
                        }
                    });
            });
        });

        changed
    }

    fn render_logs_panel(&mut self, ctx: &egui::Context) {
        if self.last_log_refresh.elapsed() >= Duration::from_millis(250) {
            self.cached_logs = logger::read_log_file_tail(500);
            self.last_log_refresh = Instant::now();
        }

        egui::TopBottomPanel::bottom("logs_panel")
            .resizable(true)
            .default_height(175.0)
            .min_height(110.0)
            .frame(
                egui::Frame::none()
                    .fill(BG_PANEL)
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                let mut clear_clicked = false;
                let filtered: Vec<_> = self
                    .cached_logs
                    .iter()
                    .filter(|entry| match self.log_filter {
                        LogLevelFilter::All => true,
                        LogLevelFilter::WarnAndError => {
                            matches!(entry.level, log::Level::Warn | log::Level::Error)
                        }
                    })
                    .collect();

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Diagnostic Logs").strong().size(13.5));
                    ui.label(
                        RichText::new(format!("({})", filtered.len()))
                            .small()
                            .color(TEXT_MUTED),
                    );
                    ui.separator();
                    ui.selectable_value(&mut self.log_filter, LogLevelFilter::All, "All");
                    ui.selectable_value(
                        &mut self.log_filter,
                        LogLevelFilter::WarnAndError,
                        "Warn / Error",
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Clear").clicked() {
                            clear_clicked = true;
                        }
                        if ui.small_button("Copy Logs").clicked() {
                            let dump = filtered
                                .iter()
                                .map(|e| format!("[{} {:<5}] {}", e.timestamp, e.level, e.message))
                                .collect::<Vec<_>>()
                                .join("\n");
                            ctx.copy_text(dump);
                            self.status_message = Some("Copied logs to clipboard.".to_string());
                        }
                        ui.label(
                            RichText::new(format!("File: {}", self.log_path_display))
                                .small()
                                .color(TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(12, 14, 20))
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("diagnostic_logs_scroll")
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if filtered.is_empty() {
                                    ui.label(
                                        RichText::new("No log entries matching filter.")
                                            .small()
                                            .color(TEXT_MUTED),
                                    );
                                }
                                for entry in filtered {
                                    let level_color = match entry.level {
                                        log::Level::Error => COLOR_PAUSED,
                                        log::Level::Warn => COLOR_WARN,
                                        log::Level::Info => COLOR_INFO,
                                        log::Level::Debug | log::Level::Trace => TEXT_MUTED,
                                    };
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        ui.label(
                                            RichText::new(&entry.timestamp)
                                                .monospace()
                                                .small()
                                                .color(TEXT_MUTED),
                                        );
                                        ui.label(
                                            RichText::new(format!("{:<5}", entry.level))
                                                .monospace()
                                                .small()
                                                .strong()
                                                .color(level_color),
                                        );
                                        ui.label(
                                            RichText::new(&entry.message)
                                                .monospace()
                                                .small()
                                                .color(Color32::from_rgb(220, 226, 238)),
                                        );
                                    });
                                }
                            });
                    });

                if clear_clicked {
                    logger::clear_memory_logs();
                    self.cached_logs.clear();
                }
            });
    }
}

impl eframe::App for HotkeysApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_tray_commands(ctx);
        self.handle_key_recording(ctx);
        if let Err(err) = self.engine.poll() {
            let msg = format!("Hotkey dispatch error: {}", err);
            log::error!("{}", msg);
            self.status_message = Some(msg);
        }

        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        if self.recording.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else if !is_minimized {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        // On Windows (--gui child process), closing the window exits only the GUI process
        // so the background daemon keeps running at ~0% CPU and minimal RAM.
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_quit {
            if self.proxy.is_some() {
                log::info!("Closing GUI window; background daemon remains active");
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                log::info!("Window minimized; hotkeys remain active");
                self.status_message =
                    Some("Minimized to background. Hotkeys remain active.".to_string());
            }
        }

        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::none()
                    .fill(BG_PANEL)
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Hotkeys")
                            .heading()
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(6.0);

                    let enabled = self.engine.config.global_enabled;
                    let (badge_text, badge_color, badge_bg) = if enabled {
                        (
                            "● Active",
                            COLOR_ACTIVE,
                            Color32::from_rgba_unmultiplied(72, 207, 134, 24),
                        )
                    } else {
                        (
                            "⏸ Paused",
                            COLOR_PAUSED,
                            Color32::from_rgba_unmultiplied(239, 108, 108, 24),
                        )
                    };
                    let status_btn =
                        egui::Button::new(RichText::new(badge_text).color(badge_color).strong())
                            .fill(badge_bg)
                            .stroke(Stroke::new(1.0_f32, badge_color));
                    if ui
                        .add(status_btn)
                        .on_hover_text("Click to pause or resume all global hotkeys")
                        .clicked()
                    {
                        self.engine.config.global_enabled = !enabled;
                        log::info!(
                            "Global hotkeys toggled via UI -> {}",
                            self.engine.config.global_enabled
                        );
                        self.persist_and_sync();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new("Quit").color(COLOR_PAUSED))
                            .on_hover_text("Stop hotkey listener and exit completely")
                            .clicked()
                        {
                            if let Some(proxy) = &self.proxy {
                                proxy.send_quit_daemon();
                            }
                            self.allow_quit = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        if ui.button("Restore Defaults").clicked() {
                            log::info!("Restoring default keybind rules");
                            self.engine.config.rules = AppConfig::default_rules();
                            self.recording = None;
                            self.persist_and_sync();
                        }

                        let logs_btn = egui::Button::new(if self.show_logs {
                            RichText::new("Logs (On)").color(COLOR_INFO).strong()
                        } else {
                            RichText::new("Logs").color(TEXT_MUTED)
                        });
                        if ui.add(logs_btn).clicked() {
                            self.show_logs = !self.show_logs;
                        }

                        let add_btn = egui::Button::new(
                            RichText::new("+ Add Keybind")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(48, 108, 204));
                        if ui.add(add_btn).clicked() {
                            let new_rule = KeybindRule::new(
                                format!("Custom Rule {}", self.engine.config.rules.len() + 1),
                                KeyCombo::new(
                                    Modifiers::new(true, true, false, false),
                                    LogicalKey::F8,
                                ),
                                KeyCombo::bare(LogicalKey::MediaPlayPause),
                            );
                            log::info!("Added new keybind rule '{}'", new_rule.name);
                            self.engine.config.rules.push(new_rule);
                            self.persist_and_sync();
                        }
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Filter rules by name or key...")
                            .desired_width(260.0),
                    );
                    if !self.search_query.is_empty() && ui.small_button("×").clicked() {
                        self.search_query.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let enabled_count = self
                            .engine
                            .config
                            .rules
                            .iter()
                            .filter(|r| r.enabled)
                            .count();
                        ui.label(
                            RichText::new(format!(
                                "{} enabled / {} total rules",
                                enabled_count,
                                self.engine.config.rules.len()
                            ))
                            .small()
                            .color(TEXT_MUTED),
                        );
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_footer")
            .frame(
                egui::Frame::none()
                    .fill(BG_BASE)
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(14.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.engine.backend_status())
                            .small()
                            .color(TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(last) = &self.engine.last_triggered_rule {
                            ui.label(
                                RichText::new(format!(
                                    "Last fired (#{}): {}",
                                    self.engine.total_triggers_count, last
                                ))
                                .small()
                                .color(COLOR_INFO),
                            );
                        } else {
                            ui.label(
                                RichText::new(format!("Config: {}", self.config_path_display))
                                    .small()
                                    .color(TEXT_MUTED),
                            );
                        }
                    });
                });
                if let Some(msg) = &self.status_message {
                    ui.label(RichText::new(msg).small().color(Color32::LIGHT_GRAY));
                }
            });

        if self.show_logs {
            self.render_logs_panel(ctx);
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG_BASE)
                    .inner_margin(Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                if !self.engine.warnings.is_empty() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(56, 34, 20))
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, COLOR_WARN))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            for warn in &self.engine.warnings {
                                ui.label(RichText::new(format!("⚠ {}", warn)).color(COLOR_WARN));
                            }
                        });
                    ui.add_space(8.0);
                }

                ui.label(
                    RichText::new(
                        "Click any shortcut button to record keys from your keyboard, or click Edit to choose modifiers & Media/Volume keys.",
                    )
                    .small()
                    .color(TEXT_MUTED),
                );
                ui.add_space(8.0);

                let query = self.search_query.trim().to_lowercase();
                let mut any_changed = false;
                let mut delete_id: Option<Uuid> = None;
                let mut test_combo: Option<KeyCombo> = None;
                let mut next_recording = self.recording;

                egui::ScrollArea::vertical()
                    .id_salt("rules_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for rule in &mut self.engine.config.rules {
                            if !query.is_empty() {
                                let matches_name = rule.name.to_lowercase().contains(&query);
                                let matches_trigger =
                                    rule.trigger.to_string().to_lowercase().contains(&query);
                                let matches_target =
                                    rule.target.to_string().to_lowercase().contains(&query);
                                if !matches_name && !matches_trigger && !matches_target {
                                    continue;
                                }
                            }

                            let row_id = rule.id;
                            let card_stroke = if rule.enabled {
                                Stroke::new(1.0_f32, BORDER_SUBTLE)
                            } else {
                                Stroke::new(1.0_f32, Color32::from_rgb(36, 41, 54))
                            };

                            egui::Frame::none()
                                .fill(BG_CARD)
                                .rounding(Rounding::same(8.0))
                                .stroke(card_stroke)
                                .inner_margin(Margin::symmetric(12.0, 9.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        any_changed |=
                                            ui.checkbox(&mut rule.enabled, "").changed();

                                        let name_resp = ui.add(
                                            egui::TextEdit::singleline(&mut rule.name)
                                                .desired_width(140.0),
                                        );
                                        if name_resp.lost_focus()
                                            || (name_resp.changed()
                                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                        {
                                            any_changed = true;
                                        }

                                        let is_rec_trigger = next_recording
                                            == Some(RecordingTarget::Trigger(row_id));
                                        let mut toggle_trigger_rec = false;
                                        any_changed |= Self::render_combo_editor(
                                            ui,
                                            &format!("trig_{}", row_id),
                                            &mut rule.trigger,
                                            is_rec_trigger,
                                            &mut toggle_trigger_rec,
                                        );
                                        if toggle_trigger_rec {
                                            next_recording = if is_rec_trigger {
                                                None
                                            } else {
                                                Some(RecordingTarget::Trigger(row_id))
                                            };
                                        }

                                        ui.label(
                                            RichText::new("→")
                                                .strong()
                                                .size(16.0)
                                                .color(COLOR_INFO),
                                        );

                                        let is_rec_target = next_recording
                                            == Some(RecordingTarget::Target(row_id));
                                        let mut toggle_target_rec = false;
                                        any_changed |= Self::render_combo_editor(
                                            ui,
                                            &format!("targ_{}", row_id),
                                            &mut rule.target,
                                            is_rec_target,
                                            &mut toggle_target_rec,
                                        );
                                        if toggle_target_rec {
                                            next_recording = if is_rec_target {
                                                None
                                            } else {
                                                Some(RecordingTarget::Target(row_id))
                                            };
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .button(
                                                        RichText::new("Delete").color(COLOR_PAUSED),
                                                    )
                                                    .clicked()
                                                {
                                                    delete_id = Some(row_id);
                                                }
                                                if ui
                                                    .button("Test")
                                                    .on_hover_text(
                                                        "Simulate sending the target key combo now",
                                                    )
                                                    .clicked()
                                                {
                                                    test_combo = Some(rule.target);
                                                }
                                            },
                                        );
                                    });
                                });
                            ui.add_space(6.0);
                        }
                    });

                self.recording = next_recording;

                if let Some(id) = delete_id {
                    self.engine.config.rules.retain(|r| r.id != id);
                    log::info!("Deleted keybind rule id={}", id);
                    if self.recording == Some(RecordingTarget::Trigger(id))
                        || self.recording == Some(RecordingTarget::Target(id))
                    {
                        self.recording = None;
                    }
                    any_changed = true;
                }

                if any_changed {
                    self.persist_and_sync();
                }

                if let Some(target) = test_combo {
                    match self.engine.test_rule_output(&target) {
                        Ok(()) => {
                            self.status_message = Some(format!("Sent test key combo: {}", target));
                        }
                        Err(err) => {
                            self.status_message =
                                Some(format!("Failed to send test key combo: {}", err));
                        }
                    }
                }
            });
    }
}
