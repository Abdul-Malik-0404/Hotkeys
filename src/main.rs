#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod backend;
pub mod config;
pub mod engine;
pub mod gui;
pub mod logger;
pub mod model;
pub mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::HotkeyEngine;
use gui::HotkeysApp;
use tray::TrayController;

fn main() -> Result<()> {
    let log_path = logger::init_logger();
    log::info!(
        "Starting Hotkeys v{} on {} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            log::warn!("Falling back to default config due to load error: {}", err);
            AppConfig::default()
        }
    };
    let initial_enabled = config.global_enabled;
    let start_minimized = config.start_minimized;

    let platform_backend = backend::create_default_backend();
    let engine = HotkeyEngine::new(config, platform_backend)?;
    let tray = TrayController::spawn(initial_enabled);

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([920.0, 660.0])
        .with_min_inner_size([740.0, 440.0])
        .with_title("Hotkeys — Keybind & Media Shortcut Manager");

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
        Box::new(move |cc| {
            gui::configure_visuals(&cc.egui_ctx);
            tray.attach_repaint_ctx(cc.egui_ctx.clone());
            Ok(Box::new(HotkeysApp::new(engine, tray, log_path)))
        }),
    )
    .map_err(|err| anyhow::anyhow!("Failed to launch eframe GUI: {}", err))?;

    log::info!("Hotkeys exiting cleanly");
    Ok(())
}
