// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Application DTOs (04 s4.3.1) exported to
//! `app/src/ipc/bindings/` per ADR-010.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The rendering adapter (04 s4.3.1 `AppInfo.renderer`). The engine
/// work package (WP-09) fills the real adapter name; until then the
/// name is the explicit unknown marker, never a fabricated GPU
/// string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RendererInfo {
    /// Adapter name or the literal `"unknown"` until WP-09 reports it.
    pub name: String,
    /// Whether the adapter is WARP (software); `false` only means
    /// "not known to be WARP" until WP-09 reports the adapter.
    pub is_warp: bool,
}

/// The `app_info` response (04 s4.3.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    /// Semver from the package manifest.
    pub version: String,
    /// Short build commit ("unknown" outside a git checkout).
    pub commit: String,
    /// UTC build date (`YYYY-MM-DD`).
    pub build_date: String,
    /// The app identifier (`com.afterleads.videospirits`).
    pub identifier: String,
    /// GPU adapter summary.
    pub renderer: RendererInfo,
    /// Safe mode was requested at launch (02 s2.5 step 1).
    pub safe_mode: bool,
    /// Portable (no-installer) build; the packaging WP (WP-25) owns
    /// the build-time distinction.
    pub portable: bool,
    /// BCP 47 UI locale the app defaults to (REQ-I18N-001: the
    /// Windows display language when supported, else English).
    pub locale: String,
}

/// The `project_new` request (04 s4.3.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewProjectRequest {
    /// Project name (1-200 chars, Windows file-name safe).
    pub name: String,
    /// The parent folder; the project file lands in
    /// `<folder>\<name>\<name>.alvs` (DLG-01).
    pub folder: String,
    /// Sequence preset id (12 s12.8; SQP-01 auto, SQP-02..12 named -
    /// SQP-13 custom needs the explicit-format path, not this
    /// request).
    pub sequence_preset: String,
    /// Frame-rate id (12 s12.9, FPS-01..08).
    pub frame_rate: String,
    /// Audio sample rate: 44,100 or 48,000 (model rule 13).
    pub sample_rate: u32,
}

/// Why an open project is read-only (the banner states of 07 s7.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReadOnlyState {
    /// `"newerSchema"` (E1004) or `"locked"` (E1007 read-only open).
    pub reason: ReadOnlyReasonKind,
    /// Whether a lock take-over is offered (03 s3.9.4; dead or
    /// expired holder). Only meaningful with `locked`.
    pub takeover_available: bool,
}

/// The read-only reason kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReadOnlyReasonKind {
    /// Opened from a newer application version (E1004).
    NewerSchema,
    /// Opened while another holder keeps the lock (E1007).
    Locked,
}

/// The full project state the UI holds (04 s4.3.2 `ProjectSnapshot`:
/// "full normalized state" - the document plus the open context).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectSnapshot {
    /// The project document.
    pub project: alvs_model::project::Project,
    /// The open project file.
    pub path: String,
    /// Set when the project opened read-only.
    pub read_only: Option<ReadOnlyState>,
}

/// The save result (04 s4.3.2 `SaveResult`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SaveResult {
    /// The saved project file.
    pub path: String,
    /// The backup written this save, if any.
    pub backup_path: Option<String>,
}

/// One recent-project row (04 s4.3.2 `project_recent`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecentProject {
    /// Project file path.
    pub path: String,
    /// Display name.
    pub name: String,
    /// Thumbnail reference (written by later packages).
    pub thumbnail: Option<String>,
    /// Last-opened time (ISO-8601 UTC).
    pub opened_utc: String,
    /// Pinned flag.
    pub pinned: bool,
}
