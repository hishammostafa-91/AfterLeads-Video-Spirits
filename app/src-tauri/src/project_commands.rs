// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The project commands (04 s4.3.2) over the WP-03 store pipeline:
//! new/open (writable, read-only, take-over), save/save-as, close,
//! and recents. One project is open at a time (FR-001). Errors cross
//! as `ErrorEnvelope` - store failures map through `StoreError`
//! (codes + recovery per 12 s12.13), request validation through
//! E1003. `model:dirty` is emitted on every reachable change of the
//! dirty state (04 s4.10) - which is `false` until edit commands
//! exist (WP-12).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use alvs_core::error::ErrorEnvelope;
use alvs_model::project::{Project, new_project};
use alvs_model::sequence::{
    SequenceFormat, fps_drop_frame_default, fps_rate, new_sequence, preset_format,
    sqp02_fallback_format,
};
use alvs_store::error::StoreError;
use alvs_store::lock::LockFile;
use alvs_store::project::{ProjectStore, ReadOnlyReason};
use serde::{Deserialize, Serialize};

use crate::dto::{
    NewProjectRequest, ProjectSnapshot, ReadOnlyReasonKind, ReadOnlyState, RecentProject,
    SaveResult,
};
use crate::project_io::ProjectHooks;
use crate::services::Services;

/// The `model:dirty` event (04 s4.10): payload `{ dirty }`.
pub const MODEL_DIRTY_EVENT: &str = "model:dirty";

/// The one open project (FR-001: one at a time).
struct OpenProject {
    store: ProjectStore,
    doc: Arc<Project>,
    read_only: Option<ReadOnlyState>,
}

/// The managed project state.
#[derive(Default)]
pub struct ProjectState {
    open: Mutex<Option<OpenProject>>,
}

/// The validation envelope for a bad request field (E1003).
fn invalid(detail: String) -> ErrorEnvelope {
    ErrorEnvelope::from_error(&alvs_core::error::AppError::ValidationFailed { detail })
}

/// The E8004 envelope with the documented recovery action (12
/// s12.13: "Choose another location"), mirroring `StoreError`'s
/// mapping for denied/missing locations.
fn location_denied(detail: String) -> ErrorEnvelope {
    let mut envelope = invalid(String::new());
    envelope.code = alvs_core::error::ErrorCode::E8004;
    envelope.severity = alvs_core::error::ErrorCode::E8004.severity();
    envelope.message_key = alvs_core::error::ErrorCode::E8004.message_key();
    envelope.recovery = vec![alvs_core::error::RecoveryAction {
        command_id: "chooseLocation".into(),
        label_key: "action.chooseAnotherLocation".into(),
    }];
    envelope.detail = Some(detail);
    envelope
}

/// A Windows file-name safe project name (DLG-01: 1-200 chars).
pub fn valid_name(name: &str) -> Result<(), String> {
    let len = name.chars().count();
    if !(1..=200).contains(&len) {
        return Err(format!("name must be 1-200 characters, found {len}"));
    }
    if name == "." || name == ".." {
        return Err("name must not be a dot segment".into());
    }
    if name.chars().any(|c| {
        c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    }) {
        return Err("name contains characters Windows file names forbid".into());
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err("name must not end with a dot or a space (Windows)".into());
    }
    Ok(())
}

/// Resolve a new-project request into the initial sequence format
/// (12 s12.8/12.9 closed catalogs; the model's own preset table).
pub fn request_format(request: &NewProjectRequest) -> Result<SequenceFormat, String> {
    let mut format = match request.sequence_preset.as_str() {
        // Auto keeps the SQP-02 fallback with auto-match pending.
        "SQP-01" => sqp02_fallback_format(),
        other => preset_format(other)
            .ok_or_else(|| format!("unknown sequence preset {other} (12 s12.8 closed catalog)"))?,
    };
    let rate = fps_rate(&request.frame_rate).ok_or_else(|| {
        format!(
            "unknown frame rate {} (12 s12.9 closed catalog)",
            request.frame_rate
        )
    })?;
    format.frame_rate = rate;
    format.drop_frame_timecode = fps_drop_frame_default(&request.frame_rate);
    if request.sample_rate != 44_100 && request.sample_rate != 48_000 {
        return Err(format!(
            "sample rate must be 44,100 or 48,000, found {}",
            request.sample_rate
        ));
    }
    format.sample_rate = request.sample_rate;
    Ok(format)
}

