use std::collections::HashMap;
use std::io::ErrorKind;
use std::os::fd::AsRawFd;

use anyhow::{Context, Result};
use evdev::{
    uinput::VirtualDevice, AttributeSet, Device, EventType, InputEvent, KeyCode,
};
use uuid::Uuid;

use crate::backend::{
    keymap::{from_linux_keycode, to_linux_keycode},
    PlatformBackend,
};
use crate::model::{KeyCombo, KeybindRule, Modifiers};

const VIRTUAL_DEVICE_NAME: &str = "Hotkeys Virtual Keyboard";

const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_LEFTMETA: u16 = 125;
const KEY_RIGHTMETA: u16 = 126;

pub struct LinuxBackend {
    devices: Vec<Device>,
    uinput: Option<VirtualDevice>,
    active_rules: HashMap<KeyCombo, (Uuid, KeyCombo)>,
    global_enabled: bool,
    held_ctrl: bool,
    held_alt: bool,
    held_shift: bool,
    held_meta: bool,
    init_warning: Option<String>,
}

impl LinuxBackend {
    pub fn new() -> Self {
        let mut init_warning = None;
        let uinput = match Self::create_uinput_device() {
            Ok(dev) => Some(dev),
            Err(err) => {
                init_warning = Some(format!(
                    "Wayland/Linux uinput unavailable ({}). Ensure your user is in the 'input' group and /dev/uinput is writable.",
                    err
                ));
                None
            }
        };

        let devices = Self::open_keyboard_devices(&mut init_warning);

        Self {
            devices,
            uinput,
            active_rules: HashMap::new(),
            global_enabled: true,
            held_ctrl: false,
            held_alt: false,
            held_shift: false,
            held_meta: false,
            init_warning,
        }
    }

    fn create_uinput_device() -> Result<VirtualDevice> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for code in 1..=248u16 {
            keys.insert(KeyCode::new(code));
        }

        let dev = VirtualDevice::builder()
            .context("Failed to open /dev/uinput")?
            .name(VIRTUAL_DEVICE_NAME)
            .with_keys(&keys)
            .context("Failed to register key capabilities on uinput device")?
            .build()
            .context("Failed to build uinput virtual keyboard")?;

        Ok(dev)
    }

    fn open_keyboard_devices(warning_out: &mut Option<String>) -> Vec<Device> {
        let mut keyboards = Vec::new();
        for (_path, dev) in evdev::enumerate() {
            if dev.name() == Some(VIRTUAL_DEVICE_NAME) {
                continue;
            }
            let is_keyboard = dev.supported_keys().is_some_and(|keys| {
                keys.contains(KeyCode::KEY_A)
                    || keys.contains(KeyCode::KEY_F1)
                    || keys.contains(KeyCode::KEY_PLAYPAUSE)
            });
            if !is_keyboard {
                continue;
            }

            let fd = dev.as_raw_fd();
            unsafe {
                let flags = libc::fcntl(fd, libc::F_GETFL);
                if flags >= 0 {
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }
            }
            keyboards.push(dev);
        }

        if keyboards.is_empty() && warning_out.is_none() {
            *warning_out = Some(
                "No readable keyboard devices in /dev/input/event*. On Wayland, run: sudo usermod -aG input $USER && re-login."
                    .to_string(),
            );
        }

        keyboards
    }

    fn current_modifiers(&self) -> Modifiers {
        Modifiers {
            ctrl: self.held_ctrl,
            alt: self.held_alt,
            shift: self.held_shift,
            meta: self.held_meta,
        }
    }

    fn update_modifier_state(&mut self, code: u16, pressed: bool) -> bool {
        match code {
            KEY_LEFTCTRL | KEY_RIGHTCTRL => {
                self.held_ctrl = pressed;
                true
            }
            KEY_LEFTALT | KEY_RIGHTALT => {
                self.held_alt = pressed;
                true
            }
            KEY_LEFTSHIFT | KEY_RIGHTSHIFT => {
                self.held_shift = pressed;
                true
            }
            KEY_LEFTMETA | KEY_RIGHTMETA => {
                self.held_meta = pressed;
                true
            }
            _ => false,
        }
    }

    fn emit_key(uinput: &mut VirtualDevice, code: u16, value: i32) -> Result<()> {
        let ev = InputEvent::new(EventType::KEY.0, code, value);
        uinput
            .emit(&[ev])
            .context("Failed to emit uinput key event")
    }
}

impl PlatformBackend for LinuxBackend {
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        self.global_enabled = global_enabled;
        self.active_rules.clear();

        if self.devices.is_empty() {
            let mut warn = None;
            self.devices = Self::open_keyboard_devices(&mut warn);
        }

        let mut warnings = Vec::new();
        if let Some(w) = &self.init_warning {
            warnings.push(w.clone());
        }

