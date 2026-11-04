// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-PRJ-003 (09 §9.10): autosave timing and content (REQ-PRJ-003,
//! NFR-REL-02) and the crash-recovery service half of REQ-REL-001
//! (session marker + recovery offer, 03 §3.9.5). Timing is driven by an
//! injected clock; file-age ordering uses real write order (NTFS mtime
//! resolution is far finer than the separations used here).

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use alvs_store::autosave::{AutosaveDriver, AutosaveHandle, Clock, SnapshotWriter};
use alvs_store::error::StoreError;
use alvs_store::paths::AppPaths;
use alvs_store::project::autosave_path;
use alvs_store::session::{self, SessionMarker};

/// A deterministic clock the tests advance by hand.
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

/// A writer that snapshots fixed bytes through the atomic primitive.
struct BytesWriter {
    sidecar: PathBuf,
    bytes: Vec<u8>,
    writes: AtomicI64,
}

impl SnapshotWriter for BytesWriter {
    fn write_snapshot(&self) -> Result<(), StoreError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        alvs_store::atomic::atomic_write(&self.sidecar, &self.bytes)
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-prj-003-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

#[test]
fn t_prj_003_autosave_fires_exactly_on_dirty_and_interval() {
    let dir = scratch("timing");
    let project = dir.join("doc.alvs");
    std::fs::write(&project, b"main-file-bytes").unwrap_or(());
    let clock = Arc::new(FakeClock::new(0));
    let writer = Arc::new(BytesWriter {
        sidecar: autosave_path(&project),
        bytes: b"snapshot".to_vec(),
        writes: AtomicI64::new(0),
    });
    // Default interval 60 s (alvs.general.autosaveIntervalSec, 03 §3.6);
    // a fresh session's baseline is the driver's start time.
    let driver = Arc::new(AutosaveDriver::new(60, clock.clone(), writer.clone()));

    // Clean project: never writes, however much time passes.
    clock.advance(10 * 60 * 1000);
    driver.tick(false).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 0);
    // A manual save also resets the clock (§3.9.2): the baseline moves to
    // now, so a dirty project waits a full interval again.
    driver.note_external_write();
    clock.advance(59 * 1000);
    driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 0, "59 s: skip");
    // One more second: write (§3.9.3 dirty ∧ elapsed ≥ interval).
    clock.advance(1_000);
    let wrote = driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert!(wrote);
    assert_eq!(writer.writes.load(Ordering::SeqCst), 1);
    // Still dirty: a full interval must elapse before the next write.
    let wrote = driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert!(!wrote, "no double-write inside one interval");
    clock.advance(60 * 1000);
    let wrote = driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert!(wrote);
    assert_eq!(writer.writes.load(Ordering::SeqCst), 2);

    // The main file is never touched (REQ-PRJ-003) and the sidecar holds
    // the snapshot bytes (T-PRJ-003 "content").
    assert_eq!(
        std::fs::read_to_string(&project).unwrap_or_default(),
        "main-file-bytes"
    );
    assert_eq!(
        std::fs::read_to_string(autosave_path(&project)).unwrap_or_default(),
        "snapshot"
    );
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_003_custom_interval_from_settings_range() {
    // The interval comes from alvs.general.autosaveIntervalSec (15–600,
    // 03 §3.6); 15 (the minimum) must behave exactly.
    let dir = scratch("interval15");
    let project = dir.join("d.alvs");
    std::fs::write(&project, b"m").unwrap_or(());
    let clock = Arc::new(FakeClock::new(0));
    let writer = Arc::new(BytesWriter {
        sidecar: autosave_path(&project),
        bytes: b"s".to_vec(),
        writes: AtomicI64::new(0),
    });
    let driver = AutosaveDriver::new(15, clock.clone(), writer.clone());
    driver.note_external_write();
    clock.advance(14 * 1000);
    driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 0, "14 s: skip");
    clock.advance(1_000);
    driver.tick(true).unwrap_or_else(|e| panic!("tick: {e}"));
    assert_eq!(writer.writes.load(Ordering::SeqCst), 1, "15 s: write");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_003_production_thread_writes_and_stops() {
    let dir = scratch("thread");
    let project = dir.join("t.alvs");
    std::fs::write(&project, b"m").unwrap_or(());
    let writer = Arc::new(BytesWriter {
        sidecar: autosave_path(&project),
        bytes: b"snap".to_vec(),
        writes: AtomicI64::new(0),
    });
    let driver = Arc::new(AutosaveDriver::new(
        1,
        Arc::new(alvs_store::autosave::SystemClock),
        writer.clone(),
    ));
    let dirty = Arc::new(AtomicBool::new(true));
    let handle: AutosaveHandle = AutosaveHandle::spawn(driver, dirty, 20);
    // Within a generous window the 1-second driver must have written at
    // least once (a real-clock smoke of the production loop only — the
    // timing CONTRACT is covered by the fake-clock tests above).
    let mut saw = false;
    for _ in 0..200 {
        if writer.writes.load(Ordering::SeqCst) >= 1 {
            saw = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(saw, "the spawned driver must autosave a dirty project");
    assert!(
        std::fs::read_to_string(&project).unwrap_or_default() == *"m",
        "the main file stays untouched"
    );
    drop(handle); // stops the thread
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_003_session_marker_lifecycle_and_recovery_offer() {
    let dir = scratch("marker");
    let (roaming, local, temp, home) = (
        dir.join("roaming"),
        dir.join("local"),
        dir.join("tmp"),
        dir.join("home"),
    );
    for d in [&roaming, &local, &temp, &home] {
        std::fs::create_dir_all(d).unwrap_or(());
    }
    let paths = match AppPaths::installed(&roaming, &local, &temp, &home) {
        Ok(p) => p,
        Err(e) => panic!("paths: {e}"),
    };

    // Clean exit removes the marker (03 §3.9.5).
    let marker = match SessionMarker::create(&paths) {
        Ok(m) => m,
        Err(e) => panic!("create: {e}"),
    };
    assert!(SessionMarker::exists(&paths));
    marker.clear().unwrap_or_else(|e| panic!("clear: {e}"));
    assert!(!SessionMarker::exists(&paths));

    // Crash: the marker survives; the offer is made only when the
    // autosave is NEWER than the main file (03 §3.9.5 — timestamps are
    // compared, not existence). Age ordering comes from write order.
    let marker = match SessionMarker::create(&paths) {
        Ok(m) => m,
        Err(e) => panic!("create: {e}"),
    };
    let project = dir.join("crash.alvs");
    let sidecar = autosave_path(&project);

    // No sidecar at all: no offer.
    assert!(session::evaluate(true, &project, &sidecar).is_none());

    // Sidecar written FIRST (older): no offer — the crash left a stale
    // sidecar newer-than check must fail.
    std::fs::write(&sidecar, b"older").unwrap_or(());
    std::thread::sleep(std::time::Duration::from_millis(30));
    std::fs::write(&project, b"saved-later").unwrap_or(());
    assert!(session::evaluate(true, &project, &sidecar).is_none());

    // Sidecar written LAST (newer): the offer with BOTH timestamps
    // (REQ-REL-001; DLG-15 data).
    std::thread::sleep(std::time::Duration::from_millis(30));
    std::fs::write(&sidecar, b"newer-snapshot").unwrap_or(());
    let offer = match session::evaluate(true, &project, &sidecar) {
        Some(o) => o,
        None => panic!("offer expected"),
    };
    assert!(offer.autosave_utc.ends_with('Z'));
    assert!(offer.saved_utc.ends_with('Z'));

    // Marker absent (clean previous exit): never offer.
    assert!(session::evaluate(false, &project, &sidecar).is_none());
    marker.clear().unwrap_or_else(|e| panic!("clear: {e}"));
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
