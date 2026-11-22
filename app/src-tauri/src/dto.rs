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
