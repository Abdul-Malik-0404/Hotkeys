use std::collections::VecDeque;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use eframe::egui;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::PlatformBackend;
use crate::model::{KeyCombo, KeybindRule, Modifiers};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComboEnvelope {
    combo: KeyCombo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiToDaemon {
    Sync { effective_enabled: bool },
    TestCombo(KeyCombo),
    QuitDaemon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonToGui {
    SyncStatus(String),
    SyncWarn(String),
    SyncDone,
    TestOk,
    TestErr(String),
    Fired(Uuid),
    GlobalEnabled(bool),
    Focus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiControlEvent {
    Focus,
    SetGlobalEnabled(bool),
}

fn sanitize_single_line(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

pub fn encode_gui_msg(msg: &GuiToDaemon) -> String {
    match msg {
        GuiToDaemon::Sync { effective_enabled } => format!("SYNC|{}\n", effective_enabled),
        GuiToDaemon::TestCombo(combo) => {
            let encoded = toml::to_string(&ComboEnvelope { combo: *combo })
                .unwrap_or_default()
                .replace('\n', ";");
            format!("TEST|{}\n", encoded)
        }
        GuiToDaemon::QuitDaemon => "QUIT\n".to_string(),
    }
}

pub fn decode_gui_msg(line: &str) -> Option<GuiToDaemon> {
    let trimmed = line.trim();
    if trimmed == "QUIT" {
        return Some(GuiToDaemon::QuitDaemon);
    }
    if let Some(rest) = trimmed.strip_prefix("SYNC|") {
        let effective_enabled = rest.trim() == "true";
        return Some(GuiToDaemon::Sync { effective_enabled });
    }
    if let Some(rest) = trimmed.strip_prefix("TEST|") {
        let toml_str = rest.replace(';', "\n");
        if let Ok(env) = toml::from_str::<ComboEnvelope>(&toml_str) {
            return Some(GuiToDaemon::TestCombo(env.combo));
        }
    }
    None
}

pub fn encode_daemon_msg(msg: &DaemonToGui) -> String {
    match msg {
        DaemonToGui::SyncStatus(status) => format!("STATUS|{}\n", sanitize_single_line(status)),
        DaemonToGui::SyncWarn(warn) => format!("WARN|{}\n", sanitize_single_line(warn)),
        DaemonToGui::SyncDone => "SYNC_DONE\n".to_string(),
        DaemonToGui::TestOk => "TEST_OK\n".to_string(),
        DaemonToGui::TestErr(err) => format!("TEST_ERR|{}\n", sanitize_single_line(err)),
        DaemonToGui::Fired(id) => format!("FIRED|{}\n", id),
        DaemonToGui::GlobalEnabled(enabled) => format!("GLOBAL|{}\n", enabled),
        DaemonToGui::Focus => "FOCUS\n".to_string(),
    }
}

pub fn decode_daemon_msg(line: &str) -> Option<DaemonToGui> {
    let trimmed = line.trim();
    if trimmed == "SYNC_DONE" {
        return Some(DaemonToGui::SyncDone);
    }
    if trimmed == "TEST_OK" {
        return Some(DaemonToGui::TestOk);
    }
    if trimmed == "FOCUS" {
        return Some(DaemonToGui::Focus);
    }
    if let Some(rest) = trimmed.strip_prefix("STATUS|") {
        return Some(DaemonToGui::SyncStatus(rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("WARN|") {
        return Some(DaemonToGui::SyncWarn(rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("TEST_ERR|") {
        return Some(DaemonToGui::TestErr(rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("FIRED|") {
        if let Ok(id) = Uuid::parse_str(rest.trim()) {
            return Some(DaemonToGui::Fired(id));
        }
    }
    if let Some(rest) = trimmed.strip_prefix("GLOBAL|") {
        return Some(DaemonToGui::GlobalEnabled(rest.trim() == "true"));
    }
    None
}

pub struct DaemonProxyHandle {
    pub control_rx: Receiver<GuiControlEvent>,
    pub repaint_ctx: Arc<Mutex<Option<egui::Context>>>,
    stdout_lock: Arc<Mutex<io::Stdout>>,
}

impl DaemonProxyHandle {
    pub fn attach_repaint_ctx(&self, ctx: egui::Context) {
        if let Ok(mut guard) = self.repaint_ctx.lock() {
            *guard = Some(ctx);
        }
    }

    pub fn send_quit_daemon(&self) {
        if let Ok(mut out) = self.stdout_lock.lock() {
            let _ = out.write_all(encode_gui_msg(&GuiToDaemon::QuitDaemon).as_bytes());
            let _ = out.flush();
        }
    }
}

pub struct DaemonProxyBackend {
    stdout_lock: Arc<Mutex<io::Stdout>>,
    sync_rx: Receiver<(String, Vec<String>)>,
    test_rx: Receiver<Result<(), String>>,
    fired_queue: Arc<Mutex<VecDeque<Uuid>>>,
    status: String,
}

impl DaemonProxyBackend {
    pub fn connect() -> (Self, DaemonProxyHandle) {
        let stdout_lock = Arc::new(Mutex::new(io::stdout()));
        let (sync_tx, sync_rx) = mpsc::channel();
        let (test_tx, test_rx) = mpsc::channel();
        let (control_tx, control_rx) = mpsc::channel();
        let fired_queue = Arc::new(Mutex::new(VecDeque::new()));
        let repaint_ctx: Arc<Mutex<Option<egui::Context>>> = Arc::new(Mutex::new(None));

        let fired_ref = Arc::clone(&fired_queue);
        let ctx_ref = Arc::clone(&repaint_ctx);

        thread::Builder::new()
            .name("gui-ipc-reader".to_string())
            .spawn(move || {
                let stdin = io::stdin();
                let mut current_status = String::from("Connected to Hotkeys Daemon");
                let mut current_warnings = Vec::new();

                for line in stdin.lock().lines().map_while(Result::ok) {
                    let Some(msg) = decode_daemon_msg(&line) else {
                        continue;
                    };
                    match msg {
                        DaemonToGui::SyncStatus(s) => {
                            current_status = s;
                        }
                        DaemonToGui::SyncWarn(w) => {
                            current_warnings.push(w);
                        }
                        DaemonToGui::SyncDone => {
                            let warns = std::mem::take(&mut current_warnings);
                            let _ = sync_tx.send((current_status.clone(), warns));
                        }
                        DaemonToGui::TestOk => {
                            let _ = test_tx.send(Ok(()));
                        }
                        DaemonToGui::TestErr(e) => {
                            let _ = test_tx.send(Err(e));
                        }
                        DaemonToGui::Fired(id) => {
                            if let Ok(mut q) = fired_ref.lock() {
                                q.push_back(id);
                            }
                            if let Ok(guard) = ctx_ref.lock() {
                                if let Some(ctx) = &*guard {
                                    ctx.request_repaint();
                                }
                            }
                        }
                        DaemonToGui::GlobalEnabled(enabled) => {
                            let _ = control_tx.send(GuiControlEvent::SetGlobalEnabled(enabled));
                            if let Ok(guard) = ctx_ref.lock() {
                                if let Some(ctx) = &*guard {
                                    ctx.request_repaint();
                                }
                            }
                        }
                        DaemonToGui::Focus => {
                            let _ = control_tx.send(GuiControlEvent::Focus);
                            if let Ok(guard) = ctx_ref.lock() {
                                if let Some(ctx) = &*guard {
                                    ctx.request_repaint();
                                }
                            }
                        }
                    }
                }
            })
            .ok();

        let backend = Self {
            stdout_lock: Arc::clone(&stdout_lock),
            sync_rx,
            test_rx,
            fired_queue,
            status: "Connected to Hotkeys Daemon".to_string(),
        };

        let handle = DaemonProxyHandle {
            control_rx,
            repaint_ctx,
            stdout_lock,
        };

        (backend, handle)
    }
}

impl PlatformBackend for DaemonProxyBackend {
    fn apply_rules(&mut self, _rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        if let Ok(mut out) = self.stdout_lock.lock() {
            let msg = encode_gui_msg(&GuiToDaemon::Sync {
                effective_enabled: global_enabled,
            });
            out.write_all(msg.as_bytes())?;
            out.flush()?;
        }

        if let Ok((status, warnings)) = self.sync_rx.recv_timeout(std::time::Duration::from_secs(2))
        {
            self.status = status;
            return Ok(warnings);
        }

        Ok(Vec::new())
    }

    fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>> {
        let mut fired = Vec::new();
        if let Ok(mut q) = self.fired_queue.lock() {
            while let Some(id) = q.pop_front() {
                fired.push(id);
            }
        }
        Ok(fired)
    }

    fn send_combo(
        &mut self,
        combo: &KeyCombo,
        _held_trigger_mods: Option<Modifiers>,
    ) -> Result<()> {
        if let Ok(mut out) = self.stdout_lock.lock() {
            let msg = encode_gui_msg(&GuiToDaemon::TestCombo(*combo));
            out.write_all(msg.as_bytes())?;
            out.flush()?;
        }
        if let Ok(res) = self.test_rx.recv_timeout(std::time::Duration::from_secs(2)) {
            return res.map_err(|e| anyhow::anyhow!(e));
        }
        Ok(())
    }

    fn status_summary(&self) -> String {
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LogicalKey;

    #[test]
    fn test_gui_to_daemon_roundtrip() {
        let combo = KeyCombo::new(Modifiers::new(true, true, false, false), LogicalKey::F8);
        let msg = GuiToDaemon::TestCombo(combo);
        let encoded = encode_gui_msg(&msg);
        assert_eq!(decode_gui_msg(&encoded), Some(msg));

        let sync = GuiToDaemon::Sync {
            effective_enabled: true,
        };
        assert_eq!(decode_gui_msg(&encode_gui_msg(&sync)), Some(sync));
    }

    #[test]
    fn test_daemon_to_gui_roundtrip() {
        let id = Uuid::new_v4();
        let fired = DaemonToGui::Fired(id);
        assert_eq!(decode_daemon_msg(&encode_daemon_msg(&fired)), Some(fired));

        let status = DaemonToGui::SyncStatus("Windows Win32 Thread Backend (tid=123)".to_string());
        assert_eq!(decode_daemon_msg(&encode_daemon_msg(&status)), Some(status));
    }
}
