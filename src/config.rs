use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::{KeyCombo, KeybindRule, LogicalKey, Modifiers};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub global_enabled: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub rules: Vec<KeybindRule>,
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_enabled: true,
            start_minimized: false,
            rules: Self::default_rules(),
        }
    }
}

impl AppConfig {
    pub fn default_rules() -> Vec<KeybindRule> {
        let alt = Modifiers::new(false, true, false, false);
        vec![
            KeybindRule::new(
                "Previous Track",
                KeyCombo::new(alt, LogicalKey::F1),
                KeyCombo::bare(LogicalKey::MediaPrevTrack),
            ),
            KeybindRule::new(
                "Play / Pause",
                KeyCombo::new(alt, LogicalKey::F2),
                KeyCombo::bare(LogicalKey::MediaPlayPause),
            ),
            KeybindRule::new(
                "Next Track",
                KeyCombo::new(alt, LogicalKey::F3),
                KeyCombo::bare(LogicalKey::MediaNextTrack),
            ),
            KeybindRule::new(
                "Mute / Unmute",
                KeyCombo::new(alt, LogicalKey::F5),
                KeyCombo::bare(LogicalKey::VolumeMute),
            ),
            KeybindRule::new(
                "Volume Down",
                KeyCombo::new(alt, LogicalKey::F6),
                KeyCombo::bare(LogicalKey::VolumeDown),
            ),
            KeybindRule::new(
                "Volume Up",
                KeyCombo::new(alt, LogicalKey::F7),
                KeyCombo::bare(LogicalKey::VolumeUp),
            ),
        ]
    }

    pub fn default_config_path() -> Result<PathBuf> {
        let base = dirs::config_dir().context("Could not determine OS configuration directory")?;
        Ok(base.join("hotkeys").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::default_config_path()?;
        if !path.exists() {
            log::info!(
                "No config found at {}; creating default keybind rules",
                path.display()
            );
            let cfg = Self::default();
            if let Err(err) = cfg.save_to_path(&path) {
                log::warn!("Failed to write initial default config: {}", err);
            }
            return Ok(cfg);
        }
        match Self::load_from_path(&path) {
            Ok(cfg) => {
                log::info!(
                    "Loaded {} keybind rule(s) from {}",
                    cfg.rules.len(),
                    path.display()
                );
                Ok(cfg)
            }
            Err(err) => {
                log::error!("Failed to load config from {}: {}", path.display(), err);
                Err(err)
            }
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let cfg: Self = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse TOML config in {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_config_path()?;
        self.save_to_path(&path)?;
        log::info!(
            "Saved {} keybind rule(s) to {}",
            self.rules.len(),
            path.display()
        );
        Ok(())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }
        let serialized =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;
        fs::write(path, serialized)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_roundtrip() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.rules.len(), 6);
        let toml_str = toml::to_string_pretty(&cfg).expect("serialize toml");
        let parsed: AppConfig = toml::from_str(&toml_str).expect("deserialize toml");
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn test_save_and_load_from_temp_path() {
        let tmp_dir = std::env::temp_dir().join(format!("hotkeys_test_{}", uuid::Uuid::new_v4()));
        let cfg_path = tmp_dir.join("config.toml");

        let mut cfg = AppConfig::default();
        cfg.rules.push(KeybindRule::new(
            "Copy Shortcut",
            KeyCombo::new(Modifiers::new(true, true, false, false), LogicalKey::C),
            KeyCombo::new(Modifiers::new(true, false, false, false), LogicalKey::C),
        ));

        cfg.save_to_path(&cfg_path).expect("save config");
        let loaded = AppConfig::load_from_path(&cfg_path).expect("load config");
        assert_eq!(cfg, loaded);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
