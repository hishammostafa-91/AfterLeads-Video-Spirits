// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-PRJ-008 (09 §9.10): lock file creation, read-only, stale takeover —
//! REQ-PRJ-011 over docs/03 §3.9.4. The liveness probe (OD-11) is
//! exercised against a verifiably live pid (our own) and a verifiably
//! free pid; no spawn/kill races (pid recycling makes that flaky).

use std::path::{Path, PathBuf};

use alvs_store::lock::{LockFile, LockInfo, LockOutcome, process_alive};
use alvs_store::project::{DocumentHooks, ProjectStore, ReadOnlyReason};

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-prj-008-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

/// Write a lock body directly, simulating another holder.
fn plant_lock(project: &Path, info: &LockInfo) {
    let path = alvs_store::lock::lock_path(project);
    let bytes = serde_json::to_vec_pretty(info).unwrap_or_default();
    std::fs::write(path, bytes).unwrap_or(());
}

fn foreign_info(pid: u32, host: &str, started_ms: i64) -> LockInfo {
    LockInfo {
        pid,
        host: host.into(),
        user: "someone".into(),
        started_utc: alvs_store::utc::iso_utc(started_ms).unwrap_or_default(),
        started_ms,
        app_version: "0.1.0".into(),
    }
}

/// Find a pid the real probe reports as dead (deterministic precondition:
/// no spawn/kill races in a busy parallel test run).
fn find_dead_pid() -> u32 {
    // Probe candidate pids descending from a high, rarely-used range; the
    // vast majority are free on any workstation.
    for pid in (40000..60000).rev() {
        if !process_alive(pid) {
            return pid;
        }
    }
    panic!("no dead pid found in the probe range")
}

#[test]
fn t_prj_008_exclusive_create_and_live_holder_is_read_only() {
    let dir = scratch("exclusive");
    let project = dir.join("p.alvs");

    let (first, outcome, existing) = match LockFile::acquire(&project) {
        Ok(v) => v,
        Err(e) => panic!("acquire: {e}"),
    };
    assert_eq!(outcome, LockOutcome::Acquired);
    assert!(existing.is_none());
    assert!(alvs_store::lock::lock_path(&project).exists());

    // Our own pid is alive on this host: read-only (E1007), never a
    // silent take-over (03 §3.9.4).
    let (_, second, info) = match LockFile::acquire(&project) {
        Ok(v) => v,
        Err(e) => panic!("second acquire: {e}"),
    };
    assert_eq!(second, LockOutcome::ReadOnlyHeld);
    let info = match info {
        Some(i) => i,
        None => panic!("holder info"),
    };
    assert_eq!(info.pid, std::process::id());
    first.release().unwrap_or_else(|e| panic!("release: {e}"));
    assert!(!alvs_store::lock::lock_path(&project).exists());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_008_dead_holder_offers_takeover() {
    let dir = scratch("dead");
    let project = dir.join("d.alvs");
    // A verifiably dead holder: the real Win32 probe reports the pid free
    // before we plant it, so the take-over path is exercised against a
    // genuinely dead process (03 §3.9.4) without pid-reuse flakiness.
    let pid = find_dead_pid();
    assert!(!process_alive(pid), "precondition: the planted pid is dead");
    plant_lock(
        &project,
        &foreign_info(pid, "THIS-HOST", alvs_store::utc::epoch_ms_now()),
    );

    let (_, outcome, _) = match LockFile::acquire(&project) {
        Ok(v) => v,
        Err(e) => panic!("acquire: {e}"),
    };
    assert_eq!(outcome, LockOutcome::TakeoverAvailable);
    let (lock, outcome, _) = match LockFile::take_over(&project) {
        Ok(v) => v,
        Err(e) => panic!("take over: {e}"),
    };
    assert_eq!(outcome, LockOutcome::Acquired);
    lock.release().unwrap_or_else(|e| panic!("release: {e}"));
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_008_other_host_rules_follow_the_24h_boundary() {
    let dir = scratch("foreign");
    let project = dir.join("f.alvs");
    let now = alvs_store::utc::epoch_ms_now();
    // A verifiably ALIVE holder (our own pid) on another host, so only the
    // 24-hour age rule decides (03 §3.9.4).

    // Older than 24 h: take-over offered.
    plant_lock(
        &project,
        &foreign_info(std::process::id(), "OTHER-HOST", now - 25 * 60 * 60 * 1000),
    );
    let (_, outcome, _) = match LockFile::acquire(&project) {
        Ok(v) => v,
        Err(e) => panic!("acquire: {e}"),
    };
    assert_eq!(outcome, LockOutcome::TakeoverAvailable);

    // Younger than 24 h: read-only only.
    plant_lock(
        &project,
        &foreign_info(std::process::id(), "OTHER-HOST", now - 60 * 60 * 1000),
    );
    let (_, outcome, info) = match LockFile::acquire(&project) {
        Ok(v) => v,
        Err(e) => panic!("acquire: {e}"),
    };
    assert_eq!(outcome, LockOutcome::ReadOnlyHeld);
    assert!(info.is_some());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

// --- Pipeline integration: the open path surfaces the outcomes ---

struct ValueHooks;

impl DocumentHooks for ValueHooks {
    type Doc = serde_json::Value;
    fn current_version() -> u32 {
        1
    }
    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, alvs_store::error::StoreError> {
        serde_json::to_vec_pretty(doc).map_err(|e| alvs_store::error::StoreError::SaveFailed {
            detail: e.to_string(),
        })
    }
    fn parse(bytes: &[u8]) -> Result<Self::Doc, alvs_store::error::StoreError> {
        serde_json::from_slice(bytes).map_err(|e| alvs_store::error::StoreError::ValidationFailed {
            detail: e.to_string(),
        })
    }
    fn validate(_doc: &Self::Doc) -> Result<(), alvs_store::error::StoreError> {
        Ok(())
    }
    fn check_size(_len_bytes: u64) -> Option<String> {
        None
    }
    fn schema_version(bytes: &[u8]) -> Result<u32, alvs_store::error::StoreError> {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
            alvs_store::error::StoreError::MigrationFailed {
                detail: e.to_string(),
            }
        })?;
        value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| alvs_store::error::StoreError::MigrationFailed {
                detail: "missing schemaVersion".into(),
            })
    }
    fn migrate(_bytes: &[u8]) -> Result<Vec<u8>, alvs_store::error::StoreError> {
        Err(alvs_store::error::StoreError::MigrationFailed {
            detail: "unused".into(),
        })
    }
    fn fixup_media_paths(
        _doc: &mut Self::Doc,
        _project_dir: &std::path::Path,
    ) -> Result<(), alvs_store::error::StoreError> {
        Ok(())
    }
}

