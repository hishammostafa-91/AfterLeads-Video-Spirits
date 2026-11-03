//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Top-level project, settings, and view state (docs/03 §3.4.1, §3.4.11).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::asset::Asset;
use crate::bin::Bin;
use crate::media::MediaItem;
use crate::sequence::Sequence;

/// The `.alvs` top-level document (03 §3.4.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Project {
    /// Exactly `afterleads-video-spirits.project`.
    pub format: String,
    /// 1 for release 1.0; migrations are forward-only (03 §3.9.6).
    pub schema_version: u32,
    /// Project id (UUID v7).
    pub id: String,
    /// Project name, 1–200 characters.
    pub name: String,
    /// Creation time (RFC 3339 UTC).
    pub created_utc: String,
    /// Last modification time; `modifiedUtc ≥ createdUtc`.
    pub modified_utc: String,
    /// The last writer (03 §3.4.1).
    pub saved_by: SavedBy,
    /// Project-level settings.
    pub settings: ProjectSettings,
    /// Imported media (03 §3.4.2).
    pub media: Vec<MediaItem>,
    /// Bin tree (03 §3.4.3).
    pub bins: Vec<Bin>,
    /// Assets such as LUTs (03 §3.4.4).
    pub assets: Vec<Asset>,
    /// Sequences; at least one (03 §3.4.5).
    pub sequences: Vec<Sequence>,
    /// Non-undoable view state (03 §3.4.11).
    pub ui: ViewState,
    /// Unknown fields from newer versions, preserved on re-save (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// The `savedBy` block (03 §3.4.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SavedBy {
    /// Application name.
    pub app: String,
    /// Application version.
    pub version: String,
    /// Commit of the last writer.
    pub commit: String,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Project-level settings (03 §3.4.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectSettings {
    /// The default sequence (id).
    pub default_sequence_id: String,
    /// "Use proxies" toggle (REQ-MED-005).
    pub proxy_mode: bool,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Non-undoable view state saved with the project (03 §3.4.11).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct ViewState {
    /// The active sequence.
    pub active_sequence_id: Option<String>,
    /// Playhead per sequence, in flicks (`#[ts(type = "number")]`, 03 §3.2.6).
    #[ts(type = "Record<string, number>")]
    pub playheads: BTreeMap<String, i64>,
    /// Timeline zoom per sequence (pixels per second).
    #[ts(type = "Record<string, number>")]
    pub timeline_zoom: BTreeMap<String, f64>,
    /// Timeline scroll per sequence.
    pub timeline_scroll: BTreeMap<String, TimelineScroll>,
    /// Source viewer state.
    pub source_viewer: SourceViewer,
    /// Source/track targeting selection.
    #[ts(type = "Record<string, any>")]
    pub selected_track_targets: BTreeMap<String, serde_json::Value>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Timeline scroll position (03 §3.4.11).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct TimelineScroll {
    /// Horizontal position in flicks (`number`, 03 §3.2.6).
    #[ts(type = "number")]
    pub x: i64,
    /// Vertical position in pixels.
    pub y: u32,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Source viewer state (03 §3.4.11).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct SourceViewer {
    /// Media shown in the source viewer.
    pub media_id: Option<String>,
    /// Source in point (flicks).
    #[ts(type = "number")]
    pub r#in: Option<i64>,
    /// Source out point (flicks).
    #[ts(type = "number")]
    pub out: Option<i64>,
    /// Playhead (flicks).
    #[ts(type = "number")]
    pub playhead: i64,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a fresh project stamped with the caller's build identity
/// (03 §3.4.1 `savedBy`): one sequence, current timestamps, fresh ids (model
/// side of REQ-PRJ-001; the dialog is WP-05). Production callers (WP-03+)
/// inject the real application version and commit; tests use
/// [`new_project`] with a dev identity.
pub fn new_project_with(name: &str, sequence: Sequence, saved_by: SavedBy) -> Project {
    let now = iso_now();
    Project {
        format: "afterleads-video-spirits.project".into(),
        schema_version: 1,
        id: id::new_id(),
        name: name.into(),
        created_utc: now.clone(),
        modified_utc: now,
        saved_by,
        settings: ProjectSettings {
            default_sequence_id: sequence.id.clone(),
            proxy_mode: false,
            extra: BTreeMap::new(),
        },
        media: Vec::new(),
        bins: Vec::new(),
        assets: Vec::new(),
        sequences: vec![sequence],
        ui: ViewState::default(),
        extra: BTreeMap::new(),
    }
}

/// The development/test identity used by [`new_project`] (03 §3.4.1: real
/// `savedBy` comes from the writing application's build info).
pub fn dev_saved_by() -> SavedBy {
    SavedBy {
        app: "AfterLeads-Video-Spirits".into(),
        version: "dev".into(),
        commit: "dev".into(),
        extra: BTreeMap::new(),
    }
}

/// Create a fresh project with the development identity (see
/// [`new_project_with`]).
pub fn new_project(name: &str, sequence: Sequence) -> Project {
    new_project_with(name, sequence, dev_saved_by())
}

/// RFC 3339 UTC timestamp for `createdUtc`/`modifiedUtc` (03 §3.4.1). Uses the
/// system clock; tests inject their own values.
pub fn iso_now() -> String {
    // `time`/`chrono` are not in the baseline; format from UNIX seconds via
    // the standard library's civil-from-days algorithm (Howard Hinnant).
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        if m <= 2 { y + 1 } else { y },
        u32::try_from(m).unwrap_or(1),
        d,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1)); // 2024-01-01
        assert_eq!(civil_from_days(20_648), (2026, 7, 14)); // 2026-07-14
    }

    #[test]
    fn iso_now_is_rfc3339_utc_shaped() {
        let now = iso_now();
        // e.g. 2026-11-03T12:34:56Z
        assert_eq!(now.len(), 20);
        assert_eq!(&now[4..5], "-");
        assert_eq!(&now[10..11], "T");
        assert!(now.ends_with('Z'));
    }
}
