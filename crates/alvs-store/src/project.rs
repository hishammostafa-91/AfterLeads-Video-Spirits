// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The project save/open pipeline (docs/03 §3.9.2, §3.9.6; REQ-PRJ-002),
//! generic over the [`DocumentHooks`] seam (research D2, owner Option A):
//! `alvs-store` stays CMP-01-only; the model-aware halves live behind the
//! trait, implemented for `alvs_model::Project` in the app backend
//! (`app/src-tauri/src/project_io.rs`) and in this crate's integration
//! tests via the dev-dependency.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::atomic::atomic_write;
use crate::backup;
use crate::error::StoreError;
use crate::lock::{LockFile, LockOutcome};
use crate::recent::RecentProjects;

/// The model-aware halves of the pipeline (research D2). Every method is
/// pure with respect to the store: no file I/O happens inside the hooks.
pub trait DocumentHooks {
    /// The document type (e.g. `alvs_model::Project`).
    type Doc: Send + Sync + 'static;

    /// The schema version this build writes (e.g.
    /// `alvs_model::migrate::CURRENT_SCHEMA_VERSION`).
    fn current_version() -> u32;

    /// Serialize a snapshot to the `.alvs` byte form (03 §3.2 conventions).
    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, StoreError>;

    /// Parse bytes into a document, leniently keeping unknown fields
    /// (03 §3.2.3; §3.9.6 read-only path relies on this).
    fn parse(bytes: &[u8]) -> Result<Self::Doc, StoreError>;

    /// Validate a parsed document (03 §3.5); `Err` carries the issues.
    fn validate(doc: &Self::Doc) -> Result<(), StoreError>;

    /// The raw document size gate (03 §3.5 rule 15; 05 SEC-03): `Some`
    /// means the document is rejected before parsing. Implementations
    /// delegate to the model's size hook.
    fn check_size(len_bytes: u64) -> Option<String>;

    /// Read the raw document's `schemaVersion` (missing → `Err`).
    fn schema_version(bytes: &[u8]) -> Result<u32, StoreError>;

    /// Migrate older bytes to the current version; the caller has already
    /// copied the original to `<Name>.alvs.v<old>.bak`. Failure maps to
    /// E1005 and must leave the original file recoverable (§3.9.6).
    fn migrate(bytes: &[u8]) -> Result<Vec<u8>, StoreError>;

    /// Resolve media paths after parsing: when an item's absolute path is
    /// missing, its project-relative path is tried first (REQ-PRJ-002).
    fn fixup_media_paths(doc: &mut Self::Doc, project_dir: &Path) -> Result<(), StoreError>;
}

/// Why an opened project is read-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnlyReason {
    /// The file is from a newer application version (E1004; §3.9.6).
    NewerSchema {
        /// The version found in the file.
        found: u32,
    },
    /// The project is open in another window or computer (E1007;
    /// §3.9.4). When `takeover_available` holds, the holder is gone or
    /// expired and the caller may confirm a take-over
    /// ([`crate::lock::LockFile::take_over`]) and re-open writable.
    Locked {
        /// The holder's process id.
        pid: u32,
        /// The holder's host.
        host: String,
        /// Whether a take-over is offered (dead holder, or another host
        /// with a lock older than 24 h).
        takeover_available: bool,
    },
}

impl ReadOnlyReason {
    /// The store error to surface for this reason (E1004/E1007).
    pub fn store_error(&self) -> StoreError {
        match self {
            ReadOnlyReason::NewerSchema { found } => StoreError::ReadOnlyNewer { found: *found },
            ReadOnlyReason::Locked {
                pid,
                host,
                takeover_available,
            } => StoreError::Locked {
                detail: format!(
                    "pid {pid} on {host}{}",
                    if *takeover_available {
                        " (take-over available)"
                    } else {
                        ""
                    }
                ),
            },
        }
    }
}

/// A successfully opened project (§3.9.2/§3.9.6 outcomes).
pub struct Opened<Doc> {
    /// The parsed (and, unless newer-schema, validated) document.
    pub doc: Doc,
    /// The project file.
    pub path: PathBuf,
    /// The acquired lock; `None` when the open is read-only (E1004).
    pub lock: Option<LockFile>,
    /// Set when the project opened read-only.
    pub read_only: Option<ReadOnlyReason>,
    /// The version a successful migration started from (§3.9.6).
    pub migrated_from: Option<u32>,
}

/// What a manual save did (§3.9.2), for evidence and the recent list.
pub struct SaveReceipt {
    /// The saved project file.
    pub path: PathBuf,
    /// Serialized size in bytes.
    pub bytes: usize,
    /// The backup written this save, if any.
    pub backup: Option<PathBuf>,
}

/// The save/open pipeline for one project file.
pub struct ProjectStore {
    path: PathBuf,
    keep_backups: u32,
    recent: Option<Arc<dyn RecentProjects>>,
}

/// The autosave sidecar path (03 §3.8): `<Name>.alvs.autosave`.
pub fn autosave_path(project: &Path) -> PathBuf {
    with_suffix(project, ".autosave")
}

/// Append a suffix to the project file's name, keeping it in the same
/// folder (03 §3.8 sidecars).
fn with_suffix(project: &Path, suffix: &str) -> PathBuf {
    let mut name = project
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str(suffix);
    project
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name)
}

/// The `.v<old>.bak` migration backup path (§3.9.6).
fn migration_backup(project: &Path, from: u32) -> PathBuf {
    with_suffix(project, &format!(".v{from}.bak"))
}

