use std::collections::{HashMap, VecDeque};
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use uuid::Uuid;

use crate::backend::{
    keymap::{to_macos_key, MacOsKeyTarget},
    PlatformBackend,
};
use crate::model::{KeyCombo, KeybindRule, LogicalKey, Modifiers};

type OSStatus = i32;
type EventRef = *mut c_void;
type EventHandlerCallRef = *mut c_void;
type EventHandlerRef = *mut c_void;
type EventTargetRef = *mut c_void;
type EventHotKeyRef = *mut c_void;
type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;
type CGKeyCode = u16;
type CGEventFlags = u64;

const CMD_KEY: u32 = 1 << 8;
const SHIFT_KEY: u32 = 1 << 9;
const OPTION_KEY: u32 = 1 << 11;
const CONTROL_KEY: u32 = 1 << 12;

const K_CG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 0x0002_0000;
const K_CG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 0x0004_0000;
const K_CG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 0x0008_0000;
const K_CG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 0x0010_0000;
const K_CG_HID_EVENT_TAP: u32 = 0;

const K_EVENT_CLASS_KEYBOARD: u32 = u32::from_be_bytes(*b"keyb");
const K_EVENT_HOT_KEY_PRESSED: u32 = 5;
const K_EVENT_PARAM_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
const TYPE_EVENT_HOT_KEY_ID: u32 = u32::from_be_bytes(*b"hkid");

#[repr(C)]
#[derive(Copy, Clone)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

#[cfg_attr(target_os = "macos", link(name = "Carbon", kind = "framework"))]
extern "C" {
    fn GetApplicationEventTarget() -> EventTargetRef;
    fn InstallEventHandler(
        in_target: EventTargetRef,
        in_handler: extern "C" fn(EventHandlerCallRef, EventRef, *mut c_void) -> OSStatus,
        in_num_types: u32,
        in_list: *const EventTypeSpec,
        in_user_data: *mut c_void,
        out_ref: *mut EventHandlerRef,
    ) -> OSStatus;
    fn RegisterEventHotKey(
        in_hot_key_code: u32,
        in_hot_key_modifiers: u32,
        in_hot_key_id: EventHotKeyID,
        in_target: EventTargetRef,
        in_options: u32,
        out_ref: *mut EventHotKeyRef,
    ) -> OSStatus;
    fn UnregisterEventHotKey(in_hot_key: EventHotKeyRef) -> OSStatus;
    fn GetEventParameter(
        in_event: EventRef,
        in_name: u32,
        in_desired_type: u32,
        out_actual_type: *mut u32,
        in_buffer_size: u32,
        out_actual_size: *mut u32,
        out_data: *mut c_void,
    ) -> OSStatus;
}

#[cfg_attr(
    target_os = "macos",
    link(name = "ApplicationServices", kind = "framework")
)]
extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: CGKeyCode,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: CGEventFlags);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(cf: *const c_void);
}

static PENDING_HOTKEY_IDS: OnceLock<Mutex<VecDeque<u32>>> = OnceLock::new();

fn pending_queue() -> &'static Mutex<VecDeque<u32>> {
    PENDING_HOTKEY_IDS.get_or_init(|| Mutex::new(VecDeque::new()))
}

extern "C" fn carbon_hotkey_handler(
    _next_handler: EventHandlerCallRef,
    event: EventRef,
    _user_data: *mut c_void,
) -> OSStatus {
    let mut hk_id = EventHotKeyID::default();
    let status = unsafe {
        GetEventParameter(
            event,
            K_EVENT_PARAM_DIRECT_OBJECT,
            TYPE_EVENT_HOT_KEY_ID,
            null_mut(),
            std::mem::size_of::<EventHotKeyID>() as u32,
            null_mut(),
            (&mut hk_id as *mut EventHotKeyID).cast::<c_void>(),
        )
    };
    if status == 0 {
        if let Ok(mut queue) = pending_queue().lock() {
            queue.push_back(hk_id.id);
        }
    }
    0
}

pub struct MacOsBackend {
    handler_installed: bool,
    hotkey_refs: Vec<EventHotKeyRef>,
    registered_ids: HashMap<u32, (Uuid, KeyCombo, Modifiers)>,
    global_enabled: bool,
}

unsafe impl Send for MacOsBackend {}

impl Default for MacOsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOsBackend {
    pub fn new() -> Self {
        let mut backend = Self {
            handler_installed: false,
            hotkey_refs: Vec::new(),
            registered_ids: HashMap::new(),
            global_enabled: true,
        };
        backend.ensure_handler();
        backend
    }

    fn ensure_handler(&mut self) {
        if self.handler_installed {
            return;
        }
        let spec = EventTypeSpec {
            event_class: K_EVENT_CLASS_KEYBOARD,
            event_kind: K_EVENT_HOT_KEY_PRESSED,
        };
        let mut handler_ref: EventHandlerRef = null_mut();
        let status = unsafe {
            InstallEventHandler(
                GetApplicationEventTarget(),
                carbon_hotkey_handler,
                1,
                &spec,
                null_mut(),
                &mut handler_ref,
            )
        };
        if status == 0 {
            self.handler_installed = true;
        }
    }

