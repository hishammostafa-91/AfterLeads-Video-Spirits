// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-PRJ-004 (09 §9.10): backup creation, pruning, restore — REQ-PRJ-004
//! through the full save pipeline (each manual save writes a timestamped
//! backup; keep the newest `alvs.general.backupsKeep`; restore into a new
//! file). The name-stamp ordering and same-second overwrite are unit-tested
//! in `backup.rs`; this suite proves the user-facing behavior end-to-end.

use std::path::PathBuf;
use std::sync::Arc;

use alvs_model::project::new_project;
use alvs_model::sequence::{new_sequence, sqp02_fallback_format};
use alvs_store::backup;
use alvs_store::project::{DocumentHooks, ProjectStore};

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
        "alvs-t-prj-004-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

fn project_named(name: &str) -> alvs_model::project::Project {
    let mut format = sqp02_fallback_format();
    format.auto_match_pending = false;
    new_project(name, new_sequence("Main", format))
}

#[test]
fn t_prj_004_each_save_writes_a_backup_and_prune_holds_the_limit() {
    let dir = scratch("pipeline");
    let project_path = dir.join("keep20.alvs");
    // backupsKeep range is 1–100 (03 §3.6); use 3 for a fast cycle and
    // distinct names via re-saving projects with different names.
    let store = ProjectStore::new(&project_path, 3);
    for i in 0..5 {
        let doc = project_named(&format!("Save-{i}"));
        store
            .save::<ProjectHooks>(Arc::new(doc))
            .unwrap_or_else(|e| panic!("save {i}: {e}"));
        // Saves inside the same second share a backup name (atomic
        // overwrite); separate the stamps so five backups exist.
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let entries = match backup::list(&project_path) {
        Ok(l) => l,
        Err(e) => panic!("list: {e}"),
    };
    assert_eq!(
        entries.len(),
        3,
        "keep=3 holds after five saves: {entries:?}"
    );
    // The newest backup's content is the LAST save's project.
    let newest = &entries[0];
    let bytes = std::fs::read(&newest.path).unwrap_or_default();
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("backup parse: {e}"),
    };
    assert_eq!(value["name"], serde_json::json!("Save-4"));
    assert!(newest.size > 0);
    assert!(newest.modified_utc.ends_with('Z'));
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_004_restore_lists_and_restores_into_a_new_file() {
    let dir = scratch("restore");
    let project_path = dir.join("restorable.alvs");
    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(project_named("First")))
        .unwrap_or_else(|e| panic!("save: {e}"));
    std::thread::sleep(std::time::Duration::from_secs(1));
    store
        .save::<ProjectHooks>(Arc::new(project_named("Second")))
        .unwrap_or_else(|e| panic!("save: {e}"));

    let entries = match backup::list(&project_path) {
        Ok(l) => l,
        Err(e) => panic!("list: {e}"),
    };
    assert_eq!(entries.len(), 2);
    // The listing carries time and size for every entry (DLG-16).
    assert!(
        entries
            .iter()
            .all(|e| e.size > 0 && !e.modified_utc.is_empty())
    );

    // Restore the OLDEST backup into a new file: "restores into a new
    // file" (REQ-PRJ-004); the backup itself stays untouched.
    let oldest = entries.last().cloned();
    let oldest = match oldest {
        Some(e) => e,
        None => panic!("oldest backup"),
    };
    let target = dir.join("restored-from-backup.alvs");
    let out = match backup::restore(&oldest.path, &target) {
        Ok(p) => p,
        Err(e) => panic!("restore: {e}"),
    };
    assert_eq!(out, target);
    let bytes = std::fs::read(&target).unwrap_or_default();
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("restored parse: {e}"),
    };
    assert_eq!(value["name"], serde_json::json!("First"));
    assert!(oldest.path.exists(), "the backup is untouched");
    // The restored file is a normal project: it opens.
    let restored_store = ProjectStore::new(&target, 20);
    let opened = match restored_store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open restored: {e}"),
    };
    assert!(opened.read_only.is_none());
    if let Some(lock) = opened.lock {
        lock.release().unwrap_or(());
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
