//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Media items (docs/03 §3.4.2).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Media kind (03 §3.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MediaKind {
    /// A video file.
    Video,
    /// An audio file.
    Audio,
    /// A still image.
    Image,
    /// An image sequence.
    ImageSequence,
}

/// Relink fingerprint: size, mtime, SHA-256 of the first and last MiB
/// (03 §3.4.2; REQ-PRJ-010; `mediaKey` cache key source).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Fingerprint {
    /// File size in bytes.
    #[ts(type = "number")]
    pub size: u64,
    /// Modification time (RFC 3339 UTC).
    pub mtime_utc: String,
    /// SHA-256 of the first and last MiB.
    pub head_tail_sha256: String,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A probed stream snapshot (03 §3.4.2); codec/color names use FFmpeg's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProbeStream {
    /// Stream index in the container.
    pub index: u32,
    /// `video` or `audio`.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub stream_type: String,
    /// Codec name (FFmpeg).
    pub codec: String,
    /// Width for video streams.
    pub width: Option<u32>,
    /// Height for video streams.
    pub height: Option<u32>,
    /// Frame rate for video streams.
    pub frame_rate: Option<alvs_core::rational::Rational>,
    /// Variable frame rate flag.
    pub vfr: Option<bool>,
    /// Bit depth for video streams.
    pub bit_depth: Option<u32>,
    /// Chroma subsampling for video streams.
    pub chroma: Option<String>,
    /// Color primaries (FFmpeg name).
    pub primaries: Option<String>,
    /// Transfer characteristics (FFmpeg name).
    pub transfer: Option<String>,
    /// Color matrix (FFmpeg name).
    pub matrix: Option<String>,
    /// `limited` or `full`.
    pub range: Option<String>,
    /// HDR packaging (`hlg`, …) when present.
    pub hdr: Option<String>,
    /// Rotation in degrees.
    pub rotation: Option<i32>,
    /// Interlaced flag.
    pub interlaced: Option<bool>,
    /// Channel count for audio streams.
    pub channels: Option<u32>,
    /// Channel layout for audio streams.
    pub layout: Option<String>,
    /// Sample rate for audio streams.
    pub sample_rate: Option<u32>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Probe snapshot needed to edit offline (03 §3.4.2); the cache database
/// holds the full probe (WP-03/WP-08).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Probe {
    /// Duration in flicks (`number`, 03 §3.2.6).
    #[ts(type = "number")]
    pub duration_flicks: i64,
    /// Container format name (FFmpeg).
    pub container: String,
    /// Stream snapshots.
    pub streams: Vec<ProbeStream>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Image sequence description (03 §3.4.2, `kind = imageSequence`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImageSequence {
    /// File name pattern, e.g. `shot_%04d.png`.
    pub pattern: String,
    /// First frame number.
    pub first: u32,
    /// Frame count.
    #[ts(type = "number")]
    pub count: u32,
    /// Frame rate.
    pub frame_rate: alvs_core::rational::Rational,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// An optimized derived file (REQ-MED-011, 03 §3.4.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Optimized {
    /// Path relative to the project folder.
    pub rel_path: String,
    /// Optimization preset id (XP catalog).
    pub preset: String,
    /// Whether the derived file is in use.
    pub active: bool,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A stabilized derived file (REQ-MED-013, 03 §3.4.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Stabilized {
    /// Path relative to the project folder.
    pub rel_path: String,
    /// Stabilization strength 0–1.
    pub strength: f64,
    /// Crop mode (`auto`, …).
    pub crop: String,
    /// Whether the derived file is in use.
    pub active: bool,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Derived media of an item (03 §3.4.2).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Derived {
    /// Optimized file when present.
    pub optimized: Option<Optimized>,
    /// Stabilized file when present.
    pub stabilized: Option<Stabilized>,
}

/// A media item (03 §3.4.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MediaItem {
    /// Item id (UUID v7).
    pub id: String,
    /// Kind of media.
    pub kind: MediaKind,
    /// Display name (usually the file name).
    pub name: String,
    /// Absolute path.
    pub path: String,
    /// Path relative to the project folder (tried when `path` misses).
    pub rel_path: String,
    /// Relink fingerprint.
    pub fingerprint: Fingerprint,
    /// Probe snapshot.
    pub probe: Probe,
    /// Image sequence data for `kind = imageSequence`.
    pub image_sequence: Option<ImageSequence>,
    /// Owning bin, if any.
    pub bin_id: Option<String>,
    /// Label color (`CLR-01`..`CLR-08`) or none.
    pub label: Option<String>,
    /// Free-form notes.
    pub notes: String,
    /// Derived media.
    pub derived: Derived,
    /// Import time (RFC 3339 UTC).
    pub added_utc: String,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}
