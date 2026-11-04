// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Shared production-shaped `DocumentHooks` over the real
//! `alvs_model::Project` (research D2) — the single test-side mirror of
//! `app/src-tauri/src/project_io.rs` so the suites cannot drift apart
//! (review F-17). Each suite pulls it in with `mod common;`.

use std::path::Path;

use alvs_model::migrate;
use alvs_model::project::Project;
use alvs_store::error::StoreError;
use alvs_store::project::{DocumentHooks, MediaFixup, resolve_media_entry};

/// The production-shaped hooks (the app backend implements the same
/// contract).
pub struct ProjectHooks;

impl DocumentHooks for ProjectHooks {
    type Doc = Project;

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
            serde_json::from_slice(bytes).map_err(|e| StoreError::ValidationFailed {
                // An unparseable document is corrupt, not unupgradable:
                // E1003, the same verdict `parse` gives (review F-04).
                detail: format!("unreadable document: {e}"),
            })?;
        value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| StoreError::ValidationFailed {
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
        // before any access (T032/T033; 05 §5.7) — UNC (plain or
        // verbatim) is never probed (THR-13) and surfaces for the
        // caller's confirmation.
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
