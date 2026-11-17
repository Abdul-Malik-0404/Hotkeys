use std::time::Duration;

use eframe::egui::{self, Color32, RichText};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::engine::HotkeyEngine;
use crate::model::{KeyCombo, KeybindRule, LogicalKey, Modifiers};
use crate::tray::{TrayCommand, TrayController};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordingTarget {
    Trigger(Uuid),
    Target(Uuid),
}

pub struct HotkeysApp {
    engine: HotkeyEngine,
    tray: TrayController,
    recording: Option<RecordingTarget>,
    status_message: Option<String>,
    config_path_display: String,
    allow_quit: bool,
}

impl HotkeysApp {
    pub fn new(engine: HotkeyEngine, tray: TrayController) -> Self {
        let config_path_display = AppConfig::default_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "config.toml".to_string());

        Self {
            engine,
            tray,
            recording: None,
            status_message: None,
            config_path_display,
            allow_quit: false,
        }
    }

    fn persist_and_sync(&mut self) {
        if let Err(err) = self.engine.sync_backend() {
            self.status_message = Some(format!("Backend sync error: {}", err));
        }
        match self.engine.config.save() {
            Ok(()) => {
                self.status_message = Some("Saved configuration.".to_string());
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to save config: {}", err));
            }
        }
        self.tray
            .set_global_enabled(self.engine.config.global_enabled);
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
                        rule.trigger = combo;
                    }
                }
                RecordingTarget::Target(id) => {
                    if let Some(rule) = self.engine.config.rules.iter_mut().find(|r| r.id == id) {
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
        while let Some(cmd) = self.tray.try_recv() {
            match cmd {
                TrayCommand::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayCommand::ToggleGlobalEnabled => {
                    self.engine.config.global_enabled = !self.engine.config.global_enabled;
                    self.persist_and_sync();
                }
                TrayCommand::Quit => {
                    self.allow_quit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
            let btn_label = if is_recording {
                RichText::new("Press keys... (Esc)")
                    .color(Color32::from_rgb(255, 200, 80))
                    .strong()
            } else {
                RichText::new(format!("[ {} ]", combo)).monospace().strong()
            };

            if ui
                .button(btn_label)
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
                    changed |= ui.checkbox(&mut combo.modifiers.meta, "Super").changed();
                });
                ui.separator();
                ui.label(RichText::new("Key (including Media/Volume)").strong());
                egui::ComboBox::from_id_salt(id_salt)
                    .selected_text(format!("{} ({})", combo.key.label(), combo.key.category()))
                    .width(220.0)
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
}

impl eframe::App for HotkeysApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll tray commands & background hotkey events at ~60Hz
        self.handle_tray_commands(ctx);
        self.handle_key_recording(ctx);
        if let Err(err) = self.engine.poll() {
            self.status_message = Some(format!("Hotkey dispatch error: {}", err));
        }
        ctx.request_repaint_after(Duration::from_millis(16));

        // Minimize to system tray on window close unless Quit was chosen
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            self.status_message =
                Some("Minimized to background / system tray. Hotkeys remain active.".to_string());
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("Hotkeys Manager");
                ui.separator();

                let mut global_enabled = self.engine.config.global_enabled;
                let toggle_text = if global_enabled {
                    RichText::new("Hotkeys Active")
                        .color(Color32::from_rgb(80, 210, 120))
                        .strong()
                } else {
                    RichText::new("Hotkeys Paused")
                        .color(Color32::from_rgb(230, 110, 100))
                        .strong()
                };
                if ui.checkbox(&mut global_enabled, toggle_text).changed() {
                    self.engine.config.global_enabled = global_enabled;
                    self.persist_and_sync();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("Quit")
                        .on_hover_text("Stop daemon and exit completely")
                        .clicked()
                    {
                        self.allow_quit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if ui.button("Restore Defaults").clicked() {
                        self.engine.config.rules = AppConfig::default_rules();
                        self.recording = None;
                        self.persist_and_sync();
                    }

                    if ui.button("+ Add Keybind").clicked() {
                        let new_rule = KeybindRule::new(
                            format!("Custom Rule {}", self.engine.config.rules.len() + 1),
                            KeyCombo::new(
                                Modifiers::new(true, true, false, false),
                                LogicalKey::F8,
                            ),
                            KeyCombo::bare(LogicalKey::MediaPlayPause),
                        );
                        self.engine.config.rules.push(new_rule);
                        self.persist_and_sync();
                    }
                });
            });
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("status_footer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.engine.backend_status())
                        .small()
                        .color(Color32::GRAY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(last) = &self.engine.last_triggered_rule {
                        ui.label(
                            RichText::new(format!(
                                "Last fired (#{}): {}",
                                self.engine.total_triggers_count, last
                            ))
                            .small()
                            .color(Color32::from_rgb(120, 200, 255)),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!("Config: {}", self.config_path_display))
                                .small()
                                .color(Color32::GRAY),
                        );
                    }
                });
            });
            if let Some(msg) = &self.status_message {
                ui.label(RichText::new(msg).small());
            }
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.engine.warnings.is_empty() {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(60, 35, 20))
                    .show(ui, |ui| {
                        for warn in &self.engine.warnings {
                            ui.label(
                                RichText::new(format!("Warning: {}", warn))
                                    .color(Color32::from_rgb(255, 190, 100)),
                            );
                        }
                    });
                ui.add_space(8.0);
            }

            ui.label(
                "Click any Trigger or Target shortcut button to record a key combination from your keyboard, or click 'Edit' to pick special media/volume keys.",
            );
            ui.add_space(8.0);

            let mut any_changed = false;
            let mut delete_id: Option<Uuid> = None;
            let mut test_combo: Option<KeyCombo> = None;
            let mut next_recording = self.recording;

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("keybinds_grid")
                    .num_columns(6)
                    .striped(true)
                    .spacing([14.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("On").strong());
                        ui.label(RichText::new("Rule Name").strong());
                        ui.label(RichText::new("Trigger Shortcut (Global)").strong());
                        ui.label(RichText::new("->").strong());
                        ui.label(RichText::new("Target Key Combo (Output)").strong());
                        ui.label(RichText::new("Actions").strong());
                        ui.end_row();

                        for rule in &mut self.engine.config.rules {
                            let row_id = rule.id;
                            any_changed |= ui.checkbox(&mut rule.enabled, "").changed();

                            let name_resp = ui.add(
                                egui::TextEdit::singleline(&mut rule.name).desired_width(150.0),
                            );
                            if name_resp.lost_focus()
                                || (name_resp.changed()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                any_changed = true;
                            }

                            let is_rec_trigger =
                                next_recording == Some(RecordingTarget::Trigger(row_id));
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

                            ui.label(RichText::new("->").strong());

                            let is_rec_target =
                                next_recording == Some(RecordingTarget::Target(row_id));
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

                            ui.horizontal(|ui| {
                                if ui
                                    .button("Test")
                                    .on_hover_text("Simulate sending the target key combo now")
                                    .clicked()
                                {
                                    test_combo = Some(rule.target);
                                }
                                if ui
                                    .button(
                                        RichText::new("Delete")
                                            .color(Color32::from_rgb(230, 100, 100)),
                                    )
                                    .clicked()
                                {
                                    delete_id = Some(row_id);
                                }
                            });

                            ui.end_row();
                        }
                    });
            });

            self.recording = next_recording;

            if let Some(id) = delete_id {
                self.engine.config.rules.retain(|r| r.id != id);
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
