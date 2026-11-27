#[cfg(target_os = "windows")]
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use std::mem::{size_of, zeroed};
#[cfg(target_os = "windows")]
use std::ptr::null_mut;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::Result;
use uuid::Uuid;

use crate::backend::{keymap::to_win32_vk, PlatformBackend};
use crate::model::{KeyCombo, KeybindRule, LogicalKey, Modifiers};

const WM_HOTKEY: u32 = 0x0312;
const WM_QUIT: u32 = 0x0012;
const WM_APP_WAKE: u32 = 0x8001;
const PM_NOREMOVE: u32 = 0x0000;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
const KEYEVENTF_KEYUP: u32 = 0x0002;

const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12; // Alt
const VK_LWIN: u16 = 0x5B;

#[repr(C)]
#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Msg {
    hwnd: *mut c_void,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: Point,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KeybdInput {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
union InputUnion {
    mi: MouseInput,
    ki: KeybdInput,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Input {
    r#type: u32,
    u: InputUnion,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentThreadId() -> u32;
    fn GetLastError() -> u32;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn RegisterHotKey(hwnd: *mut c_void, id: i32, fs_modifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hwnd: *mut c_void, id: i32) -> i32;
    fn GetMessageW(
        lp_msg: *mut Msg,
        hwnd: *mut c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
    ) -> i32;
    fn PeekMessageW(
        lp_msg: *mut Msg,
        hwnd: *mut c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> i32;
    fn PostThreadMessageW(id_thread: u32, msg: u32, w_param: usize, l_param: isize) -> i32;
    fn GetAsyncKeyState(v_key: i32) -> i16;
    fn MapVirtualKeyW(u_code: u32, u_map_type: u32) -> u32;
    fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
}

enum WorkerCommand {
    ApplyRules {
        rules: Vec<KeybindRule>,
        global_enabled: bool,
        reply_tx: Sender<(Vec<String>, usize)>,
    },
    SendCombo {
        combo: KeyCombo,
        held_trigger_mods: Option<Modifiers>,
        reply_tx: Sender<Result<()>>,
    },
    Shutdown,
}

pub struct WindowsBackend {
    cmd_tx: Sender<WorkerCommand>,
    worker_thread_id: u32,
    worker_handle: Option<JoinHandle<()>>,
    fired_queue: Arc<Mutex<VecDeque<Uuid>>>,
    active_count: usize,
}

impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsBackend {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (tid_tx, tid_rx) = mpsc::channel();
        let fired_queue = Arc::new(Mutex::new(VecDeque::new()));
        let worker_fired = Arc::clone(&fired_queue);

        let worker_handle = thread::Builder::new()
            .name("win32-hotkey-pump".to_string())
            .spawn(move || {
                Self::run_worker(cmd_rx, tid_tx, worker_fired);
            })
            .ok();

        let worker_thread_id = tid_rx.recv().unwrap_or(0);
        log::info!(
            "WindowsBackend initialized with dedicated message-pump thread (tid={})",
            worker_thread_id
        );

        Self {
            cmd_tx,
            worker_thread_id,
            worker_handle,
            fired_queue,
            active_count: 0,
        }
    }

    fn wake_worker(&self) {
        #[cfg(target_os = "windows")]
        if self.worker_thread_id != 0 {
            let ok = unsafe { PostThreadMessageW(self.worker_thread_id, WM_APP_WAKE, 0, 0) };
            if ok == 0 {
                log::warn!("PostThreadMessageW(WM_APP_WAKE) failed (err={})", unsafe {
                    GetLastError()
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn run_worker(
        cmd_rx: Receiver<WorkerCommand>,
        tid_tx: Sender<u32>,
        fired_queue: Arc<Mutex<VecDeque<Uuid>>>,
    ) {
        let mut msg: Msg = unsafe { zeroed() };
        // Force the OS to create the thread's Win32 message queue before exposing its thread ID.
        unsafe {
            PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
        }
        let tid = unsafe { GetCurrentThreadId() };
        let _ = tid_tx.send(tid);
        log::info!("Win32 hotkey worker thread started (tid={})", tid);

        let mut registered: HashMap<i32, (Uuid, String, KeyCombo, KeyCombo)> = HashMap::new();
        let mut global_enabled = true;

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    WorkerCommand::ApplyRules {
                        rules,
                        global_enabled: enabled,
                        reply_tx,
                    } => {
                        global_enabled = enabled;
                        let warnings = Self::worker_apply_rules(&mut registered, &rules, enabled);
                        let _ = reply_tx.send((warnings, registered.len()));
                    }
                    WorkerCommand::SendCombo {
                        combo,
                        held_trigger_mods,
                        reply_tx,
                    } => {
                        let res = Self::send_combo_win32(&combo, held_trigger_mods);
                        let _ = reply_tx.send(res);
                    }
                    WorkerCommand::Shutdown => {
                        Self::worker_unregister_all(&mut registered);
                        log::info!("Win32 hotkey worker shutting down");
                        return;
                    }
                }
            }

            let ret = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if ret <= 0 {
                if ret < 0 {
                    log::error!("GetMessageW returned -1 (err={})", unsafe {
                        GetLastError()
                    });
                }
                break;
            }

            match msg.message {
                WM_HOTKEY => {
                    let id = msg.w_param as i32;
                    if global_enabled {
                        if let Some((rule_id, name, trigger, target)) = registered.get(&id).cloned()
                        {
                            log::info!(
                                "WM_HOTKEY received (id={}, rule='{}', {} -> {})",
                                id,
                                name,
                                trigger,
                                target
                            );
                            if let Err(err) =
                                Self::send_combo_win32(&target, Some(trigger.modifiers))
                            {
                                log::error!(
                                    "Failed to dispatch hotkey '{}' ({} -> {}): {}",
                                    name,
                                    trigger,
                                    target,
                                    err
                                );
                            } else if let Ok(mut q) = fired_queue.lock() {
                                q.push_back(rule_id);
                            }
                        } else {
                            log::warn!("WM_HOTKEY received for unknown id={}", id);
                        }
                    }
                }
                WM_APP_WAKE => {
                    // Drain commands on the next iteration at top of loop.
                }
                WM_QUIT => {
                    break;
                }
                _ => {}
            }
        }

        Self::worker_unregister_all(&mut registered);
    }

    #[cfg(not(target_os = "windows"))]
    fn run_worker(
        cmd_rx: Receiver<WorkerCommand>,
        tid_tx: Sender<u32>,
        _fired_queue: Arc<Mutex<VecDeque<Uuid>>>,
    ) {
        let _ = tid_tx.send(0);
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                WorkerCommand::ApplyRules { reply_tx, .. } => {
                    let _ = reply_tx.send((Vec::new(), 0));
                }
                WorkerCommand::SendCombo { reply_tx, .. } => {
                    let _ = reply_tx.send(Ok(()));
                }
                WorkerCommand::Shutdown => break,
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn worker_unregister_all(registered: &mut HashMap<i32, (Uuid, String, KeyCombo, KeyCombo)>) {
        if !registered.is_empty() {
            log::debug!(
                "Unregistering {} active Windows hotkey(s)",
                registered.len()
            );
        }
        for &id in registered.keys() {
            unsafe {
                UnregisterHotKey(null_mut(), id);
            }
        }
        registered.clear();
    }

    #[cfg(target_os = "windows")]
    fn worker_apply_rules(
        registered: &mut HashMap<i32, (Uuid, String, KeyCombo, KeyCombo)>,
        rules: &[KeybindRule],
        global_enabled: bool,
    ) -> Vec<String> {
        Self::worker_unregister_all(registered);
        let mut warnings = Vec::new();

        if !global_enabled {
            log::info!("Global hotkeys paused; 0 shortcuts registered");
            return warnings;
        }

        for (index, rule) in rules.iter().filter(|r| r.enabled).enumerate() {
            let hotkey_id = (index as i32) + 1;
            let fs_mods = Self::win32_modifiers(rule.trigger.modifiers);
            let vk = to_win32_vk(rule.trigger.key) as u32;

            let ok = unsafe { RegisterHotKey(null_mut(), hotkey_id, fs_mods, vk) };
            if ok == 0 {
                let err_code = unsafe { GetLastError() };
                let msg = format!(
                    "Failed to register '{}' for rule '{}' (Win32 error {})",
                    rule.trigger, rule.name, err_code
                );
                log::warn!("{}", msg);
                warnings.push(msg);
            } else {
                log::info!(
                    "Registered hotkey id={}: '{}' ({} -> {})",
                    hotkey_id,
                    rule.name,
                    rule.trigger,
                    rule.target
                );
                registered.insert(
                    hotkey_id,
                    (rule.id, rule.name.clone(), rule.trigger, rule.target),
                );
            }
        }

        warnings
    }

    pub fn is_media_or_volume_key(key: LogicalKey) -> bool {
        matches!(
            key,
            LogicalKey::MediaPlayPause
                | LogicalKey::MediaNextTrack
                | LogicalKey::MediaPrevTrack
                | LogicalKey::MediaStop
                | LogicalKey::VolumeUp
                | LogicalKey::VolumeDown
                | LogicalKey::VolumeMute
        )
    }

    pub fn is_extended_vk(vk: u16) -> bool {
        matches!(
            vk,
            0x21..=0x28 // PageUp, PageDown, End, Home, ArrowLeft..ArrowDown
                | 0x2C // PrintScreen
                | 0x2D // Insert
                | 0x2E // Delete
                | 0x5B // VK_LWIN
                | 0x5D // VK_APPS (ContextMenu)
                | 0xAD..=0xB3 // Volume & Media keys
        )
    }

    #[allow(dead_code)]
    fn build_key_event(vk: u16, scan: u16, key_up: bool) -> Input {
        let mut flags = if Self::is_extended_vk(vk) {
            KEYEVENTF_EXTENDEDKEY
        } else {
            0
        };
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        Input {
            r#type: INPUT_KEYBOARD,
            u: InputUnion {
                ki: KeybdInput {
                    w_vk: vk,
                    w_scan: scan,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }

    #[cfg(target_os = "windows")]
    fn is_vk_physically_down(vk: u16) -> bool {
        (unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000) != 0
    }

    #[cfg(target_os = "windows")]
    fn send_combo_win32(combo: &KeyCombo, held_trigger_mods: Option<Modifiers>) -> Result<()> {
        let mut inputs = Vec::with_capacity(12);
        let target_is_bare_media =
            Self::is_media_or_volume_key(combo.key) && combo.modifiers.is_empty();

        let make_event = |vk: u16, key_up: bool| {
            let scan = unsafe { MapVirtualKeyW(vk as u32, 0) } as u16;
            Self::build_key_event(vk, scan, key_up)
        };

        // For bare media/volume keys (Play/Pause, Vol Up, etc.), avoid synthesizing fake
        // Alt/Ctrl/Shift key-ups which can steal focus to the foreground window's menu bar.
        if !target_is_bare_media {
            if let Some(held) = held_trigger_mods {
                if held.ctrl && !combo.modifiers.ctrl && Self::is_vk_physically_down(VK_CONTROL) {
                    inputs.push(make_event(VK_CONTROL, true));
                }
                if held.alt && !combo.modifiers.alt && Self::is_vk_physically_down(VK_MENU) {
                    inputs.push(make_event(VK_MENU, true));
                }
                if held.shift && !combo.modifiers.shift && Self::is_vk_physically_down(VK_SHIFT) {
                    inputs.push(make_event(VK_SHIFT, true));
                }
                if held.meta && !combo.modifiers.meta && Self::is_vk_physically_down(VK_LWIN) {
                    inputs.push(make_event(VK_LWIN, true));
                }
            }
        }

        if combo.modifiers.ctrl {
            inputs.push(make_event(VK_CONTROL, false));
        }
        if combo.modifiers.alt {
            inputs.push(make_event(VK_MENU, false));
        }
        if combo.modifiers.shift {
            inputs.push(make_event(VK_SHIFT, false));
        }
        if combo.modifiers.meta {
            inputs.push(make_event(VK_LWIN, false));
        }

        let vk = to_win32_vk(combo.key);
        inputs.push(make_event(vk, false));
        inputs.push(make_event(vk, true));

        if combo.modifiers.meta {
            inputs.push(make_event(VK_LWIN, true));
        }
        if combo.modifiers.shift {
            inputs.push(make_event(VK_SHIFT, true));
        }
        if combo.modifiers.alt {
            inputs.push(make_event(VK_MENU, true));
        }
        if combo.modifiers.ctrl {
            inputs.push(make_event(VK_CONTROL, true));
        }

        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<Input>() as i32,
            )
        };
        if sent == 0 {
            let err = unsafe { GetLastError() };
            log::error!(
                "Win32 SendInput failed for '{}' (vk={:#x}, err={})",
                combo,
                vk,
                err
            );
            anyhow::bail!("Win32 SendInput failed (error {})", err);
        }

        log::info!(
            "SendInput dispatched '{}' ({}/{} event(s), vk={:#x})",
            combo,
            sent,
            inputs.len(),
            vk
        );
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn send_combo_win32(_combo: &KeyCombo, _held_trigger_mods: Option<Modifiers>) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    fn win32_modifiers(mods: Modifiers) -> u32 {
        let mut flags = MOD_NOREPEAT;
        if mods.ctrl {
            flags |= MOD_CONTROL;
        }
        if mods.alt {
            flags |= MOD_ALT;
        }
        if mods.shift {
            flags |= MOD_SHIFT;
        }
        if mods.meta {
            flags |= MOD_WIN;
        }
        flags
    }
}

impl Drop for WindowsBackend {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown);
        self.wake_worker();
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl PlatformBackend for WindowsBackend {
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.cmd_tx
            .send(WorkerCommand::ApplyRules {
                rules: rules.to_vec(),
                global_enabled,
                reply_tx,
            })
            .map_err(|_| anyhow::anyhow!("Win32 hotkey worker thread exited unexpectedly"))?;
        self.wake_worker();

        let (warnings, active_count) = reply_rx
            .recv()
            .map_err(|_| anyhow::anyhow!("Failed to receive apply_rules reply from worker"))?;
        self.active_count = active_count;
        Ok(warnings)
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

    fn send_combo(&mut self, combo: &KeyCombo, held_trigger_mods: Option<Modifiers>) -> Result<()> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.cmd_tx
            .send(WorkerCommand::SendCombo {
                combo: *combo,
                held_trigger_mods,
                reply_tx,
            })
            .map_err(|_| anyhow::anyhow!("Win32 hotkey worker thread exited unexpectedly"))?;
        self.wake_worker();
        reply_rx
            .recv()
            .map_err(|_| anyhow::anyhow!("Failed to receive send_combo reply from worker"))?
    }

    fn status_summary(&self) -> String {
        format!(
            "Windows Win32 Thread Backend (tid={}, {} active shortcut(s))",
            self.worker_thread_id, self.active_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_vk_flags() {
        assert!(WindowsBackend::is_extended_vk(to_win32_vk(
            LogicalKey::MediaPlayPause
        )));
        assert!(WindowsBackend::is_extended_vk(to_win32_vk(
            LogicalKey::VolumeUp
        )));
        assert!(WindowsBackend::is_extended_vk(to_win32_vk(
            LogicalKey::ArrowRight
        )));
        assert!(!WindowsBackend::is_extended_vk(to_win32_vk(LogicalKey::A)));
        assert!(!WindowsBackend::is_extended_vk(to_win32_vk(LogicalKey::F2)));
    }
}
