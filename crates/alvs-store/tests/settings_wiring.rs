// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T029 (a): settings values drive every service (autosave interval,
//! backup keep-count, recent limit, log level, redaction flag) — the
//! typed getters are the wiring point the app composes through.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use alvs_store::autosave::{AutosaveDriver, Clock, SnapshotWriter};
use alvs_store::backup;
use alvs_store::error::StoreError;
use alvs_store::project::autosave_path;
use alvs_store::recent::{MemoryRecent, RecentProjects};
use alvs_store::settings::Settings;

struct FakeClock(AtomicI64);

impl Clock for FakeClock {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct CountingWriter {
    sidecar: PathBuf,
    writes: AtomicI64,
}

impl SnapshotWriter for CountingWriter {
    fn write_snapshot(&self) -> Result<(), StoreError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        let roots = [self
            .sidecar
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default()];
        alvs_store::atomic::atomic_write_contained(&self.sidecar, b"s", &roots)
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-wiring-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

#[test]
fn settings_values_reach_every_service() {
    let dir = scratch("wiring");
    let path = dir.join("settings.json");
    let backup_path = dir.join("settings.json.bak");
    // A settings file with three overrides (all within §3.6 ranges).
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "settingsVersion": 1,
            "alvs.general.autosaveIntervalSec": 15,
            "alvs.general.backupsKeep": 2,
            "alvs.general.recentLimit": 5,
            "alvs.advanced.logLevel": "debug",
            "alvs.advanced.redactDiagnostics": false,
        }))
        .unwrap_or_default(),
    )
    .unwrap_or(());
    let settings = Settings::load(&path, &backup_path);
    assert!(settings.notices().is_empty());

    // Autosave: the interval from settings drives the driver.
    let clock = Arc::new(FakeClock(AtomicI64::new(0)));
    let writer = Arc::new(CountingWriter {
        sidecar: autosave_path(&dir.join("w.alvs")),
        writes: AtomicI64::new(0),
    });
    let driver = AutosaveDriver::new(
        settings.autosave_interval_sec(),
        clock.clone(),
        writer.clone(),
    );
    driver.note_external_write();
    clock.0.fetch_add(14_000, Ordering::SeqCst);
    driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 0, "14 s < 15 s");
    clock.0.fetch_add(1_000, Ordering::SeqCst);
    driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 1, "15 s fires");

    // Backups: the keep-count from settings drives pruning.
    let project = dir.join("w.alvs");
    for i in 0..4i64 {
        backup::write_backup(&project, b"b", 1_700_000_000_000 + i * 1000)
            .map(drop)
            .unwrap_or_else(|e| panic!("backup {i}: {e}"));
    }
    let removed =
        backup::prune(&project, settings.backups_keep()).unwrap_or_else(|e| panic!("prune: {e}"));
    assert_eq!(removed.len(), 2, "keep=2 removes the oldest two");
    assert_eq!(
        backup::list(&project).unwrap_or_default().len(),
        2,
        "the settings keep-count holds"
    );

    // Recent: the limit from settings drives trimming.
    let recent = MemoryRecent::new(settings.recent_limit() as usize);
    for name in ["a", "b", "c", "d", "e", "f"] {
        recent
            .record(&dir.join(format!("{name}.alvs")), name, None)
            .unwrap_or_else(|e| panic!("record {name}: {e}"));
    }
    let list = recent.list().unwrap_or_else(|e| panic!("list: {e}"));
    assert_eq!(list.len(), 5, "recentLimit=5 holds");
    assert_eq!(list[0].name, "f");

    // Logging + redaction: the level and flag are read as strings/bools
    // (the subscriber itself is proven in t_rel_003).
    assert_eq!(settings.log_level(), "debug");
    assert!(!settings.redact_diagnostics());

    std::fs::remove_dir_all(&dir).unwrap_or(());
}
