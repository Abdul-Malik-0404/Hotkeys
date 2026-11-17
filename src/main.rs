#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod backend;
pub mod config;
pub mod engine;
pub mod gui;
pub mod model;
pub mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::HotkeyEngine;
use gui::HotkeysApp;
use tray::TrayController;

fn main() -> Result<()> {
    env_logger::init();

    let config = AppConfig::load().unwrap_or_default();
    let initial_enabled = config.global_enabled;
    let start_minimized = config.start_minimized;

    let platform_backend = backend::create_default_backend();
    let engine = HotkeyEngine::new(config, platform_backend)?;
    let tray = TrayController::spawn(initial_enabled);

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([820.0, 500.0])
        .with_min_inner_size([680.0, 360.0])
        .with_title("Hotkeys — Cross-Platform Keybind Manager");

    if start_minimized {
        viewport = viewport.with_visible(false);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Hotkeys",
        native_options,
        Box::new(move |_cc| Ok(Box::new(HotkeysApp::new(engine, tray)))),
    )
    .map_err(|err| anyhow::anyhow!("Failed to launch eframe GUI: {}", err))?;

    Ok(())
}
