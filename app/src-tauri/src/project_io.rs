// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The production `DocumentHooks` implementation for `alvs_model::Project`
//! (WP-03, research D2 / owner Option A): the app backend (CMP-12) is the
//! row-legal place where `alvs-store`'s generic save/open pipeline meets
//! the model. Pure delegation — no file I/O lives here.

use std::path::Path;

use alvs_model::migrate;
use alvs_model::project::Project;
use alvs_store::error::StoreError;
use alvs_store::project::{DocumentHooks, MediaFixup, resolve_media_entry};

/// `DocumentHooks` over the real project model.
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
                // E1003, the same verdict `parse` gives (review F-04) —
                // `open` consults the version before parsing, so a
                // corrupt file must surface the restore-from-backup
                // action, not "report a problem".
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
        // The shipped registry is empty at schemaVersion = 1 (03 §3.4.1);
        // future versions append their steps here.
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
        // REQ-PRJ-002: the relative path is tried first when the absolute
        // path is missing; every candidate is SEC-04-gated before any
        // access (T032; 05 §5.7) — UNC paths from the document are never
        // touched (THR-13) and surface for the caller's confirmation
        // (dialog: WP-05); device paths and reserved names stay offline;
        // both-missing items stay offline (E2001 is the model's concern)
        // without blocking the open.
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