/// The project file of a new project: `<folder>\<name>\<name>.alvs`.
pub fn new_project_path(folder: &Path, name: &str) -> PathBuf {
    folder.join(name).join(format!("{name}.alvs"))
}

fn store_for(path: &Path, services: &Services) -> ProjectStore {
    let keep = services
        .settings
        .get("alvs.general.backupsKeep")
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(20)
        .max(1);
    ProjectStore::new(path, keep).with_recent(services.recent.clone())
}

fn snapshot_of(doc: &Project, path: &Path, read_only: Option<ReadOnlyState>) -> ProjectSnapshot {
    ProjectSnapshot {
        project: doc.clone(),
        path: path.to_string_lossy().into_owned(),
        read_only,
    }
}

fn read_only_state(reason: &Option<ReadOnlyReason>) -> Option<ReadOnlyState> {
    match reason {
        None => None,
        Some(ReadOnlyReason::NewerSchema { .. }) => Some(ReadOnlyState {
            reason: ReadOnlyReasonKind::NewerSchema,
            takeover_available: false,
        }),
        Some(ReadOnlyReason::Locked {
            takeover_available, ..
        }) => Some(ReadOnlyState {
            reason: ReadOnlyReasonKind::Locked,
            takeover_available: *takeover_available,
        }),
    }
}

/// Open `path` through the store and install it as the one open
/// project; `accept_locked` distinguishes `project_open` (E1007 on a
/// live lock) from `project_open_readonly` (accepts it).
fn open_and_install(
    app: &tauri::AppHandle,
    state: &ProjectState,
    services: &Services,
    path: &Path,
    accept_locked: bool,
) -> Result<ProjectSnapshot, ErrorEnvelope> {
    use tauri::Emitter as _;

    let store = store_for(path, services);
    let opened = store.open::<ProjectHooks>().map_err(|e| e.envelope())?;
    let read_only = read_only_state(&opened.read_only);
    if let Some(state) = &read_only
        && state.reason == ReadOnlyReasonKind::Locked
        && !accept_locked
    {
        // 04 s4.3.2: `project_open` errors E1007; the UI then chooses
        // open-read-only or take-over via the recovery actions.
        let mut envelope = ReadOnlyReason::Locked {
            pid: 0,
            host: String::new(),
            takeover_available: state.takeover_available,
        }
        .store_error()
        .envelope();
        envelope.params.insert(
            "takeoverAvailable".into(),
            state.takeover_available.to_string(),
        );
        return Err(envelope);
    }
    let snapshot = snapshot_of(&opened.doc, path, read_only);
    let doc = Arc::new(opened.doc);
    let mut open = state
        .open
        .lock()
        .map_err(|_| invalid("project state lock poisoned".into()))?;
    // Replacing an open project drops its lock (03 s3.9.4); the UI
    // gates the dirty case with DLG-20 before calling an open.
    *open = Some(OpenProject {
        store,
        doc,
        read_only: snapshot.read_only,
    });
    let _ = app.emit(MODEL_DIRTY_EVENT, serde_json::json!({ "dirty": false }));
    Ok(snapshot)
}

/// `project_new` - create `<folder>\<name>\<name>.alvs` and open it
/// (DLG-01; E1008/E8004 on save/location failures).
#[tauri::command]
pub fn project_new(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    services: tauri::State<Services>,
    request: NewProjectRequest,
) -> Result<ProjectSnapshot, ErrorEnvelope> {
    valid_name(&request.name).map_err(invalid)?;
    let format = request_format(&request).map_err(invalid)?;
    let folder = Path::new(&request.folder);
    if !folder.is_dir() {
        return Err(location_denied(format!(
            "location is not an existing folder: {}",
            request.folder
        )));
    }
    let path = new_project_path(folder, &request.name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StoreError::from_io(&e).envelope())?;
    }
    if path.exists() {
        return Err(location_denied(format!(
            "a project already exists at {}",
            path.display()
        )));
    }
    // The anatomy's sequence tab is "Main" (07 s7.2).
    let project = new_project(&request.name, new_sequence("Main", format));
    store_for(&path, &services)
        .save::<ProjectHooks>(Arc::new(project))
        .map_err(|e| e.envelope())?;
    open_and_install(&app, &state, &services, &path, false)
}

