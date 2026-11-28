#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod backend;
pub mod config;
pub mod daemon;
pub mod engine;
pub mod gui;
pub mod ipc;
pub mod logger;
pub mod model;
pub mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::HotkeyEngine;
use gui::HotkeysApp;
use ipc::DaemonProxyBackend;
use tray::TrayController;

fn run_gui_window(
    engine: HotkeyEngine,
    tray: Option<TrayController>,
    proxy: Option<ipc::DaemonProxyHandle>,
    log_path: std::path::PathBuf,
    start_minimized: bool,
) -> Result<()> {
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
            if let Some(ref t) = tray {
                t.attach_repaint_ctx(cc.egui_ctx.clone());
            }
            if let Some(ref p) = proxy {
                p.attach_repaint_ctx(cc.egui_ctx.clone());
            }
            Ok(Box::new(HotkeysApp::new(engine, tray, proxy, log_path)))
        }),
    )
    .map_err(|err| anyhow::anyhow!("Failed to launch eframe GUI: {}", err))?;

    Ok(())
}

fn run_gui_child() -> Result<()> {
    let log_path = logger::init_logger(false);
    log::info!("Starting Hotkeys GUI window v{}", env!("CARGO_PKG_VERSION"));
    let config = AppConfig::load().unwrap_or_default();
    let (proxy_backend, proxy_handle) = DaemonProxyBackend::connect();
    let engine = HotkeyEngine::new(config, Box::new(proxy_backend))?;
    run_gui_window(engine, None, Some(proxy_handle), log_path, false)?;
    log::info!("Hotkeys GUI window closed");
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let gui_only = args.iter().any(|a| a == "--gui");
    let start_minimized_arg = args.iter().any(|a| a == "--minimized");

    if gui_only {
        return run_gui_child();
    }

    let log_path = logger::init_logger(true);
    log::info!(
        "Starting Hotkeys v{} on {} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    #[cfg(target_os = "windows")]
    {
        let _ = log_path;
        daemon::run_daemon(start_minimized_arg)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let config = match AppConfig::load() {
            Ok(cfg) => cfg,
            Err(err) => {
                log::warn!("Falling back to default config due to load error: {}", err);
                AppConfig::default()
            }
        };
        let initial_enabled = config.global_enabled;
        let start_minimized = start_minimized_arg || config.start_minimized;

        let platform_backend = backend::create_default_backend();
        let engine = HotkeyEngine::new(config, platform_backend)?;
        let tray = TrayController::spawn(initial_enabled);
        run_gui_window(engine, Some(tray), None, log_path, start_minimized)?;
        log::info!("Hotkeys exiting cleanly");
        Ok(())
    }
}