        if !global_enabled {
            return Ok(warnings);
        }

        for rule in rules.iter().filter(|r| r.enabled) {
            if self
                .active_rules
                .insert(rule.trigger, (rule.id, rule.target))
                .is_some()
            {
                warnings.push(format!(
                    "Duplicate trigger '{}' on rule '{}'",
                    rule.trigger, rule.name
                ));
            }
        }

        Ok(warnings)
    }

    fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>> {
        let mut triggered_actions: Vec<(Uuid, KeyCombo, Modifiers)> = Vec::new();

        for idx in 0..self.devices.len() {
            let events: Vec<InputEvent> = match self.devices[idx].fetch_events() {
                Ok(iter) => iter.collect(),
                Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                Err(_) => continue,
            };

            for ev in events {
                if ev.event_type() != EventType::KEY {
                    continue;
                }
                let code = ev.code();
                let value = ev.value(); // 0 = release, 1 = press, 2 = repeat
                if value == 2 {
                    continue; // ignore key repeat (MOD_NOREPEAT behavior)
                }

                let pressed = value == 1;
                if self.update_modifier_state(code, pressed) {
                    continue;
                }

                if pressed && self.global_enabled {
                    if let Some(logical_key) = from_linux_keycode(code) {
                        let mods = self.current_modifiers();
                        let combo = KeyCombo::new(mods, logical_key);
                        if let Some(&(rule_id, target)) = self.active_rules.get(&combo) {
                            triggered_actions.push((rule_id, target, mods));
                        }
                    }
                }
            }
        }

        let mut fired_ids = Vec::new();
        for (rule_id, target, trigger_mods) in triggered_actions {
            self.send_combo(&target, Some(trigger_mods))?;
            fired_ids.push(rule_id);
        }

        Ok(fired_ids)
    }

    fn send_combo(&mut self, combo: &KeyCombo, held_trigger_mods: Option<Modifiers>) -> Result<()> {
        let Some(uinput) = self.uinput.as_mut() else {
            anyhow::bail!("uinput virtual keyboard is not initialized");
        };

        // Temporarily release physically held trigger modifiers that are not part of the target combo
        if let Some(held) = held_trigger_mods {
            if held.ctrl && !combo.modifiers.ctrl {
                Self::emit_key(uinput, KEY_LEFTCTRL, 0)?;
                Self::emit_key(uinput, KEY_RIGHTCTRL, 0)?;
            }
            if held.alt && !combo.modifiers.alt {
                Self::emit_key(uinput, KEY_LEFTALT, 0)?;
                Self::emit_key(uinput, KEY_RIGHTALT, 0)?;
            }
            if held.shift && !combo.modifiers.shift {
                Self::emit_key(uinput, KEY_LEFTSHIFT, 0)?;
                Self::emit_key(uinput, KEY_RIGHTSHIFT, 0)?;
            }
            if held.meta && !combo.modifiers.meta {
                Self::emit_key(uinput, KEY_LEFTMETA, 0)?;
                Self::emit_key(uinput, KEY_RIGHTMETA, 0)?;
            }
        }

        // Press target modifiers
        if combo.modifiers.ctrl {
            Self::emit_key(uinput, KEY_LEFTCTRL, 1)?;
        }
        if combo.modifiers.alt {
            Self::emit_key(uinput, KEY_LEFTALT, 1)?;
        }
        if combo.modifiers.shift {
            Self::emit_key(uinput, KEY_LEFTSHIFT, 1)?;
        }
        if combo.modifiers.meta {
            Self::emit_key(uinput, KEY_LEFTMETA, 1)?;
        }

        // Press and release target key
        let target_code = to_linux_keycode(combo.key);
        Self::emit_key(uinput, target_code, 1)?;
        Self::emit_key(uinput, target_code, 0)?;

        // Release target modifiers
        if combo.modifiers.meta {
            Self::emit_key(uinput, KEY_LEFTMETA, 0)?;
        }
        if combo.modifiers.shift {
            Self::emit_key(uinput, KEY_LEFTSHIFT, 0)?;
        }
        if combo.modifiers.alt {
            Self::emit_key(uinput, KEY_LEFTALT, 0)?;
        }
        if combo.modifiers.ctrl {
            Self::emit_key(uinput, KEY_LEFTCTRL, 0)?;
        }

        Ok(())
    }

    fn status_summary(&self) -> String {
        if let Some(w) = &self.init_warning {
            format!("Linux Wayland/X11 (evdev/uinput): {}", w)
        } else {
            format!(
                "Linux Wayland/X11 (evdev + uinput active — {} keyboard(s) monitored, {} rule(s))",
                self.devices.len(),
                self.active_rules.len()
            )
        }
    }
}
