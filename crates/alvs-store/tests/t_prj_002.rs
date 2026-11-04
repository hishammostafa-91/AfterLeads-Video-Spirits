// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-PRJ-002 (09 §9.10): save/open round-trip, relative-path fallback,
//! atomic write under simulated failure. Proves the whole pipeline through
//! the `DocumentHooks` seam over the real `alvs_model::Project` (research
//! D2 — the dev-dependency is exempt from CMP row checks).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use alvs_model::media::{Fingerprint, MediaItem, MediaKind, Probe};
use alvs_model::migrate::{self, MigrationStep};
use alvs_model::project::new_project;
use alvs_model::sequence::{new_sequence, sqp02_fallback_format};
use alvs_store::error::StoreError;
use alvs_store::project::{
    DocumentHooks, MediaFixup, ProjectStore, ReadOnlyReason, resolve_media_entry,
};
use alvs_store::recent::{MemoryRecent, RecentProjects};

/// The production-shaped hooks over the real model (the app backend's
/// `project_io.rs` implements the same contract).
struct ProjectHooks;

impl DocumentHooks for ProjectHooks {
    type Doc = alvs_model::project::Project;

    fn current_version() -> u32 {
        migrate::CURRENT_SCHEMA_VERSION
    }

    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, StoreError> {
        serde_json::to_vec_pretty(doc).map_err(|e| StoreError::SaveFailed {
            detail: format!("serialization failed: {e}"),
        })
    }

    fn parse(bytes: &[u8]) -> Result<Self::Doc, StoreError> {
        serde_json::from_slice(bytes).map_err(|e| StoreError::ValidationFailed {
            detail: format!("parse failed: {e}"),
        })
    }

