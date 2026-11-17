use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;

use anyhow::Result;
use uuid::Uuid;

use crate::backend::{keymap::to_win32_vk, PlatformBackend};
use crate::model::{KeyCombo, KeybindRule, Modifiers};

const WM_HOTKEY: u32 = 0x0312;
const PM_REMOVE: u32 = 0x0001;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

const INPUT_KEYBOARD: u32 = 1;
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
    hwnd: *mut core::ffi::c_void,
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

#[cfg_attr(target_os = "windows", link(name = "user32"))]
extern "system" {
    fn RegisterHotKey(hwnd: *mut core::ffi::c_void, id: i32, fs_modifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hwnd: *mut core::ffi::c_void, id: i32) -> i32;
    fn PeekMessageW(
        lp_msg: *mut Msg,
        hwnd: *mut core::ffi::c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> i32;
    fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
}

pub struct WindowsBackend {
    registered_ids: HashMap<i32, (Uuid, KeyCombo, Modifiers)>,
    global_enabled: bool,
}

impl WindowsBackend {
    pub fn new() -> Self {
        Self {
            registered_ids: HashMap::new(),
            global_enabled: true,
        }
    }

    fn unregister_all(&mut self) {
        for &id in self.registered_ids.keys() {
            unsafe {
                UnregisterHotKey(null_mut(), id);
            }
        }
        self.registered_ids.clear();
    }

    fn key_event(vk: u16, key_up: bool) -> Input {
        Input {
            r#type: INPUT_KEYBOARD,
            u: InputUnion {
                ki: KeybdInput {
                    w_vk: vk,
                    w_scan: 0,
                    dw_flags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }

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
        self.unregister_all();
    }
}

impl PlatformBackend for WindowsBackend {
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        self.unregister_all();
        self.global_enabled = global_enabled;
        let mut warnings = Vec::new();

        if !global_enabled {
            return Ok(warnings);
        }

        for (index, rule) in rules.iter().filter(|r| r.enabled).enumerate() {
            let hotkey_id = (index as i32) + 1;
            let fs_mods = Self::win32_modifiers(rule.trigger.modifiers);
            let vk = to_win32_vk(rule.trigger.key) as u32;

            let ok = unsafe { RegisterHotKey(null_mut(), hotkey_id, fs_mods, vk) };
            if ok == 0 {
                warnings.push(format!(
                    "Failed to register '{}' for rule '{}' (may already be in use by Windows or another rule)",
                    rule.trigger, rule.name
                ));
            } else {
                self.registered_ids
                    .insert(hotkey_id, (rule.id, rule.target, rule.trigger.modifiers));
            }
        }

        Ok(warnings)
    }

    fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>> {
        let mut triggered = Vec::new();
        let mut msg: Msg = unsafe { zeroed() };

        while unsafe { PeekMessageW(&mut msg, null_mut(), WM_HOTKEY, WM_HOTKEY, PM_REMOVE) } != 0 {
            if msg.message == WM_HOTKEY && self.global_enabled {
                let id = msg.w_param as i32;
                if let Some(&(rule_id, target, trigger_mods)) = self.registered_ids.get(&id) {
                    triggered.push((rule_id, target, trigger_mods));
                }
            }
        }

        let mut fired_ids = Vec::new();
        for (rule_id, target, trigger_mods) in triggered {
            self.send_combo(&target, Some(trigger_mods))?;
            fired_ids.push(rule_id);
        }

        Ok(fired_ids)
    }

    fn send_combo(&mut self, combo: &KeyCombo, held_trigger_mods: Option<Modifiers>) -> Result<()> {
        let mut inputs = Vec::with_capacity(12);

        if let Some(held) = held_trigger_mods {
            if held.ctrl && !combo.modifiers.ctrl {
                inputs.push(Self::key_event(VK_CONTROL, true));
            }
            if held.alt && !combo.modifiers.alt {
                inputs.push(Self::key_event(VK_MENU, true));
            }
            if held.shift && !combo.modifiers.shift {
                inputs.push(Self::key_event(VK_SHIFT, true));
            }
            if held.meta && !combo.modifiers.meta {
                inputs.push(Self::key_event(VK_LWIN, true));
            }
        }

        if combo.modifiers.ctrl {
            inputs.push(Self::key_event(VK_CONTROL, false));
        }
        if combo.modifiers.alt {
            inputs.push(Self::key_event(VK_MENU, false));
        }
        if combo.modifiers.shift {
            inputs.push(Self::key_event(VK_SHIFT, false));
        }
        if combo.modifiers.meta {
            inputs.push(Self::key_event(VK_LWIN, false));
        }

        let vk = to_win32_vk(combo.key);
        inputs.push(Self::key_event(vk, false));
        inputs.push(Self::key_event(vk, true));

        if combo.modifiers.meta {
            inputs.push(Self::key_event(VK_LWIN, true));
        }
        if combo.modifiers.shift {
            inputs.push(Self::key_event(VK_SHIFT, true));
        }
        if combo.modifiers.alt {
            inputs.push(Self::key_event(VK_MENU, true));
        }
        if combo.modifiers.ctrl {
            inputs.push(Self::key_event(VK_CONTROL, true));
        }

        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<Input>() as i32,
            )
        };
        if sent == 0 {
            anyhow::bail!("Win32 SendInput failed to inject key events");
        }
        Ok(())
    }

    fn status_summary(&self) -> String {
        format!(
            "Windows Win32 Backend (RegisterHotKey + SendInput — {} active shortcut(s))",
            self.registered_ids.len()
        )
    }
}