/// `project_open` - open writable or fail with E1007 (the recovery
/// actions offer read-only / take-over; 12 s12.13).
#[tauri::command]
pub fn project_open(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    services: tauri::State<Services>,
    request: ProjectPathRequest,
) -> Result<ProjectSnapshot, ErrorEnvelope> {
    let path = PathBuf::from(&request.path);
    open_and_install(&app, &state, &services, &path, false)
}

/// `project_open_readonly` - accept a held lock (used after E1007).
#[tauri::command]
pub fn project_open_readonly(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    services: tauri::State<Services>,
    request: ProjectPathRequest,
) -> Result<ProjectSnapshot, ErrorEnvelope> {
    let path = PathBuf::from(&request.path);
    open_and_install(&app, &state, &services, &path, true)
}

/// `project_take_over_lock` - the confirmed take-over (03 s3.9.4):
/// only offered for dead or expired holders.
#[tauri::command]
pub fn project_take_over_lock(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    services: tauri::State<Services>,
    request: ProjectPathRequest,
) -> Result<ProjectSnapshot, ErrorEnvelope> {
    let path = PathBuf::from(&request.path);
    // Probe the current outcome first: a released or already-ours
    // lock simply opens.
    let probe = store_for(&path, &services)
        .open::<ProjectHooks>()
        .map_err(|e| e.envelope())?;
    match probe.read_only {
        None => {
            // Writable already - reinstall through the common path.
            open_and_install(&app, &state, &services, &path, false)
        }
        Some(ReadOnlyReason::NewerSchema { .. }) => {
            open_and_install(&app, &state, &services, &path, true)
        }
        Some(ReadOnlyReason::Locked {
            takeover_available: true,
            ..
        }) => {
            let (_lock, _outcome, _info) = LockFile::take_over(&path).map_err(|e| e.envelope())?;
            open_and_install(&app, &state, &services, &path, false)
        }
        Some(ReadOnlyReason::Locked {
            takeover_available: false,
            ..
        }) => Err(ReadOnlyReason::Locked {
            pid: 0,
            host: String::new(),
            takeover_available: false,
        }
        .store_error()
        .envelope()),
    }
}

/// A path-only request (`project_open` family).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPathRequest {
    /// The `.alvs` file.
    pub path: String,
}

/// `project_save` - save in place; read-only projects cannot save
/// (the banner's Save As is the path, E1008 with its recovery).
#[tauri::command]
pub fn project_save(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
) -> Result<SaveResult, ErrorEnvelope> {
    use tauri::Emitter as _;

    let open = state
        .open
        .lock()
        .map_err(|_| invalid("project state lock poisoned".into()))?;
    let Some(project) = open.as_ref() else {
        return Err(invalid("no project is open".into()));
    };
    if project.read_only.is_some() {
        return Err(StoreError::SaveFailed {
            detail: "the project is open read-only; use Save As".into(),
        }
        .envelope());
    }
    let receipt = project
        .store
        .save::<ProjectHooks>(Arc::clone(&project.doc))
        .map_err(|e| e.envelope())?;
    let _ = app.emit(MODEL_DIRTY_EVENT, serde_json::json!({ "dirty": false }));
    Ok(SaveResult {
        path: receipt.path.to_string_lossy().into_owned(),
        backup_path: receipt.backup.map(|p| p.to_string_lossy().into_owned()),
    })
}

/// `project_save_as` - save to `path` and continue there.
#[tauri::command]
pub fn project_save_as(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    services: tauri::State<Services>,
    request: ProjectPathRequest,
) -> Result<SaveResult, ErrorEnvelope> {
    let path = PathBuf::from(&request.path);
    if let Some(parent) = path.parent()
        && !parent.is_dir()
    {
        return Err(location_denied(format!(
            "folder does not exist: {}",
            parent.display()
        )));
    }
    {
        let open = state
            .open
            .lock()
            .map_err(|_| invalid("project state lock poisoned".into()))?;
        let Some(project) = open.as_ref() else {
            return Err(invalid("no project is open".into()));
        };
        let mut doc = (*project.doc).clone();
        // Save As renames the project to the file stem (03 s3.4.1).
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            doc.name = stem.to_owned();
        }
        store_for(&path, &services)
            .save::<ProjectHooks>(Arc::new(doc))
            .map_err(|e| e.envelope())?;
    }
    // Continue in the new file: open it (lock + recents).
    open_and_install(&app, &state, &services, &path, false)?;
    Ok(SaveResult {
        path: path.to_string_lossy().into_owned(),
        backup_path: None,
    })
}

