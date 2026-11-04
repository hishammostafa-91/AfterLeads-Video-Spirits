// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-SET-003 (09 §9.10): settings validation, unknown keys, corruption
//! recovery — REQ-SET-003 over docs/03 §3.6, with the debounce driven by
//! an injected clock (no wall-clock dependence).

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use alvs_store::autosave::Clock;
use alvs_store::settings::{Settings, SettingsWriter};

struct FakeClock {
    now: AtomicI64,
}

impl FakeClock {
    fn new(start: i64) -> Self {
        Self {
            now: AtomicI64::new(start),
        }
    }
    fn advance(&self, ms: i64) {
        self.now.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-set-003-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

fn write_doc(path: &std::path::Path, json: &serde_json::Value) {
    std::fs::write(path, serde_json::to_vec_pretty(json).unwrap_or_default()).unwrap_or(());
}

#[test]
fn t_set_003_invalid_values_revert_to_defaults_with_one_notice() {
    let dir = scratch("invalid");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    write_doc(
        &path,
        &serde_json::json!({
            "settingsVersion": 1,
            "alvs.general.autosaveIntervalSec": 5,     // below the 15–600 range
            "alvs.appearance.theme": "purple",          // not an enum member
            "alvs.appearance.uiZoom": 85,               // violates the step-10 grid
            "alvs.future.unknownKey": { "kept": true }, // unknown keys preserved
        }),
    );

    let settings = Settings::load(&path, &backup);
    // Invalid → documented defaults (03 §3.6).
    assert_eq!(
        settings.get("alvs.general.autosaveIntervalSec"),
        serde_json::json!(60)
    );
    assert_eq!(
        settings.get("alvs.appearance.theme"),
        serde_json::json!("dark")
    );
    assert_eq!(
        settings.get("alvs.appearance.uiZoom"),
        serde_json::json!(100)
    );
    // Exactly one collective E8002 notice listing the three invalid keys.
    let notices = settings.notices();
    assert_eq!(notices.len(), 3, "{notices:?}");
    let envelope = match settings.restoration_envelope() {
        Some(e) => e,
        None => panic!("notice envelope expected"),
    };
    assert_eq!(envelope.code(), alvs_core::error::ErrorCode::E8002);
    // Unknown keys ride through untouched.
    assert_eq!(
        settings.get("alvs.future.unknownKey"),
        serde_json::json!({ "kept": true })
    );

    // A flush persists valid values and the unknown key together.
    let clock = Arc::new(FakeClock::new(0));
    let writer = SettingsWriter::new(&path, &backup, clock, 500);
    writer
        .flush(settings.document())
        .unwrap_or_else(|e| panic!("flush: {e}"));
    let saved: serde_json::Value =
        match serde_json::from_slice(&std::fs::read(&path).unwrap_or_default()) {
            Ok(v) => v,
            Err(e) => panic!("parse: {e}"),
        };
    assert_eq!(
        saved["alvs.general.autosaveIntervalSec"],
        serde_json::json!(60)
    );
    assert_eq!(
        saved["alvs.future.unknownKey"]["kept"],
        serde_json::json!(true)
    );
    assert!(backup.exists(), "the previous good file became .bak");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_set_003_corrupt_file_restores_from_the_last_good_backup() {
    let dir = scratch("corrupt-bak");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    write_doc(
        &backup,
        &serde_json::json!({
            "settingsVersion": 1,
            "alvs.general.backupsKeep": 33,
        }),
    );
    std::fs::write(&path, b"{ not json").unwrap_or(());

    let settings = Settings::load(&path, &backup);
    assert_eq!(
        settings.get("alvs.general.backupsKeep"),
        serde_json::json!(33),
        "the .bak value survives"
    );
    assert!(!settings.notices().is_empty(), "one E8002 notice");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_set_003_corrupt_without_backup_falls_back_to_defaults() {
    let dir = scratch("corrupt-none");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    std::fs::write(&path, b"\x00garbage").unwrap_or(());

    let settings = Settings::load(&path, &backup);
    assert_eq!(
        settings.get("alvs.general.autosaveIntervalSec"),
        serde_json::json!(60)
    );
    assert_eq!(settings.log_level(), "info");
    assert!(settings.redact_diagnostics());
    assert!(!settings.notices().is_empty(), "one E8002 notice");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_set_003_wrong_version_is_treated_as_corrupt() {
    let dir = scratch("version");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    write_doc(&path, &serde_json::json!({ "settingsVersion": 2 }));

    let settings = Settings::load(&path, &backup);
    assert_eq!(
        settings.get("alvs.general.recentLimit"),
        serde_json::json!(20),
        "defaults apply"
    );
    assert!(!settings.notices().is_empty());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_set_003_rapid_changes_coalesce_into_one_debounced_write() {
    let dir = scratch("debounce");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    let clock = Arc::new(FakeClock::new(0));
    let writer = Arc::new(SettingsWriter::new(&path, &backup, clock.clone(), 500));
    let mut settings = Settings::defaults();

    // Change at t=0 and t=300: inside the window, no write.
    settings.set("alvs.general.recentLimit", serde_json::json!(7));
    writer.note_change();
    clock.advance(300);
    settings.set("alvs.general.recentLimit", serde_json::json!(9));
    writer.note_change();
    let wrote = writer
        .maybe_flush(settings.document())
        .unwrap_or_else(|e| panic!("flush: {e}"));
    assert!(!wrote, "400 ms < 500 ms debounce: coalesced");
    assert!(!path.exists(), "nothing written yet");

    // Quiet for the full window: exactly one write with the LAST value.
    clock.advance(500);
    let wrote = writer
        .maybe_flush(settings.document())
        .unwrap_or_else(|e| panic!("flush: {e}"));
    assert!(wrote);
    let wrote_again = writer
        .maybe_flush(settings.document())
        .unwrap_or_else(|e| panic!("flush: {e}"));
    assert!(!wrote_again, "clean documents do not rewrite");
    let saved: serde_json::Value =
        match serde_json::from_slice(&std::fs::read(&path).unwrap_or_default()) {
            Ok(v) => v,
            Err(e) => panic!("parse: {e}"),
        };
    assert_eq!(saved["alvs.general.recentLimit"], serde_json::json!(9));
    assert_eq!(settings.recent_limit(), 9);
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_set_003_typed_getters_cover_the_consumed_keys() {
    let settings = Settings::defaults();
    assert_eq!(settings.autosave_interval_sec(), 60);
    assert_eq!(settings.backups_keep(), 20);
    assert_eq!(settings.recent_limit(), 20);
    assert_eq!(settings.log_level(), "info");
    assert!(settings.redact_diagnostics());
    assert_eq!(settings.cache_folder(), "default");
    assert!(settings.default_project_folder().contains("Videos"));
}
