use std::collections::HashMap;

use anyhow::Result;
use uuid::Uuid;

use crate::backend::PlatformBackend;
use crate::config::AppConfig;
use crate::model::{KeyCombo, KeybindRule};

pub struct HotkeyEngine {
    pub config: AppConfig,
    backend: Box<dyn PlatformBackend>,
    pub warnings: Vec<String>,
    pub recording_active: bool,
    pub last_triggered_rule: Option<String>,
    pub total_triggers_count: u64,
}

impl HotkeyEngine {
    pub fn new(config: AppConfig, backend: Box<dyn PlatformBackend>) -> Result<Self> {
        let mut engine = Self {
            config,
            backend,
            warnings: Vec::new(),
            recording_active: false,
            last_triggered_rule: None,
            total_triggers_count: 0,
        };
        engine.sync_backend()?;
        Ok(engine)
    }

    pub fn sync_backend(&mut self) -> Result<()> {
        let effective_enabled = self.config.global_enabled && !self.recording_active;
        let mut warnings = Self::find_duplicate_triggers(&self.config.rules);
        let backend_warnings = self
            .backend
            .apply_rules(&self.config.rules, effective_enabled)?;
        for w in backend_warnings {
            if !warnings.contains(&w) {
                warnings.push(w);
            }
        }
        self.warnings = warnings;
        Ok(())
    }

    pub fn set_recording_active(&mut self, active: bool) -> Result<()> {
        if self.recording_active != active {
            self.recording_active = active;
            self.sync_backend()?;
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Vec<Uuid>> {
        if !self.config.global_enabled || self.recording_active {
            return Ok(Vec::new());
        }
        let fired_ids = self.backend.poll_and_dispatch()?;
        for id in &fired_ids {
            self.total_triggers_count += 1;
            if let Some(rule) = self.config.rules.iter().find(|r| r.id == *id) {
                self.last_triggered_rule = Some(format!(
                    "{} ({} -> {})",
                    rule.name, rule.trigger, rule.target
                ));
            }
        }
        Ok(fired_ids)
    }

    pub fn test_rule_output(&mut self, target: &KeyCombo) -> Result<()> {
        self.backend.send_combo(target, None)
    }

    pub fn backend_status(&self) -> String {
        self.backend.status_summary()
    }

    pub fn find_duplicate_triggers(rules: &[KeybindRule]) -> Vec<String> {
        let mut seen: HashMap<KeyCombo, &str> = HashMap::new();
        let mut warnings = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            if let Some(prev_name) = seen.insert(rule.trigger, &rule.name) {
                warnings.push(format!(
                    "Conflict: '{}' and '{}' both use trigger '{}'",
                    prev_name, rule.name, rule.trigger
                ));
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::model::{LogicalKey, Modifiers};
    use std::sync::{Arc, Mutex};

    struct SharedMockBackend(Arc<Mutex<MockBackend>>);

    impl PlatformBackend for SharedMockBackend {
        fn apply_rules(
            &mut self,
            rules: &[KeybindRule],
            global_enabled: bool,
        ) -> Result<Vec<String>> {
            self.0.lock().unwrap().apply_rules(rules, global_enabled)
        }

        fn poll_and_dispatch(&mut self) -> Result<Vec<Uuid>> {
            self.0.lock().unwrap().poll_and_dispatch()
        }

        fn send_combo(
            &mut self,
            combo: &KeyCombo,
            held_trigger_mods: Option<Modifiers>,
        ) -> Result<()> {
            self.0.lock().unwrap().send_combo(combo, held_trigger_mods)
        }

        fn status_summary(&self) -> String {
            self.0.lock().unwrap().status_summary()
        }
    }

    #[test]
    fn test_engine_dispatches_mapped_key_combo() {
        let mock = Arc::new(Mutex::new(MockBackend::new()));
        let cfg = AppConfig::default();
        let mut engine = HotkeyEngine::new(cfg, Box::new(SharedMockBackend(mock.clone()))).unwrap();

        let alt_f2 = KeyCombo::new(Modifiers::new(false, true, false, false), LogicalKey::F2);
        mock.lock().unwrap().simulate_user_hotkey(alt_f2);

        let fired = engine.poll().unwrap();
        assert_eq!(fired.len(), 1);
        assert_eq!(engine.total_triggers_count, 1);

        let emitted = &mock.lock().unwrap().emitted_combos;
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0], KeyCombo::bare(LogicalKey::MediaPlayPause));
    }

    #[test]
    fn test_engine_pauses_while_recording_or_disabled() {
        let mock = Arc::new(Mutex::new(MockBackend::new()));
        let cfg = AppConfig::default();
        let mut engine = HotkeyEngine::new(cfg, Box::new(SharedMockBackend(mock.clone()))).unwrap();

        engine.set_recording_active(true).unwrap();
        let alt_f1 = KeyCombo::new(Modifiers::new(false, true, false, false), LogicalKey::F1);
        mock.lock().unwrap().simulate_user_hotkey(alt_f1);

        let fired = engine.poll().unwrap();
        assert!(fired.is_empty());
        assert!(mock.lock().unwrap().emitted_combos.is_empty());
    }

    #[test]
    fn test_duplicate_trigger_detection() {
        let trigger = KeyCombo::new(Modifiers::new(true, false, false, false), LogicalKey::F1);
        let rules = vec![
            KeybindRule::new("Rule A", trigger, KeyCombo::bare(LogicalKey::VolumeUp)),
            KeybindRule::new("Rule B", trigger, KeyCombo::bare(LogicalKey::VolumeDown)),
        ];
        let warnings = HotkeyEngine::find_duplicate_triggers(&rules);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Rule A"));
        assert!(warnings[0].contains("Rule B"));
    }
}
