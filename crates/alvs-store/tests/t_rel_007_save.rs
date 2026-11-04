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
use alvs_store::project::{DocumentHooks, ProjectStore};
use alvs_store::recent::{MemoryRecent, RecentProjects};

struct ProjectHooks;

impl DocumentHooks for ProjectHooks {
    type Doc = alvs_model::project::Project;

    fn current_version() -> u32 {
        alvs_model::migrate::CURRENT_SCHEMA_VERSION
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
    fn validate(doc: &Self::Doc) -> Result<(), alvs_store::error::StoreError> {
        let issues = alvs_model::validate::validate(doc);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(alvs_store::error::StoreError::ValidationFailed {
                detail: format!("{issues:?}"),
            })
        }
    }
    fn check_size(len_bytes: u64) -> Option<String> {
        alvs_model::validate::check_file_size(len_bytes).map(|i| format!("{i:?}"))
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
    fn migrate(bytes: &[u8]) -> Result<Vec<u8>, alvs_store::error::StoreError> {
        let mut value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
            alvs_store::error::StoreError::MigrationFailed {
                detail: e.to_string(),
            }
        })?;
        alvs_model::migrate::migrate(&mut value, &[]).map_err(|e| {
            alvs_store::error::StoreError::MigrationFailed {
                detail: e.to_string(),
            }
        })?;
        serde_json::to_vec_pretty(&value).map_err(|e| {
            alvs_store::error::StoreError::MigrationFailed {
                detail: e.to_string(),
            }
        })
    }
    fn fixup_media_paths(
        _doc: &mut Self::Doc,
        _project_dir: &std::path::Path,
    ) -> Result<(), alvs_store::error::StoreError> {
        Ok(())
    }
}

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