    fn validate(doc: &Self::Doc) -> Result<(), StoreError> {
        let issues = alvs_model::validate::validate(doc);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(StoreError::ValidationFailed {
                detail: format!("{issues:?}"),
            })
        }
    }

    fn check_size(len_bytes: u64) -> Option<String> {
        alvs_model::validate::check_file_size(len_bytes).map(|i| format!("{i:?}"))
    }

    fn schema_version(bytes: &[u8]) -> Result<u32, StoreError> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| StoreError::MigrationFailed {
                detail: format!("unreadable document: {e}"),
            })?;
        value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| StoreError::MigrationFailed {
                detail: "missing schemaVersion".into(),
            })
    }

    fn migrate(bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let mut value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| StoreError::MigrationFailed {
                detail: format!("unreadable document: {e}"),
            })?;
        // The shipped registry is empty at schemaVersion = 1 (03 §3.4.1).
        migrate::migrate(&mut value, &[]).map_err(|e| StoreError::MigrationFailed {
            detail: e.to_string(),
        })?;
        serde_json::to_vec_pretty(&value).map_err(|e| StoreError::MigrationFailed {
            detail: format!("reserialization failed: {e}"),
        })
    }

    fn fixup_media_paths(
        doc: &mut Self::Doc,
        project_dir: &Path,
    ) -> Result<MediaFixup, StoreError> {
        // Mirrors the app backend's production hooks: REQ-PRJ-002
        // relative-first resolution with every candidate SEC-04-gated
        // before any access (T032; 05 §5.7) — UNC is never probed
        // (THR-13) and surfaces for the caller's confirmation.
        let mut fixup = MediaFixup::default();
        for item in &mut doc.media {
            let resolution = resolve_media_entry(&item.path, &item.rel_path, project_dir);
            item.path = resolution.path;
            if let Some(path) = resolution.needs_confirmation {
                fixup.needs_confirmation.push(path);
            }
        }
        Ok(fixup)
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-prj-002-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

/// A minimal valid media item (the WP-02 proptest shape); ids must be
/// real UUID v7 strings (03 s3.2.2) to pass model validation.
fn media_item(path: &str, rel_path: &str) -> MediaItem {
    MediaItem {
        id: alvs_core::id::new_id(),
        kind: MediaKind::Video,
        name: "clip.mp4".into(),
        path: path.into(),
        rel_path: rel_path.into(),
        fingerprint: Fingerprint {
            size: 1,
            mtime_utc: "2026-01-01T00:00:00Z".into(),
            head_tail_sha256: "aa".into(),
            extra: Default::default(),
        },
        probe: Probe {
            duration_flicks: 10 * 705_600_000,
            container: "mp4".into(),
            streams: Vec::new(),
            extra: Default::default(),
        },
        image_sequence: None,
        bin_id: None,
        label: None,
        notes: String::new(),
        derived: Default::default(),
        added_utc: "2026-01-01T00:00:00Z".into(),
        extra: Default::default(),
    }
}

fn sample_project() -> alvs_model::project::Project {
    let mut format = sqp02_fallback_format();
    format.auto_match_pending = false;
    new_project("Round-trip", new_sequence("Main", format))
}

#[test]
fn t_prj_002_round_trip_is_lossless_and_records_recent() {
    let dir = scratch("roundtrip");
    let project_path = dir.join("trip.alvs");
    let mut doc = sample_project();
    doc.media
        .push(media_item(&dir.join("a.mp4").to_string_lossy(), "a.mp4"));
    std::fs::write(dir.join("a.mp4"), b"media-bytes").unwrap_or(());

    let recent = Arc::new(MemoryRecent::new(20));
    let store = ProjectStore::new(&project_path, 20).with_recent(recent.clone());
    // A stale autosave sidecar from an earlier session is removed by the
    // successful save (03 §3.9.2).
    std::fs::write(alvs_store::project::autosave_path(&project_path), b"stale").unwrap_or(());
    let receipt = match store.save::<ProjectHooks>(Arc::new(doc.clone())) {
        Ok(r) => r,
        Err(e) => panic!("save: {e}"),
    };
    assert!(receipt.bytes > 0);
    assert!(receipt.backup.is_some());
    assert!(!alvs_store::project::autosave_path(&project_path).exists());

    let opened = match store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    assert_eq!(opened.doc, doc, "round-trip must be lossless");
    assert!(opened.read_only.is_none());
    assert!(opened.migrated_from.is_none());
    match opened.lock {
        Some(lock) => lock.release().unwrap_or(()),
        None => panic!("a writable open acquires the lock"),
    }
    let listed = match recent.list() {
        Ok(l) => l,
        Err(e) => panic!("recent list: {e}"),
    };
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "trip");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_002_relative_path_tried_first_when_absolute_missing() {
    let dir = scratch("relfallback");
    let project_path = dir.join("moved.alvs");
    let mut doc = sample_project();
    // The project folder moved with its media: the absolute path is stale,
    // the relative path resolves (REQ-PRJ-002).
    let missing_abs = dir.join("old-location").join("b.mp4");
    doc.media
        .push(media_item(&missing_abs.to_string_lossy(), "b.mp4"));
    std::fs::write(dir.join("b.mp4"), b"media-bytes").unwrap_or(());

    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(doc))
        .unwrap_or_else(|e| panic!("save: {e}"));
    let opened = match store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    assert_eq!(
        PathBuf::from(&opened.doc.media[0].path),
        dir.join("b.mp4"),
        "the relative path must take over"
    );
    if let Some(lock) = opened.lock {
        lock.release().unwrap_or(());
    }

    // Both paths missing: the item stays as-is and the project still opens.
    let mut offline = sample_project();
    offline.media.push(media_item(
        &dir.join("gone").join("c.mp4").to_string_lossy(),
        "also-gone.mp4",
    ));
    let offline_path = dir.join("offline.alvs");
    let store = ProjectStore::new(&offline_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(offline))
        .unwrap_or_else(|e| panic!("save offline: {e}"));
    let opened = match store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open offline: {e}"),
    };
    assert!(opened.doc.media[0].path.contains("gone"), "stays offline");
    if let Some(lock) = opened.lock {
        lock.release().unwrap_or(());
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_002_media_paths_are_sec04_gated_before_any_access() {
    // T032 (convergence): document-controlled media candidates pass the
    // SEC-04 gate before any probe — UNC is never touched (THR-13) and
    // surfaces on the open; device paths and reserved names stay
    // offline; local media anywhere on disk still resolves.
    let base = scratch("sec04");
    let dir = base.join("proj");
    std::fs::create_dir_all(&dir).unwrap_or(());
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap_or(());
    std::fs::write(dir.join("ok.mp4"), b"x").unwrap_or(());
    std::fs::write(elsewhere.join("e.mp4"), b"x").unwrap_or(());

    let project_path = dir.join("gated.alvs");
    let mut doc = sample_project();
    // UNC absolute: reported, never probed.
    doc.media
        .push(media_item(r"\\alvs-t-prj-002-host\share\a.mp4", ""));
    // Missing absolute + UNC rel_path (the join-base escape): reported,
    // never probed, the item keeps its own path.
    doc.media.push(media_item(
        &dir.join("gone").join("b.mp4").to_string_lossy(),
        r"\\alvs-t-prj-002-host\share\b.mp4",
    ));
    // Device path: silently offline, not a confirmation candidate.
    doc.media.push(media_item(r"\\.\PhysicalDrive1", ""));
    // Reserved name as the relative candidate: stays offline.
    doc.media.push(media_item(
        &dir.join("gone").join("c.mp4").to_string_lossy(),
        "CON",
    ));
    // Local media outside the project folder resolves (containment is
    // the write rule; 05 §5.7).
    doc.media
        .push(media_item(&elsewhere.join("e.mp4").to_string_lossy(), ""));
    // Missing absolute + in-folder relative: the relative takes over.
    doc.media.push(media_item(
        &dir.join("gone").join("d.mp4").to_string_lossy(),
        "ok.mp4",
    ));

    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(doc))
        .unwrap_or_else(|e| panic!("save: {e}"));
    let opened = match store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };

    // The two UNC paths — and only those — surface, sorted and
    // deduplicated; the probe-free open returns promptly (a probe would
    // stall on the nonexistent SMB host).
    assert_eq!(
        opened.media_needs_confirmation,
        vec![
            PathBuf::from(r"\\alvs-t-prj-002-host\share\a.mp4"),
            PathBuf::from(r"\\alvs-t-prj-002-host\share\b.mp4"),
        ]
    );
    assert_eq!(
        opened.doc.media[0].path,
        r"\\alvs-t-prj-002-host\share\a.mp4"
    );
    assert_eq!(
        PathBuf::from(&opened.doc.media[1].path),
        dir.join("gone").join("b.mp4"),
        "the join-escape item keeps its own path"
    );
    assert_eq!(opened.doc.media[2].path, r"\\.\PhysicalDrive1");
    assert_eq!(
        PathBuf::from(&opened.doc.media[3].path),
        dir.join("gone").join("c.mp4")
    );
    assert_eq!(
        PathBuf::from(&opened.doc.media[4].path),
        elsewhere.join("e.mp4")
    );
    assert_eq!(
        PathBuf::from(&opened.doc.media[5].path),
        dir.join("ok.mp4"),
        "the in-folder relative resolution still works"
    );
    if let Some(lock) = opened.lock {
        lock.release().unwrap_or(());
    }
    std::fs::remove_dir_all(&base).unwrap_or(());
}