impl ProjectStore {
    /// A store for `path` keeping `keep_backups` backups
    /// (`alvs.general.backupsKeep`).
    pub fn new(path: &Path, keep_backups: u32) -> Self {
        Self {
            path: path.to_path_buf(),
            keep_backups,
            recent: None,
        }
    }

    /// Attach the recent-projects service (recorded on save and open).
    pub fn with_recent(mut self, recent: Arc<dyn RecentProjects>) -> Self {
        self.recent = Some(recent);
        self
    }

    fn display_name(&self) -> String {
        self.path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    fn record_recent(&self) -> Result<(), StoreError> {
        if let Some(recent) = &self.recent {
            recent.record(&self.path, &self.display_name(), None)?;
        }
        Ok(())
    }

    /// Save per 03 §3.9.2: serialize the snapshot on a background thread →
    /// atomic write → backup → prune → delete the autosave sidecar →
    /// update recent projects.
    pub fn save<H: DocumentHooks>(&self, snapshot: Arc<H::Doc>) -> Result<SaveReceipt, StoreError> {
        // Serialization happens on a background thread (§3.9.2); the Arc
        // snapshot keeps the caller editing meanwhile.
        let handle = std::thread::spawn(move || H::serialize(&snapshot));
        let bytes = match handle.join() {
            Ok(result) => result?,
            Err(_) => {
                return Err(StoreError::SaveFailed {
                    detail: "serializer thread panicked".into(),
                });
            }
        };
        atomic_write(&self.path, &bytes)?;
        let backup_path = backup::write_backup(&self.path, &bytes, crate::utc::epoch_ms_now())?;
        backup::prune(&self.path, self.keep_backups)?;
        // The sidecar exists only while dirty (03 §3.8): a successful save
        // removes it (§3.9.2).
        let sidecar = autosave_path(&self.path);
        match std::fs::remove_file(&sidecar) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(StoreError::from_io(&e)),
        }
        self.record_recent()?;
        Ok(SaveReceipt {
            path: self.path.clone(),
            bytes: bytes.len(),
            backup: Some(backup_path),
        })
    }

    /// Open per 03 §3.9.6: existence + raw size gate → parse → migrate
    /// (with `.v<old>.bak` first) → validate → lock → media fixup. A
    /// newer-schema file opens read-only with E1004 semantics and unknown
    /// fields kept.
    pub fn open<H: DocumentHooks>(&self) -> Result<Opened<H::Doc>, StoreError> {
        let len = std::fs::metadata(&self.path)
            .map_err(|e| StoreError::from_io(&e))?
            .len();
        if let Some(reason) = H::check_size(len) {
            return Err(StoreError::ValidationFailed { detail: reason });
        }
        let bytes = std::fs::read(&self.path).map_err(|e| StoreError::from_io(&e))?;
        let version = H::schema_version(&bytes)?;
        let current = H::current_version();

        // Newer file: read-only, unknown fields kept (§3.9.6; E1004).
        if version > current {
            let doc = H::parse(&bytes)?;
            self.record_recent()?;
            return Ok(Opened {
                doc,
                path: self.path.clone(),
                lock: None,
                read_only: Some(ReadOnlyReason::NewerSchema { found: version }),
                migrated_from: None,
            });
        }

        // Older file: copy the original aside, then migrate (§3.9.6). The
        // main file itself is only rewritten on the next save.
        let (bytes, migrated_from) = if version < current {
            let bak = migration_backup(&self.path, version);
            atomic_write(&bak, &bytes)?;
            let migrated = H::migrate(&bytes)?;
            (migrated, Some(version))
        } else {
            (bytes, None)
        };

        let mut doc = H::parse(&bytes)?;
        H::validate(&doc)?;
        // §3.9.4: a live same-host holder offers read-only (E1007); a
        // dead or expired holder offers take-over — both as data for the
        // caller's confirmation, never a silent seizure.
        let (lock, read_only) = match LockFile::acquire(&self.path)? {
            (lock, LockOutcome::Acquired, _) => (Some(lock), None),
            (_, LockOutcome::ReadOnlyHeld, Some(info)) => (
                None,
                Some(ReadOnlyReason::Locked {
                    pid: info.pid,
                    host: info.host,
                    takeover_available: false,
                }),
            ),
            (_, LockOutcome::TakeoverAvailable, Some(info)) => (
                None,
                Some(ReadOnlyReason::Locked {
                    pid: info.pid,
                    host: info.host,
                    takeover_available: true,
                }),
            ),
            // acquire always reads the info on the existing-lock path;
            // unreachable in practice, treated conservatively as held.
            (_, outcome, None) => (
                None,
                Some(ReadOnlyReason::Locked {
                    pid: 0,
                    host: String::new(),
                    takeover_available: outcome == LockOutcome::TakeoverAvailable,
                }),
            ),
        };
        let project_dir = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        H::fixup_media_paths(&mut doc, &project_dir)?;
        self.record_recent()?;
        Ok(Opened {
            doc,
            path: self.path.clone(),
            lock,
            read_only,
            migrated_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_paths_stay_in_the_project_folder() {
        assert_eq!(
            autosave_path(Path::new(r"C:\proj\demo.alvs")),
            PathBuf::from(r"C:\proj\demo.alvs.autosave")
        );
        assert_eq!(
            migration_backup(Path::new(r"C:\proj\demo.alvs"), 0),
            PathBuf::from(r"C:\proj\demo.alvs.v0.bak")
        );
        assert_eq!(
            with_suffix(Path::new("bare.alvs"), ".x"),
            PathBuf::from("bare.alvs.x")
        );
    }
}