/// `project_close` - drop the open project and its lock. The UI's
/// DLG-20 gate resolves before calling; a dirty non-discard close is
/// a caller contract violation (E0001), never data loss.
#[tauri::command]
pub fn project_close(
    state: tauri::State<ProjectState>,
    request: CloseRequest,
) -> Result<(), ErrorEnvelope> {
    let mut open = state
        .open
        .lock()
        .map_err(|_| invalid("project state lock poisoned".into()))?;
    let Some(project) = open.as_ref() else {
        // Closing with nothing open is a no-op (idempotent UI reset).
        return Ok(());
    };
    if project_dirty(project) && !request.discard_changes {
        return Err(ErrorEnvelope::from_error(
            &alvs_core::error::AppError::Internal {
                detail: "close on a dirty project without discard; the UI gates with DLG-20 first"
                    .into(),
            },
        ));
    }
    // Dropping releases the lock (03 s3.9.4).
    *open = None;
    Ok(())
}

/// Whether the open project has unsaved changes. No edit commands
/// exist before WP-12, so the reachable state is always clean; the
/// event wiring stays honest for when edits land.
fn project_dirty(_project: &OpenProject) -> bool {
    false
}

/// The close request (04 s4.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseRequest {
    /// True after DLG-20's "Don't save".
    pub discard_changes: bool,
}

/// `project_recent` - the newest-first recents (03 s3.7).
#[tauri::command]
pub fn project_recent(
    services: tauri::State<Services>,
) -> Result<Vec<RecentProject>, ErrorEnvelope> {
    let entries = services.recent.list().map_err(|e| e.envelope())?;
    Ok(entries
        .into_iter()
        .map(|entry| RecentProject {
            path: entry.path.to_string_lossy().into_owned(),
            name: entry.name,
            thumbnail: entry.thumbnail.map(|p| p.to_string_lossy().into_owned()),
            opened_utc: entry.opened_utc,
            pinned: entry.pinned,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{new_project_path, request_format, valid_name};
    use crate::dto::NewProjectRequest;

    fn request(preset: &str, fps: &str, sample: u32) -> NewProjectRequest {
        NewProjectRequest {
            name: "Interviews".into(),
            folder: "C:\\Projects".into(),
            sequence_preset: preset.into(),
            frame_rate: fps.into(),
            sample_rate: sample,
        }
    }

    #[test]
    fn names_follow_the_dlg01_rule() {
        assert!(valid_name("Interviews").is_ok());
        assert!(valid_name("a").is_ok());
        assert!(valid_name(&"x".repeat(200)).is_ok());
        assert!(valid_name("").is_err());
        assert!(valid_name(&"x".repeat(201)).is_err());
        assert!(valid_name("bad:name").is_err());
        assert!(valid_name("bad<name").is_err());
        assert!(valid_name("bad/name").is_err());
        assert!(valid_name("trailing.").is_err());
        assert!(valid_name("trailing ").is_err());
        assert!(valid_name("..").is_err());
    }

    #[test]
    fn auto_preset_keeps_auto_match_others_do_not() {
        let auto = request_format(&request("SQP-01", "FPS-05", 48_000)).expect("auto");
        assert!(auto.auto_match_pending);
        assert_eq!((auto.width, auto.height), (1920, 1080));
        let named = request_format(&request("SQP-06", "FPS-05", 48_000)).expect("named");
        assert!(!named.auto_match_pending);
        assert_eq!((named.width, named.height), (1080, 1920));
    }

    #[test]
    fn fps_and_drop_frame_apply_and_catalogs_close() {
        let format = request_format(&request("SQP-02", "FPS-04", 44_100)).expect("format");
        assert_eq!(format.frame_rate.num, 30_000);
        assert_eq!(format.frame_rate.den, 1001);
        assert!(format.drop_frame_timecode);
        assert_eq!(format.sample_rate, 44_100);
        assert!(request_format(&request("SQP-02", "FPS-09", 48_000)).is_err());
        assert!(request_format(&request("SQP-13", "FPS-05", 48_000)).is_err());
        assert!(request_format(&request("SQP-99", "FPS-05", 48_000)).is_err());
        assert!(request_format(&request("SQP-02", "FPS-05", 22_050)).is_err());
    }

    #[test]
    fn new_projects_nest_in_a_named_folder() {
        let path = new_project_path(Path::new("C:\\Projects"), "Interviews");
        assert_eq!(path, Path::new("C:\\Projects\\Interviews\\Interviews.alvs"));
    }
}
