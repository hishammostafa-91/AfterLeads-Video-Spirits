// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The settings store (docs/03 §3.6; REQ-SET-003): a flat
//! `{ "settingsVersion": 1, … }` document over the 68-key table
//! ([`crate::keys`]). Absent keys take defaults, unknown keys are
//! preserved, each value is validated (invalid → default + one collective
//! E8002 notice), writes are debounced 500 ms and atomic, and
//! `settings.json.bak` (the last good file) recovers a corrupt document
//! (defaults when none exists yet — spec assumption).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;

use crate::atomic::atomic_write;
use crate::autosave::Clock;
use crate::error::StoreError;
use crate::keys;

/// One invalid key replaced by its default during a load (03 §3.6); all
/// notices together form the single E8002 envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// The invalid key.
    pub key: String,
    /// Why the value was rejected.
    pub reason: String,
}

/// The §3.6 debounce/flush predicate: flush iff dirty and at least
/// `debounce_ms` passed since the last change.
pub fn should_flush(dirty: bool, since_change_ms: i64, debounce_ms: u64) -> bool {
    dirty && since_change_ms >= debounce_ms as i64
}

/// An in-memory settings document plus its load notices.
pub struct Settings {
    doc: BTreeMap<String, Value>,
    notices: Vec<Notice>,
}

impl Settings {
    /// Every key at its documented default (03 §3.6).
    pub fn defaults() -> Self {
        Self {
            doc: defaults_map(),
            notices: Vec::new(),
        }
    }

    /// The load ladder (REQ-SET-003): read `path` → validate against the
    /// table (invalid → default + notice, unknown preserved) → on an
    /// unreadable/wrong-version document, restore `backup` → on no usable
    /// backup, defaults with a notice.
    pub fn load(path: &Path, backup: &Path) -> Self {
        let mut notices = Vec::new();
        match read_document(path) {
            Some(doc) => Self::validated(doc, &mut notices),
            None => {
                // Corrupt (or wrong version): the last good file recovers.
                match read_document(backup) {
                    Some(doc) => {
                        notices.push(Notice {
                            key: "settings.json".into(),
                            reason: "restored from the last good backup".into(),
                        });
                        Self::validated(doc, &mut notices)
                    }
                    None => {
                        notices.push(Notice {
                            key: "settings.json".into(),
                            reason: "no usable settings file; defaults restored".into(),
                        });
                        Self {
                            doc: defaults_map(),
                            notices,
                        }
                    }
                }
            }
        }
    }

    /// Validate every known key (unknown keys ride through untouched).
    fn validated(mut doc: BTreeMap<String, Value>, notices: &mut Vec<Notice>) -> Self {
        for def in keys::KEYS {
            let Some(value) = doc.get(def.key) else {
                continue; // absent → default, silently (03 §3.6)
            };
            if let Err(reason) = keys::validate(def, value) {
                notices.push(Notice {
                    key: def.key.into(),
                    reason,
                });
                doc.insert(def.key.into(), keys::default_value(def));
            }
        }
        Self {
            doc,
            notices: std::mem::take(notices),
        }
    }

    /// The load notices (collectively one E8002).
    pub fn notices(&self) -> &[Notice] {
        &self.notices
    }

