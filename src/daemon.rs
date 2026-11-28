#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::Result;

use crate::backend;
use crate::config::AppConfig;
use crate::engine::HotkeyEngine;
use crate::ipc::{decode_gui_msg, encode_daemon_msg, DaemonToGui, GuiToDaemon};
use crate::tray::{TrayCommand, TrayController};

struct GuiChild {
    child: Child,
    stdin: ChildStdin,
}

impl GuiChild {
    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn send(&mut self, msg: &DaemonToGui) {
        let encoded = encode_daemon_msg(msg);
        let _ = self.stdin.write_all(encoded.as_bytes());
        let _ = self.stdin.flush();
    }
}

fn spawn_or_focus_gui(
    gui_child: &mut Option<GuiChild>,
    cmd_tx: &Sender<GuiToDaemon>,
    wake_tx: &Sender<()>,
) {
    if let Some(existing) = gui_child.as_mut() {
        if existing.is_alive() {
            log::info!("Focusing already-open Hotkeys GUI window");
            existing.send(&DaemonToGui::Focus);
            return;
        }
    }
    *gui_child = None;

    let Ok(exe) = std::env::current_exe() else {
        log::error!("Failed to locate current executable path for --gui");
        return;
    };

    log::info!("Spawning on-demand Hotkeys GUI process");
    let Ok(mut child) = Command::new(exe)
        .arg("--gui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    else {
        log::error!("Failed to spawn --gui child process");
        return;
    };

    let Some(stdin) = child.stdin.take() else {
        log::error!("Failed to open stdin pipe to --gui process");
        return;
    };

    if let Some(stdout) = child.stdout.take() {
        let tx = cmd_tx.clone();
        let wake = wake_tx.clone();
        thread::Builder::new()
            .name("daemon-gui-reader".to_string())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(msg) = decode_gui_msg(&line) {
                        let _ = tx.send(msg);
                        let _ = wake.send(());
                    }
                }
                log::info!("GUI process closed; daemon remains idle in background");
            })
            .ok();
    }

    *gui_child = Some(GuiChild { child, stdin });
}

fn send_sync_result(gui_child: &mut Option<GuiChild>, engine: &HotkeyEngine) {
    if let Some(child) = gui_child.as_mut() {
        if child.is_alive() {
            child.send(&DaemonToGui::SyncStatus(engine.backend_status()));
            for warn in &engine.warnings {
                child.send(&DaemonToGui::SyncWarn(warn.clone()));
            }
            child.send(&DaemonToGui::SyncDone);
        }
    }
}

#[cfg(target_os = "windows")]
mod single_instance {
    use std::ffi::c_void;
    use std::ptr::null;
    use std::sync::mpsc::Sender;
    use std::thread;

    const ERROR_ALREADY_EXISTS: u32 = 183;
    const INFINITE: u32 = 0xFFFF_FFFF;
    const WAIT_OBJECT_0: u32 = 0;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *const c_void,
            b_initial_owner: i32,
            lp_name: *const u16,
        ) -> *mut c_void;
        fn CreateEventW(
            lp_event_attributes: *const c_void,
            b_manual_reset: i32,
            b_initial_state: i32,
            lp_name: *const u16,
        ) -> *mut c_void;
        fn SetEvent(h_event: *mut c_void) -> i32;
        fn WaitForSingleObject(h_handle: *mut c_void, dw_milliseconds: u32) -> u32;
        fn CloseHandle(h_object: *mut c_void) -> i32;
        fn GetLastError() -> u32;
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub struct InstanceGuard {
        mutex_handle: *mut c_void,
        event_handle: usize,
    }

    impl Drop for InstanceGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.mutex_handle.is_null() {
                    CloseHandle(self.mutex_handle);
                }
                if self.event_handle != 0 {
                    CloseHandle(self.event_handle as *mut c_void);
                }
            }
        }
    }

    pub fn acquire_or_signal_existing() -> Option<InstanceGuard> {
        let mutex_name = wide("Local\\HotkeysDaemonSingleInstance");
        let event_name = wide("Local\\HotkeysShowGuiEvent");

        unsafe {
            let mutex = CreateMutexW(null(), 0, mutex_name.as_ptr());
            let already_exists = GetLastError() == ERROR_ALREADY_EXISTS;
            let event = CreateEventW(null(), 0, 0, event_name.as_ptr());

            if already_exists {
                if !event.is_null() {
                    SetEvent(event);
                    CloseHandle(event);
                }
                if !mutex.is_null() {
                    CloseHandle(mutex);
                }
                return None;
            }

            Some(InstanceGuard {
                mutex_handle: mutex,
                event_handle: event as usize,
            })
        }
    }

    pub fn spawn_show_event_listener(
        guard: &InstanceGuard,
        show_tx: Sender<()>,
        wake_tx: Sender<()>,
    ) {
        let event_val = guard.event_handle;
        if event_val == 0 {
            return;
        }
        thread::Builder::new()
            .name("win32-show-listener".to_string())
            .spawn(move || loop {
                let res = unsafe { WaitForSingleObject(event_val as *mut c_void, INFINITE) };
                if res == WAIT_OBJECT_0 {
                    if show_tx.send(()).is_err() {
                        break;
                    }
                    let _ = wake_tx.send(());
                } else {
                    break;
                }
            })
            .ok();
    }
}