#[test]
fn t_prj_002_oversized_document_rejected_before_parsing() {
    let dir = scratch("oversize");
    let project_path = dir.join("huge.alvs");
    // Logically extend past the 03 §3.5 rule 15 document limit without
    // writing the bytes (fast, sparse).
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&project_path);
    match file {
        Ok(f) => {
            use std::io::Write;
            let mut f = f;
            f.write_all(b"{\"schemaVersion\":1}").unwrap_or(());
            f.set_len(256 * 1024 * 1024 + 1).unwrap_or(());
        }
        Err(e) => panic!("create: {e}"),
    }
    let store = ProjectStore::new(&project_path, 20);
    let err = match store.open::<ProjectHooks>() {
        Ok(_) => panic!("must reject"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E1003);
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_002_newer_schema_opens_read_only_unknown_fields_kept() {
    let dir = scratch("newer");
    let project_path = dir.join("newer.alvs");
    let doc = sample_project();
    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(doc))
        .unwrap_or_else(|e| panic!("save: {e}"));
    // Simulate a newer version's file: bump the version and add a field we
    // do not know (03 §3.2.3 keeps it in `extra`).
    let bytes = std::fs::read(&project_path).unwrap_or_default();
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    value["schemaVersion"] = serde_json::json!(migrate::CURRENT_SCHEMA_VERSION + 1);
    value["futureField"] = serde_json::json!("from the future");
    std::fs::write(
        &project_path,
        serde_json::to_vec_pretty(&value).unwrap_or_default(),
    )
    .unwrap_or(());

    let opened = match store.open::<ProjectHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    assert_eq!(
        opened.read_only,
        Some(ReadOnlyReason::NewerSchema {
            found: migrate::CURRENT_SCHEMA_VERSION + 1
        })
    );
    assert!(
        opened.lock.is_none(),
        "read-only opens do not lock for writing"
    );
    assert_eq!(
        opened.doc.extra.get("futureField"),
        Some(&serde_json::json!("from the future")),
        "unknown fields survive in memory (03 §3.9.6)"
    );
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

/// Synthetic hooks over raw JSON: real migration steps with versions the
/// shipped registry does not have yet (research F-03: the physical
/// `.v<old>.bak` path is provable today).
struct VersionedHooks;

const V0_TO_V1: &[MigrationStep] = &[MigrationStep {
    from: 0,
    to: 1,
    run: |value| {
        if let Some(obj) = value.as_object_mut() {
            obj.insert("schemaVersion".into(), serde_json::json!(1));
            obj.insert("upgraded".into(), serde_json::json!(true));
        }
        Ok(())
    },
}];

impl DocumentHooks for VersionedHooks {
    type Doc = serde_json::Value;

    fn current_version() -> u32 {
        1
    }
    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, StoreError> {
        serde_json::to_vec_pretty(doc).map_err(|e| StoreError::SaveFailed {
            detail: e.to_string(),
        })
    }
    fn parse(bytes: &[u8]) -> Result<Self::Doc, StoreError> {
        serde_json::from_slice(bytes).map_err(|e| StoreError::ValidationFailed {
            detail: e.to_string(),
        })
    }
    fn validate(_doc: &Self::Doc) -> Result<(), StoreError> {
        Ok(())
    }
    fn check_size(len_bytes: u64) -> Option<String> {
        alvs_model::validate::check_file_size(len_bytes).map(|i| format!("{i:?}"))
    }
    fn schema_version(bytes: &[u8]) -> Result<u32, StoreError> {
        ProjectHooks::schema_version(bytes)
    }
    fn migrate(bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let mut value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| StoreError::MigrationFailed {
                detail: e.to_string(),
            })?;
        migrate::migrate(&mut value, V0_TO_V1).map_err(|e| StoreError::MigrationFailed {
            detail: e.to_string(),
        })?;
        serde_json::to_vec_pretty(&value).map_err(|e| StoreError::MigrationFailed {
            detail: e.to_string(),
        })
    }
    fn fixup_media_paths(_doc: &mut Self::Doc, _dir: &Path) -> Result<MediaFixup, StoreError> {
        Ok(MediaFixup::default())
    }
}