    fn unregister_all(&mut self) {
        for hk_ref in self.hotkey_refs.drain(..) {
            if !hk_ref.is_null() {
                unsafe {
                    UnregisterEventHotKey(hk_ref);
                }
            }
        }
        self.registered_ids.clear();
    }

    fn carbon_modifiers(mods: Modifiers) -> u32 {
        let mut flags = 0;
        if mods.ctrl {
            flags |= CONTROL_KEY;
        }
        if mods.alt {
            flags |= OPTION_KEY;
        }
        if mods.shift {
            flags |= SHIFT_KEY;
        }
        if mods.meta {
            flags |= CMD_KEY;
        }
        flags
    }

    fn cg_event_flags(mods: Modifiers) -> CGEventFlags {
        let mut flags = 0;
        if mods.ctrl {
            flags |= K_CG_EVENT_FLAG_MASK_CONTROL;
        }
        if mods.alt {
            flags |= K_CG_EVENT_FLAG_MASK_ALTERNATE;
        }
        if mods.shift {
            flags |= K_CG_EVENT_FLAG_MASK_SHIFT;
        }
        if mods.meta {
            flags |= K_CG_EVENT_FLAG_MASK_COMMAND;
        }
        flags
    }

    fn execute_macos_media_action(key: LogicalKey) -> Result<()> {
        let script = match key {
            LogicalKey::VolumeUp => {
                "set volume output volume ((output volume of (get volume settings)) + 6)"
            }
            LogicalKey::VolumeDown => {
                "set volume output volume ((output volume of (get volume settings)) - 6)"
            }
            LogicalKey::VolumeMute => {
                "set volume output muted (not (output muted of (get volume settings)))"
            }
            _ => return Ok(()),
        };
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status();
        Ok(())
    }
}

impl Drop for MacOsBackend {
    fn drop(&mut self) {
        self.unregister_all();
    }
}

impl PlatformBackend for MacOsBackend {
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        self.ensure_handler();
        self.unregister_all();
        self.global_enabled = global_enabled;
        let mut warnings = Vec::new();

        if !global_enabled {
            return Ok(warnings);
        }

        for (idx, rule) in rules.iter().filter(|r| r.enabled).enumerate() {
            let MacOsKeyTarget::VirtualKey(vk) = to_macos_key(rule.trigger.key) else {
                warnings.push(format!(
                    "macOS Carbon hotkeys require a standard virtual key as trigger (rule '{}')",
                    rule.name
                ));
                continue;
            };

            let id = (idx as u32) + 1;
            let hk_id = EventHotKeyID {
                signature: u32::from_be_bytes(*b"HTKY"),
                id,
            };
            let mut hk_ref: EventHotKeyRef = null_mut();
            let status = unsafe {
                RegisterEventHotKey(
                    vk as u32,
                    Self::carbon_modifiers(rule.trigger.modifiers),
                    hk_id,
                    GetApplicationEventTarget(),
                    0,
                    &mut hk_ref,
                )
            };

            if status == 0 && !hk_ref.is_null() {
                self.hotkey_refs.push(hk_ref);
                self.registered_ids
                    .insert(id, (rule.id, rule.target, rule.trigger.modifiers));
            } else {
                warnings.push(format!(
                    "Failed to register '{}' on macOS (OSStatus {})",
                    rule.trigger, status
                ));
            }
        }

        Ok(warnings)
    }

    fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>> {
        let mut ids = Vec::new();
        if let Ok(mut queue) = pending_queue().lock() {
            while let Some(id) = queue.pop_front() {
                ids.push(id);
            }
        }

        let mut fired = Vec::new();
        if !self.global_enabled {
            return Ok(fired);
        }

        for id in ids {
            if let Some(&(rule_id, target, trigger_mods)) = self.registered_ids.get(&id) {
                self.send_combo(&target, Some(trigger_mods))?;
                fired.push(rule_id);
            }
        }

        Ok(fired)
    }

    fn send_combo(
        &mut self,
        combo: &KeyCombo,
        _held_trigger_mods: Option<Modifiers>,
    ) -> Result<()> {
        match to_macos_key(combo.key) {
            MacOsKeyTarget::VirtualKey(vk) => {
                let flags = Self::cg_event_flags(combo.modifiers);
                unsafe {
                    let down = CGEventCreateKeyboardEvent(null_mut(), vk, true);
                    if !down.is_null() {
                        CGEventSetFlags(down, flags);
                        CGEventPost(K_CG_HID_EVENT_TAP, down);
                        CFRelease(down.cast::<c_void>());
                    }
                    let up = CGEventCreateKeyboardEvent(null_mut(), vk, false);
                    if !up.is_null() {
                        CGEventSetFlags(up, flags);
                        CGEventPost(K_CG_HID_EVENT_TAP, up);
                        CFRelease(up.cast::<c_void>());
                    }
                }
            }
            MacOsKeyTarget::MediaKey(_) => {
                Self::execute_macos_media_action(combo.key)?;
            }
        }
        Ok(())
    }

    fn status_summary(&self) -> String {
        format!(
            "macOS Backend (Carbon RegisterEventHotKey + Quartz CGEventPost — {} active shortcut(s))",
            self.registered_ids.len()
        )
    }
}
