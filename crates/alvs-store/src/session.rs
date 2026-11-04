// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Session marker and crash recovery (docs/03 §3.9.5; REQ-REL-001
//! services): the marker is created at startup and removed only on a
//! clean exit; a leftover marker plus an autosave newer than the main
//! file yields the recovery offer (both timestamps) that DLG-15 renders.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::atomic::atomic_write;
use crate::error::StoreError;
use crate::paths::AppPaths;
use crate::utc;

/// The `session.marker` body (03 §3.8; §3.9.5) — exactly the canonical
/// fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MarkerBody {
    pid: u32,
    #[serde(rename = "startedUtc")]
    started_utc: String,
}

/// A live session marker (03 §3.9.5).
#[derive(Debug)]
pub struct SessionMarker {
    path: PathBuf,
}

impl SessionMarker {
    /// Create the marker at startup (02 §2.5 step 3/9).
    pub fn create(paths: &AppPaths) -> Result<Self, StoreError> {
        let path = paths.session_marker();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StoreError::from_io(&e))?;
        }
        let now = utc::epoch_ms_now();
        let body = MarkerBody {
            pid: std::process::id(),
            started_utc: utc::iso_utc(now).unwrap_or_default(),
        };
        let bytes = serde_json::to_vec_pretty(&body).map_err(|e| StoreError::SaveFailed {
            detail: format!("marker serialization failed: {e}"),
        })?;
        atomic_write(&path, &bytes)?;
        Ok(Self { path })
    }

    /// Whether a marker exists (an abnormal previous exit, §3.9.5).
    pub fn exists(paths: &AppPaths) -> bool {
        paths.session_marker().exists()
    }

    /// Remove the marker on a clean exit (02 §2.5 shutdown step 6).
    pub fn clear(self) -> Result<(), StoreError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::from_io(&e)),
        }
    }
}

/// The DLG-15 data (REQ-REL-001): both timestamps, ISO-8601 UTC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryOffer {
    /// When the autosave sidecar was written.
    pub autosave_utc: String,
    /// When the main file was last saved; empty when the crash happened
    /// before the first save (no main file ever existed).
    pub saved_utc: String,
}

/// The §3.9.5 predicate: an offer exists only when the previous session
/// ended abnormally (the marker was left behind) **and** the autosave is
/// newer than the main file — timestamps are compared, never existence.
/// A crash **before the first save** (no main file at all) still offers
/// recovery: the sidecar is trivially the newest state (G-16), and
/// `saved_utc` is empty — "never saved" (review F-06).
pub fn evaluate(marker_present: bool, main: &Path, autosave: &Path) -> Option<RecoveryOffer> {
    if !marker_present {
        return None;
    }
    let autosave_ms = modified_ms(autosave)?;
    let main_exists = main.exists();
    let saved_ms = if main_exists { modified_ms(main)? } else { 0 };
    if autosave_ms <= saved_ms {
        return None;
    }
    Some(RecoveryOffer {
        autosave_utc: utc::iso_utc(autosave_ms)?,
        saved_utc: if main_exists {
            utc::iso_utc(saved_ms)?
        } else {
            String::new()
        },
    })
}

/// File modification time in epoch ms.
fn modified_ms(path: &Path) -> Option<i64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_requires_marker_and_newer_autosave() {
        let dir =
            std::env::temp_dir().join(format!("alvs-session-{}", uuid::Uuid::now_v7().simple()));
        std::fs::create_dir_all(&dir).unwrap_or(());
        let main = dir.join("m.alvs");
        let sidecar = dir.join("m.alvs.autosave");
        // No sidecar: nothing to offer even with a marker.
        std::fs::write(&main, b"x").unwrap_or(());
        assert!(evaluate(true, &main, &sidecar).is_none());
        // Sidecar older than main: nothing (stale sidecar after a save).
        std::fs::write(&sidecar, b"old").unwrap_or(());
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&main, b"saved after").unwrap_or(());
        assert!(evaluate(true, &main, &sidecar).is_none());
        // Sidecar newer, but no marker: nothing.
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&sidecar, b"fresh").unwrap_or(());
        assert!(evaluate(false, &main, &sidecar).is_none());
        // Marker + newer sidecar: both timestamps.
        let offer = evaluate(true, &main, &sidecar);
        assert!(offer.is_some());
        // A crash before the FIRST save (no main file): the sidecar is
        // trivially newest — the offer exists with an empty `saved_utc`
        // (review F-06).
        std::fs::remove_file(&main).unwrap_or(());
        let offer = match evaluate(true, &main, &sidecar) {
            Some(o) => o,
            None => panic!("a never-saved session still offers recovery"),
        };
        assert!(!offer.autosave_utc.is_empty());
        assert!(offer.saved_utc.is_empty(), "never saved");
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }
}