/// Hooks whose migration always fails (E1005 path).
struct FailingMigrationHooks;

impl DocumentHooks for FailingMigrationHooks {
    type Doc = serde_json::Value;
    fn current_version() -> u32 {
        1
    }
    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, StoreError> {
        VersionedHooks::serialize(doc)
    }
    fn parse(bytes: &[u8]) -> Result<Self::Doc, StoreError> {
        VersionedHooks::parse(bytes)
    }
    fn validate(_doc: &Self::Doc) -> Result<(), StoreError> {
        Ok(())
    }
    fn check_size(len_bytes: u64) -> Option<String> {
        VersionedHooks::check_size(len_bytes)
    }
    fn schema_version(bytes: &[u8]) -> Result<u32, StoreError> {
        VersionedHooks::schema_version(bytes)
    }
    fn migrate(_bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        Err(StoreError::MigrationFailed {
            detail: "synthetic step failure".into(),
        })
    }
    fn fixup_media_paths(_doc: &mut Self::Doc, _dir: &Path) -> Result<MediaFixup, StoreError> {
        Ok(MediaFixup::default())
    }
}

#[test]
fn t_prj_002_migration_writes_v_old_bak_and_upgrades() {
    let dir = scratch("migrate");
    let project_path = dir.join("legacy.alvs");
    // A real project document with the version dialed back — the WP-02
    // driver validates that a migrated value parses as a project, so the
    // fixture must be one (research F-03).
    let mut legacy: serde_json::Value = match serde_json::from_slice(
        &serde_json::to_vec_pretty(&sample_project()).unwrap_or_default(),
    ) {
        Ok(v) => v,
        Err(e) => panic!("fixture: {e}"),
    };
    legacy["schemaVersion"] = serde_json::json!(0);
    std::fs::write(
        &project_path,
        serde_json::to_vec_pretty(&legacy).unwrap_or_default(),
    )
    .unwrap_or(());

    let store = ProjectStore::new(&project_path, 20);
    let opened = match store.open::<VersionedHooks>() {
        Ok(o) => o,
        Err(e) => panic!("open: {e}"),
    };
    assert_eq!(opened.migrated_from, Some(0));
    assert_eq!(opened.doc["schemaVersion"], serde_json::json!(1));
    assert_eq!(opened.doc["upgraded"], serde_json::json!(true));
    // The .v0.bak holds the ORIGINAL bytes (03 §3.9.6).
    let bak = dir.join("legacy.alvs.v0.bak");
    let bak_bytes = std::fs::read(&bak).unwrap_or_default();
    let bak_value: serde_json::Value = match serde_json::from_slice(&bak_bytes) {
        Ok(v) => v,
        Err(e) => panic!("bak parse: {e}"),
    };
    assert_eq!(
        bak_value, legacy,
        "the migration backup keeps the original document"
    );
    if let Some(lock) = opened.lock {
        lock.release().unwrap_or(());
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_002_failed_migration_e1005_original_recoverable() {
    let dir = scratch("e1005");
    let project_path = dir.join("legacy.alvs");
    let mut legacy: serde_json::Value = match serde_json::from_slice(
        &serde_json::to_vec_pretty(&sample_project()).unwrap_or_default(),
    ) {
        Ok(v) => v,
        Err(e) => panic!("fixture: {e}"),
    };
    legacy["schemaVersion"] = serde_json::json!(0);
    std::fs::write(
        &project_path,
        serde_json::to_vec_pretty(&legacy).unwrap_or_default(),
    )
    .unwrap_or(());

    let store = ProjectStore::new(&project_path, 20);
    let err = match store.open::<FailingMigrationHooks>() {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert_eq!(err.code(), alvs_core::error::ErrorCode::E1005);
    // The main file is unchanged and the backup exists (E1005: "the
    // original is unchanged").
    let main = std::fs::read(&project_path).unwrap_or_default();
    let main_value: serde_json::Value = match serde_json::from_slice(&main) {
        Ok(v) => v,
        Err(e) => panic!("main parse: {e}"),
    };
    assert_eq!(main_value, legacy, "the original file is untouched");
    assert!(dir.join("legacy.alvs.v0.bak").exists());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_prj_002_real_permission_failure_keeps_previous_file() {
    // T-PRJ-002's "atomic write under simulated failure" at the pipeline
    // level, with the real Windows construct: a read-only target makes the
    // §3.9.1 replace fail; the previous project file survives untouched.
    let dir = scratch("readonly-save");
    let project_path = dir.join("ro.alvs");
    let store = ProjectStore::new(&project_path, 20);
    store
        .save::<ProjectHooks>(Arc::new(sample_project()))
        .unwrap_or_else(|e| panic!("first save: {e}"));
    let before = std::fs::read(&project_path).unwrap_or_default();
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
    assert_eq!(std::fs::read(&project_path).unwrap_or_default(), before);
    assert!(!dir.join("ro.alvs.tmp").exists());

    let mut perms = std::fs::metadata(&project_path)
        .unwrap_or_else(|e| panic!("meta: {e}"))
        .permissions();
    // Test cleanup on Windows (the lint's concern is Unix-only).
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    std::fs::set_permissions(&project_path, perms).unwrap_or(());
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