    /// The collective E8002 envelope when anything was restored.
    pub fn restoration_envelope(&self) -> Option<StoreError> {
        if self.notices.is_empty() {
            None
        } else {
            Some(StoreError::SettingsRestored {
                keys: self
                    .notices
                    .iter()
                    .map(|n| n.key.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            })
        }
    }

    /// A key's value (stored, else its default). Values are valid by
    /// construction: loads replaced invalid ones and [`Self::set`] enforces
    /// the table.
    pub fn get(&self, key: &str) -> Value {
        match self.doc.get(key) {
            Some(v) => v.clone(),
            None => keys::find(key)
                .map(keys::default_value)
                .unwrap_or(Value::Null),
        }
    }

    /// Set a key: unknown keys are stored verbatim (preserved), known keys
    /// must validate (invalid → default + notice, symmetric with load).
    pub fn set(&mut self, key: &str, value: Value) {
        match keys::find(key) {
            Some(def) if keys::validate(def, &value).is_err() => {
                self.notices.push(Notice {
                    key: key.into(),
                    reason: "value rejected on set; default applied".into(),
                });
                self.doc.insert(key.into(), keys::default_value(def));
            }
            Some(_) => {
                self.doc.insert(key.into(), value);
            }
            None => {
                self.doc.insert(key.into(), value);
            }
        }
    }

    /// The whole document (used by the writer; includes `settingsVersion`
    /// and unknown keys).
    pub fn document(&self) -> &BTreeMap<String, Value> {
        &self.doc
    }

    // Typed getters for the keys this package consumes (03 §3.6).
    /// `alvs.general.autosaveIntervalSec` (default 60).
    pub fn autosave_interval_sec(&self) -> u32 {
        u32::try_from(
            self.get("alvs.general.autosaveIntervalSec")
                .as_i64()
                .unwrap_or(60),
        )
        .unwrap_or(60)
    }
    /// `alvs.general.backupsKeep` (default 20).
    pub fn backups_keep(&self) -> u32 {
        u32::try_from(self.get("alvs.general.backupsKeep").as_i64().unwrap_or(20)).unwrap_or(20)
    }
    /// `alvs.general.recentLimit` (default 20).
    pub fn recent_limit(&self) -> u32 {
        u32::try_from(self.get("alvs.general.recentLimit").as_i64().unwrap_or(20)).unwrap_or(20)
    }
    /// `alvs.advanced.logLevel` (default "info").
    pub fn log_level(&self) -> String {
        self.get("alvs.advanced.logLevel")
            .as_str()
            .unwrap_or("info")
            .to_string()
    }
    /// `alvs.advanced.redactDiagnostics` (default true).
    pub fn redact_diagnostics(&self) -> bool {
        self.get("alvs.advanced.redactDiagnostics")
            .as_bool()
            .unwrap_or(true)
    }
    /// `alvs.media.cacheFolder` (default "default").
    pub fn cache_folder(&self) -> String {
        self.get("alvs.media.cacheFolder")
            .as_str()
            .unwrap_or("default")
            .to_string()
    }
    /// `alvs.general.defaultProjectFolder`.
    pub fn default_project_folder(&self) -> String {
        self.get("alvs.general.defaultProjectFolder")
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

fn defaults_map() -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    map.insert("settingsVersion".into(), Value::from(1u32));
    for def in keys::KEYS {
        map.insert(def.key.into(), keys::default_value(def));
    }
    map
}

/// Read and gate a settings document: `None` when unreadable, unparsable,
/// not an object, or `settingsVersion ≠ 1`.
fn read_document(path: &Path) -> Option<BTreeMap<String, Value>> {
    let bytes = std::fs::read(path).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    let map = value.as_object()?;
    let mut out: BTreeMap<String, Value> = map.clone().into_iter().collect();
    match out.remove("settingsVersion") {
        Some(v) if v.as_u64() == Some(1) => {
            out.insert("settingsVersion".into(), v);
            Some(out)
        }
        _ => None,
    }
}

/// Serialize a document to the on-disk form (03 §3.2 conventions: UTF-8,
/// LF, 2-space, camelCase — key order is the map's alphabetical).
fn to_bytes(doc: &BTreeMap<String, Value>) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec_pretty(doc).map_err(|e| StoreError::SaveFailed {
        detail: format!("settings serialization failed: {e}"),
    })
}

/// Whether raw bytes are a usable settings document (version 1) — the
/// gate for promoting a previous file to `.bak`.
fn is_good_document(bytes: &[u8]) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|v| v.get("settingsVersion").cloned())
        .and_then(|v| v.as_u64())
        .is_some_and(|v| v == 1)
}

/// The debounced writer (03 §3.6 write policy): `set` marks dirty; the
/// flush decision waits 500 ms of quiet; writes are atomic and refresh
/// `settings.json.bak` with the previous good file first.
pub struct SettingsWriter {
    path: PathBuf,
    backup: PathBuf,
    clock: Arc<dyn Clock>,
    debounce_ms: u64,
    dirty: std::sync::atomic::AtomicBool,
    last_change_ms: std::sync::atomic::AtomicI64,
}