fn write_project(path: &std::path::Path) {
    let doc = serde_json::json!({ "schemaVersion": 1, "name": "L" });
    std::fs::write(path, serde_json::to_vec_pretty(&doc).unwrap_or_default()).unwrap_or(());
}

#[test]
fn t_prj_008_open_surfaces_locked_read_only_and_takeover() {
    let dir = scratch("pipeline");
    let project = dir.join("pl.alvs");
    write_project(&project);
    let now = alvs_store::utc::epoch_ms_now();

    // A live same-host holder (ourselves): the open is read-only with
    // E1007 and no lock seized (03 §3.9.4).
    let this_host = std::env::var("COMPUTERNAME").unwrap_or_default();
    plant_lock(&project, &foreign_info(std::process::id(), &this_host, now));
    let store = ProjectStore::new(&project, 20);
    let opened = match store.open::<ValueHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    match &opened.read_only {
        Some(ReadOnlyReason::Locked {
            takeover_available, ..
        }) => assert!(!takeover_available, "live holder: read-only only"),
        other => panic!("expected Locked, got {other:?}"),
    }
    assert!(opened.lock.is_none());
    assert_eq!(
        opened.read_only.as_ref().map(|r| r.store_error().code()),
        Some(alvs_core::error::ErrorCode::E1007)
    );

    // An expired other-host holder: take-over is offered as data; the
    // caller confirms (take_over + release of the stale claim), then
    // re-opens writable — the fresh open acquires cleanly.
    plant_lock(
        &project,
        &foreign_info(std::process::id(), "OTHER-HOST", now - 25 * 60 * 60 * 1000),
    );
    let opened = match store.open::<ValueHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    match &opened.read_only {
        Some(ReadOnlyReason::Locked {
            takeover_available, ..
        }) => assert!(takeover_available, "expired holder: take-over offered"),
        other => panic!("expected Locked, got {other:?}"),
    }
    let (lock, outcome, _) = match LockFile::take_over(&project) {
        Ok(v) => v,
        Err(e) => panic!("take over: {e}"),
    };
    assert_eq!(outcome, LockOutcome::Acquired);
    lock.release().unwrap_or_else(|e| panic!("release: {e}"));
    let opened = match store.open::<ValueHooks>() {
        Ok(o) => o,
        Err(e) => panic!("re-open: {e}"),
    };
    assert!(opened.read_only.is_none());
    match opened.lock {
        Some(lock) => lock.release().unwrap_or_else(|e| panic!("release: {e}")),
        None => panic!("writable re-open holds the lock"),
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
