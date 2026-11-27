pub mod keymap;
#[cfg(any(target_os = "macos", test))]
#[allow(dead_code)]
pub mod macos;
#[cfg(any(target_os = "windows", test))]
#[allow(dead_code)]
pub mod windows;

use std::collections::{HashMap, VecDeque};

use anyhow::Result;
use uuid::Uuid;

use crate::model::{KeyCombo, KeybindRule, Modifiers};

pub trait PlatformBackend: Send {
    /// Synchronizes the active keybind rules with the OS hotkey listener.
    /// Returns any non-fatal warnings (e.g. shortcut conflict or permission hint).
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>>;

    /// Polls for any triggered hotkeys and executes their corresponding target key combos.
    /// Returns the IDs of rules that were triggered during this poll cycle.
    fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>>;

    /// Directly injects a synthetic key combination into the OS.
    fn send_combo(&mut self, combo: &KeyCombo, held_trigger_mods: Option<Modifiers>) -> Result<()>;

    /// Human-readable status string shown in the GUI footer/banner.
    fn status_summary(&self) -> String;
}

pub fn create_default_backend() -> Box<dyn PlatformBackend> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsBackend::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsBackend::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Box::new(MockBackend::new())
    }
}

/// Deterministic in-memory backend for unit and integration testing on any OS.
#[derive(Debug, Default)]
pub struct MockBackend {
    pub global_enabled: bool,
    pub active_rules: HashMap<KeyCombo, (Uuid, KeyCombo)>,
    pub pending_triggers: VecDeque<KeyCombo>,
    pub emitted_combos: Vec<KeyCombo>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn simulate_user_hotkey(&mut self, trigger: KeyCombo) {
        self.pending_triggers.push_back(trigger);
    }
}

impl PlatformBackend for MockBackend {
    fn apply_rules(&mut self, rules: &[KeybindRule], global_enabled: bool) -> Result<Vec<String>> {
        self.global_enabled = global_enabled;
        self.active_rules.clear();
        let mut warnings = Vec::new();

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
        let mut fired = Vec::new();
        while let Some(trigger) = self.pending_triggers.pop_front() {
            if !self.global_enabled {
                continue;
            }
            if let Some(&(rule_id, target)) = self.active_rules.get(&trigger) {
                self.send_combo(&target, Some(trigger.modifiers))?;
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
        self.emitted_combos.push(*combo);
        Ok(())
    }

    fn status_summary(&self) -> String {
        format!(
            "MockBackend ({} active rules, enabled={})",
            self.active_rules.len(),
            self.global_enabled
        )
    }
}
