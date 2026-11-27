use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{Level, LevelFilter, Log, Metadata, Record};

const MAX_MEMORY_ENTRIES: usize = 500;
const MAX_LOG_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: Level,
    pub message: String,
}

struct LoggerState {
    entries: VecDeque<LogEntry>,
    log_path: PathBuf,
    file: Option<File>,
}

static LOGGER_STATE: OnceLock<Mutex<LoggerState>> = OnceLock::new();
static LOGGER_INSTANCE: HotkeysLogger = HotkeysLogger;

struct HotkeysLogger;

impl Log for HotkeysLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // Filter noisy third-party GUI/GPU internals unless they are warnings or errors
        let target = record.target();
        if !target.starts_with("hotkeys") && record.level() > Level::Warn {
            return;
        }

        let timestamp = format_utc_timestamp(SystemTime::now());
        let entry = LogEntry {
            timestamp,
            level: record.level(),
            message: format!("{}", record.args()),
        };
        record_entry(entry);
    }

    fn flush(&self) {
        if let Some(mutex) = LOGGER_STATE.get() {
            if let Ok(mut state) = mutex.lock() {
                if let Some(file) = state.file.as_mut() {
                    let _ = file.flush();
                }
            }
        }
    }
}

fn record_entry(entry: LogEntry) {
    #[cfg(debug_assertions)]
    eprintln!("[{} {:<5}] {}", entry.timestamp, entry.level, entry.message);

    if let Some(mutex) = LOGGER_STATE.get() {
        if let Ok(mut state) = mutex.lock() {
            let line = format!(
                "[{} {:<5}] {}\n",
                entry.timestamp, entry.level, entry.message
            );
            if let Some(file) = state.file.as_mut() {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
            if state.entries.len() >= MAX_MEMORY_ENTRIES {
                state.entries.pop_front();
            }
            state.entries.push_back(entry);
        }
    }
}

pub fn default_log_file_path() -> PathBuf {
    if let Some(base) = dirs::config_dir() {
        base.join("hotkeys").join("hotkeys.log")
    } else {
        std::env::temp_dir().join("hotkeys.log")
    }
}

fn open_log_file(path: &Path) -> Option<File> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() > MAX_LOG_FILE_BYTES {
            let backup = path.with_extension("log.1");
            let _ = fs::rename(path, backup);
        }
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}

pub fn init_logger() -> PathBuf {
    let path = default_log_file_path();
    let _ = LOGGER_STATE.get_or_init(|| {
        let file = open_log_file(&path);
        Mutex::new(LoggerState {
            entries: VecDeque::with_capacity(MAX_MEMORY_ENTRIES),
            log_path: path.clone(),
            file,
        })
    });

    if log::set_logger(&LOGGER_INSTANCE).is_ok() {
        log::set_max_level(LevelFilter::Debug);
    }

    log::info!("Hotkeys logger initialized (log file: {})", path.display());
    path
}

pub fn recent_logs() -> Vec<LogEntry> {
    LOGGER_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|state| state.entries.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn clear_memory_logs() {
    if let Some(mutex) = LOGGER_STATE.get() {
        if let Ok(mut state) = mutex.lock() {
            state.entries.clear();
        }
    }
}

pub fn log_file_path() -> PathBuf {
    LOGGER_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|state| state.log_path.clone())
        .unwrap_or_else(default_log_file_path)
}

fn format_utc_timestamp(now: SystemTime) -> String {
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = duration.as_secs() as i64;
    let millis = duration.subsec_millis();

    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    // Howard Hinnant's civil-from-days algorithm
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        year, m, d, hour, minute, second, millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_utc_timestamp_epoch() {
        let ts = format_utc_timestamp(UNIX_EPOCH);
        assert_eq!(ts, "1970-01-01 00:00:00.000");
    }

    #[test]
    fn test_ring_buffer_captures_and_caps_entries() {
        let tmp = std::env::temp_dir().join(format!("hotkeys_log_test_{}", uuid::Uuid::new_v4()));
        let path = tmp.join("hotkeys.log");
        let _ = LOGGER_STATE.get_or_init(|| {
            let file = open_log_file(&path);
            Mutex::new(LoggerState {
                entries: VecDeque::with_capacity(MAX_MEMORY_ENTRIES),
                log_path: path.clone(),
                file,
            })
        });

        clear_memory_logs();
        for idx in 0..(MAX_MEMORY_ENTRIES + 15) {
            record_entry(LogEntry {
                timestamp: "2026-01-01 00:00:00.000".to_string(),
                level: Level::Info,
                message: format!("test entry {}", idx),
            });
        }

        let entries = recent_logs();
        assert_eq!(entries.len(), MAX_MEMORY_ENTRIES);
        assert_eq!(entries.first().unwrap().message, "test entry 15");
        assert_eq!(
            entries.last().unwrap().message,
            format!("test entry {}", MAX_MEMORY_ENTRIES + 14)
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
