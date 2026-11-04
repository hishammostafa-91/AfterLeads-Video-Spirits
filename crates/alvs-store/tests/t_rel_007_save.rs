// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-REL-007 (save part, 09 §9.10): disk-full and permission errors on
//! save keep previous files intact with actionable codes (E8001/E8004/
//! E1008). The step-injected (disk-full) cases live in `atomic.rs`'s unit
//! tests (review F-05 structure); this suite proves the real Windows
//! permission constructs end-to-end through the writers that exist today
//! (save, backup); autosave/settings rows join this file in their phases.

use std::path::PathBuf;
use std::sync::Arc;

use alvs_model::project::new_project;
use alvs_model::sequence::{new_sequence, sqp02_fallback_format};
use alvs_store::backup;
use alvs_store::project::ProjectStore;
use alvs_store::recent::{MemoryRecent, RecentProjects};

mod common;

use common::ProjectHooks;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-rel-007-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

fn sample_project() -> alvs_model::project::Project {
    let mut format = sqp02_fallback_format();
    format.auto_match_pending = false;
    new_project("T-REL-007", new_sequence("Main", format))
}

#[test]
fn t_rel_007_read_only_project_file_keeps_previous_content() {
    let dir = scratch("readonly");
    let project_path = dir.join("ro.alvs");
    let recent = Arc::new(MemoryRecent::new(20));
    let store = ProjectStore::new(&project_path, 20).with_recent(recent.clone());
    store
        .save::<ProjectHooks>(Arc::new(sample_project()))
        .unwrap_or_else(|e| panic!("first save: {e}"));
    let before = std::fs::read(&project_path).unwrap_or_default();
    let backups_before = backup::list(&project_path).unwrap_or_default().len();
    let mut perms = std::fs::metadata(&project_path)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&project_path, perms).unwrap_or(());

    let err = match store.save::<ProjectHooks>(Arc::new(sample_project())) {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E8004);
    // REQ-REL-007: previous files intact — the project, its backups, and
    // no temporary residue; the failed save short-circuits recent update.
    assert_eq!(std::fs::read(&project_path).unwrap_or_default(), before);
    assert_eq!(
        backup::list(&project_path).unwrap_or_default().len(),
        backups_before
    );
    assert!(!dir.join("ro.alvs.tmp").exists());
    let listed = match recent.list() {
        Ok(l) => l,
        Err(e) => panic!("recent: {e}"),
    };
    assert_eq!(listed.len(), 1, "only the first save recorded");

    let mut perms = std::fs::metadata(&project_path)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    // Test cleanup on Windows (the lint's concern is Unix-only).
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    std::fs::set_permissions(&project_path, perms).unwrap_or(());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_rel_007_tmp_collision_keeps_previous_content() {
    // The second real Windows construct (research D13/F-04): a directory
    // squatting on the `<target>.tmp` name makes the tmp write fail with
    // ERROR_ACCESS_DENIED.
    let dir = scratch("tmpdir");
    let project_path = dir.join("t.alvs");
    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(sample_project()))
        .unwrap_or_else(|e| panic!("first save: {e}"));
    let before = std::fs::read(&project_path).unwrap_or_default();
    std::fs::create_dir(dir.join("t.alvs.tmp")).unwrap_or(());

    let err = match store.save::<ProjectHooks>(Arc::new(sample_project())) {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E8004);
    assert_eq!(std::fs::read(&project_path).unwrap_or_default(), before);
    assert!(dir.join("t.alvs.tmp").exists(), "the squatter survives");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_rel_007_missing_folder_reports_save_failure() {
    // A vanished project folder surfaces as E1008/E8004 with the previous
    // state simply absent — never a panic (REQ-REL-007's "actionable").
    let dir = scratch("missing");
    let project_path = dir.join("sub").join("m.alvs");
    let store = ProjectStore::new(&project_path, 20);
    let err = match store.save::<ProjectHooks>(Arc::new(sample_project())) {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E1008);
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

// --- Autosave and settings writer rows (T010's full writer coverage) ---

use alvs_store::autosave::{AutosaveDriver, Clock as AutoClock, SnapshotWriter};
use alvs_store::settings::Settings;
use std::sync::atomic::{AtomicI64 as Writes, Ordering as CountOrdering};

struct ClockAt(std::sync::atomic::AtomicI64);

impl AutoClock for ClockAt {
    fn now_ms(&self) -> i64 {
        self.0.load(CountOrdering::SeqCst)
    }
}

struct SidecarWriter {
    sidecar: PathBuf,
    writes: Writes,
}

impl SnapshotWriter for SidecarWriter {
    fn write_snapshot(&self) -> Result<(), alvs_store::error::StoreError> {
        self.writes.fetch_add(1, CountOrdering::SeqCst);
        let roots = [self
            .sidecar
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default()];
        alvs_store::atomic::atomic_write_contained(&self.sidecar, b"snapshot", &roots)
    }
}

#[test]
fn t_rel_007_autosave_permission_failure_keeps_files_intact() {
    let dir = scratch("autosave-ro");
    let project = dir.join("a.alvs");
    std::fs::write(&project, b"main").unwrap_or(());
    let sidecar = alvs_store::project::autosave_path(&project);
    // Pre-existing read-only sidecar: the autosave's atomic replace fails
    // (ERROR_ACCESS_DENIED) while the main file stays untouched.
    std::fs::write(&sidecar, b"previous-sidecar").unwrap_or(());
    let mut perms = std::fs::metadata(&sidecar)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&sidecar, perms).unwrap_or(());

    let clock = Arc::new(ClockAt(std::sync::atomic::AtomicI64::new(0)));
    let writer = Arc::new(SidecarWriter {
        sidecar: sidecar.clone(),
        writes: Writes::new(0),
    });
    let driver = AutosaveDriver::new(1, clock.clone(), writer.clone());
    clock.0.fetch_add(5_000, CountOrdering::SeqCst);
    let err = match driver.tick(true) {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E8004);
    assert_eq!(
        std::fs::read_to_string(&project).unwrap_or_default(),
        "main",
        "the main file is never touched by autosave (REQ-PRJ-003)"
    );
    assert_eq!(
        std::fs::read_to_string(&sidecar).unwrap_or_default(),
        "previous-sidecar",
        "the previous sidecar survives"
    );
    let mut perms = std::fs::metadata(&sidecar)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    // Test cleanup on Windows (the lint's concern is Unix-only).
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    std::fs::set_permissions(&sidecar, perms).unwrap_or(());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_rel_007_settings_write_failure_keeps_previous_file() {
    let dir = scratch("settings-ro");
    let path = dir.join("settings.json");
    let backup = dir.join("settings.json.bak");
    let clock = Arc::new(ClockAt(std::sync::atomic::AtomicI64::new(0)));
    let writer = alvs_store::settings::SettingsWriter::new(&path, &backup, clock.clone(), 500);
    let mut settings = Settings::defaults();
    writer
        .flush(settings.document())
        .unwrap_or_else(|e| panic!("first flush: {e}"));
    let before = std::fs::read(&path).unwrap_or_default();
    let mut perms = std::fs::metadata(&path)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&path, perms).unwrap_or(());

    settings.set("alvs.general.backupsKeep", serde_json::json!(5));
    writer.note_change();
    clock.0.fetch_add(5_000, CountOrdering::SeqCst);
    let err = match writer.maybe_flush(settings.document()) {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E8004);
    assert_eq!(std::fs::read(&path).unwrap_or_default(), before);

    let mut perms = std::fs::metadata(&path)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    // Test cleanup on Windows (the lint's concern is Unix-only).
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    std::fs::set_permissions(&path, perms).unwrap_or(());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