#[cfg(not(target_os = "windows"))]
mod single_instance {
    use std::sync::mpsc::Sender;

    pub struct InstanceGuard;

    pub fn acquire_or_signal_existing() -> Option<InstanceGuard> {
        Some(InstanceGuard)
    }

    pub fn spawn_show_event_listener(
        _guard: &InstanceGuard,
        _show_tx: Sender<()>,
        _wake_tx: Sender<()>,
    ) {
    }
}

pub fn run_daemon(start_minimized: bool) -> Result<()> {
    let Some(guard) = single_instance::acquire_or_signal_existing() else {
        log::info!("Hotkeys daemon is already running; signaled existing instance to open GUI");
        return Ok(());
    };

    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            log::warn!("Falling back to default config due to load error: {}", err);
            AppConfig::default()
        }
    };

    let initial_enabled = config.global_enabled;
    let open_on_start = !start_minimized && !config.start_minimized;

    let (wake_tx, wake_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let (show_tx, show_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let (cmd_tx, cmd_rx): (Sender<GuiToDaemon>, Receiver<GuiToDaemon>) = mpsc::channel();

    let platform_backend = backend::create_default_backend();
    let mut engine = HotkeyEngine::new(config, platform_backend)?;
    engine.set_wake_sender(wake_tx.clone());

    let tray = TrayController::spawn(initial_enabled);
    tray.attach_wake_tx(wake_tx.clone());
    single_instance::spawn_show_event_listener(&guard, show_tx, wake_tx.clone());

    let mut gui_child: Option<GuiChild> = None;
    if open_on_start {
        spawn_or_focus_gui(&mut gui_child, &cmd_tx, &wake_tx);
    } else {
        log::info!("Daemon started minimized in system tray");
    }

    loop {
        if wake_rx.recv().is_err() {
            break;
        }
        while wake_rx.try_recv().is_ok() {}

        if let Ok(fired_ids) = engine.poll() {
            for id in fired_ids {
                if let Some(child) = gui_child.as_mut() {
                    if child.is_alive() {
                        child.send(&DaemonToGui::Fired(id));
                    }
                }
            }
        }

        while show_rx.try_recv().is_ok() {
            spawn_or_focus_gui(&mut gui_child, &cmd_tx, &wake_tx);
        }

        let mut should_quit = false;
        while let Some(tray_cmd) = tray.try_recv() {
            match tray_cmd {
                TrayCommand::ShowWindow => {
                    spawn_or_focus_gui(&mut gui_child, &cmd_tx, &wake_tx);
                }
                TrayCommand::ToggleGlobalEnabled => {
                    engine.config.global_enabled = !engine.config.global_enabled;
                    log::info!(
                        "Toggled global hotkeys from tray -> enabled={}",
                        engine.config.global_enabled
                    );
                    let _ = engine.config.save();
                    let _ = engine.sync_backend();
                    tray.set_global_enabled(engine.config.global_enabled);
                    if let Some(child) = gui_child.as_mut() {
                        if child.is_alive() {
                            child.send(&DaemonToGui::GlobalEnabled(engine.config.global_enabled));
                        }
                    }
                    send_sync_result(&mut gui_child, &engine);
                }
                TrayCommand::Quit => {
                    log::info!("Quit requested from system tray");
                    should_quit = true;
                }
            }
        }

        while let Ok(gui_msg) = cmd_rx.try_recv() {
            match gui_msg {
                GuiToDaemon::Sync { effective_enabled } => {
                    if let Ok(cfg) = AppConfig::load() {
                        engine.config = cfg;
                    }
                    engine.recording_active = !effective_enabled && engine.config.global_enabled;
                    let _ = engine.sync_backend();
                    tray.set_global_enabled(engine.config.global_enabled);
                    send_sync_result(&mut gui_child, &engine);
                }
                GuiToDaemon::TestCombo(combo) => {
                    let reply = match engine.test_rule_output(&combo) {
                        Ok(()) => DaemonToGui::TestOk,
                        Err(err) => DaemonToGui::TestErr(err.to_string()),
                    };
                    if let Some(child) = gui_child.as_mut() {
                        if child.is_alive() {
                            child.send(&reply);
                        }
                    }
                }
                GuiToDaemon::QuitDaemon => {
                    log::info!("Quit requested from GUI process");
                    should_quit = true;
                }
            }
        }

        if should_quit {
            if let Some(mut child) = gui_child.take() {
                if child.is_alive() {
                    let _ = child.child.kill();
                }
            }
            break;
        }
    }

    log::info!("Hotkeys daemon exiting cleanly");
    Ok(())
}