impl SettingsWriter {
    /// A writer for `path` with its `.bak`, a clock, and the 500 ms
    /// debounce (03 §3.6).
    pub fn new(path: &Path, backup: &Path, clock: Arc<dyn Clock>, debounce_ms: u64) -> Self {
        Self {
            path: path.to_path_buf(),
            backup: backup.to_path_buf(),
            clock,
            debounce_ms,
            dirty: std::sync::atomic::AtomicBool::new(false),
            last_change_ms: std::sync::atomic::AtomicI64::new(0),
        }
    }

    /// Mark the document changed (the debounce window restarts).
    pub fn note_change(&self) {
        self.last_change_ms
            .store(self.clock.now_ms(), std::sync::atomic::Ordering::SeqCst);
        self.dirty.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// One debounce evaluation: write when the predicate holds. Returns
    /// whether a write happened.
    pub fn maybe_flush(&self, doc: &BTreeMap<String, Value>) -> Result<bool, StoreError> {
        if !should_flush(
            self.dirty.load(std::sync::atomic::Ordering::SeqCst),
            self.clock.now_ms()
                - self
                    .last_change_ms
                    .load(std::sync::atomic::Ordering::SeqCst),
            self.debounce_ms,
        ) {
            return Ok(false);
        }
        self.flush(doc)?;
        Ok(true)
    }

    /// Write now: the current good file becomes `.bak` (when it exists
    /// AND is a valid settings document — a corrupt file is never
    /// promoted), then the new document lands atomically (REQ-SET-003's
    /// "last good" recovery source).
    pub fn flush(&self, doc: &BTreeMap<String, Value>) -> Result<(), StoreError> {
        if let Ok(previous) = std::fs::read(&self.path)
            && is_good_document(&previous)
        {
            atomic_write(&self.backup, &previous)?;
        }
        atomic_write(&self.path, &to_bytes(doc)?)?;
        self.dirty.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// The production poll loop (stop on drop): evaluates the debounce on
    /// a shared [`Settings`] snapshot holder.
    pub fn spawn(
        self: &Arc<Self>,
        snapshot: Arc<std::sync::Mutex<BTreeMap<String, Value>>>,
        poll_ms: u64,
    ) -> SettingsWriterHandle {
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let writer = Arc::clone(self);
        let thread_stop = Arc::clone(&stop);
        let join = std::thread::spawn(move || {
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(poll_ms));
                if thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let doc = match snapshot.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let _ = writer.maybe_flush(&doc);
            }
        });
        SettingsWriterHandle {
            stop,
            join: Some(join),
        }
    }
}

/// The writer's poll thread; stops and joins on drop (02 §2.5 shutdown
/// step 4 flushes settings).
pub struct SettingsWriterHandle {
    stop: Arc<std::sync::atomic::AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl Drop for SettingsWriterHandle {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// Kind is re-exported for typed consumers of the table.
pub use keys::Kind as SettingsKind;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autosave::SystemClock;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alvs-settings-{tag}-{}",
            uuid::Uuid::now_v7().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap_or(());
        dir
    }

    #[test]
    fn flush_predicate_truth_table() {
        assert!(!should_flush(false, 10_000, 500)); // clean never
        assert!(!should_flush(true, 499, 500)); // inside the window
        assert!(should_flush(true, 500, 500)); // exactly 500 ms
    }

    #[test]
    fn flush_writes_bak_then_main_atomically() {
        let dir = scratch("flush");
        let path = dir.join("settings.json");
        let backup = dir.join("settings.json.bak");
        let writer = SettingsWriter::new(&path, &backup, Arc::new(SystemClock), 500);
        let mut settings = Settings::defaults();
        settings.set("alvs.general.backupsKeep", Value::from(7));
        writer.flush(settings.document()).unwrap_or(());
        assert!(path.exists());
        assert!(!backup.exists(), "no .bak before a second good file exists");
        settings.set("alvs.general.backupsKeep", Value::from(9));
        writer.flush(settings.document()).unwrap_or(());
        assert!(backup.exists(), "the previous good file becomes .bak");
        let saved: Value = match serde_json::from_slice(&std::fs::read(&path).unwrap_or_default()) {
            Ok(v) => v,
            Err(e) => panic!("parse: {e}"),
        };
        assert_eq!(saved["settingsVersion"], Value::from(1u32));
        assert_eq!(saved["alvs.general.backupsKeep"], Value::from(9));
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }
}
