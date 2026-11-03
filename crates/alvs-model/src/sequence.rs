//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Sequences (docs/03 §3.4.5).

use std::collections::BTreeMap;

use alvs_core::id;
use alvs_core::rational::Rational;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::caption::CaptionTrack;
use crate::effect::EffectInstance;
use crate::marker::Marker;
use crate::track::Track;

/// Sequence format (03 §3.4.5). All sequences are SDR Rec.709 with square
/// pixels and a stereo master in 1.0 (REQ-PRJ-001).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SequenceFormat {
    /// Width in pixels (even, 16–8192).
    pub width: u32,
    /// Height in pixels (even, 16–8192).
    pub height: u32,
    /// Frame rate (12 §12.9).
    pub frame_rate: Rational,
    /// Audio sample rate: 44,100 or 48,000.
    #[ts(type = "number")]
    pub sample_rate: u32,
    /// Audio layout; `stereo` in 1.0.
    pub audio_layout: String,
    /// Always `rec709-sdr` in 1.0 (HDR arrives in 1.1).
    pub color_space: String,
    /// Drop-frame timecode flag (FPS-04/FPS-07 default to drop).
    pub drop_frame_timecode: bool,
    /// True for SQP-01 sequences until the first video clip sets the format.
    pub auto_match_pending: bool,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Master audio bus of a sequence (03 §3.4.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MasterAudio {
    /// Master volume in dB (−200..+12; −200 displays as −∞, 03 §3.3.4).
    pub volume_db: f64,
    /// Master-bus effect instances (added via `addEffect` owner `master`).
    pub effects: Vec<EffectInstance>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A sequence (03 §3.4.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Sequence {
    /// Sequence id (UUID v7).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Format (SDR Rec.709, square pixels, 03 §3.4.5).
    pub format: SequenceFormat,
    /// Tracks in display order, top to bottom within each kind; video
    /// compositing order is bottom-most video track first.
    pub tracks: Vec<Track>,
    /// Caption tracks.
    pub caption_tracks: Vec<CaptionTrack>,
    /// Sequence markers (sequence time).
    pub markers: Vec<Marker>,
    /// Sequence in point (flicks) or none.
    #[ts(type = "number")]
    pub in_point: Option<i64>,
    /// Sequence out point (flicks) or none.
    #[ts(type = "number")]
    pub out_point: Option<i64>,
    /// Master audio bus.
    pub master_audio: MasterAudio,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// The SQP-02 preset format (HD 1080p at 30 fps) — the documented fallback of
/// SQP-01 "Auto — match first clip" (12 §12.8; REQ-PRJ-001) and the shape
/// used by every named preset's frame/audio defaults.
pub fn sqp02_fallback_format() -> SequenceFormat {
    SequenceFormat {
        width: 1920,
        height: 1080,
        frame_rate: Rational::new(30, 1).unwrap_or(Rational { num: 30, den: 1 }),
        sample_rate: 48_000,
        audio_layout: "stereo".into(),
        color_space: "rec709-sdr".into(),
        drop_frame_timecode: false,
        auto_match_pending: true,
        extra: BTreeMap::new(),
    }
}

/// Create a sequence: SQP-01 keeps `autoMatchPending` until the first clip
/// sets the format; named presets resolve to a concrete format and clear it
/// (03 §3.4.5; REQ-PRJ-005).
pub fn new_sequence(name: &str, format: SequenceFormat) -> Sequence {
    Sequence {
        id: id::new_id(),
        name: name.into(),
        format,
        tracks: Vec::new(),
        caption_tracks: Vec::new(),
        markers: Vec::new(),
        in_point: None,
        out_point: None,
        master_audio: MasterAudio {
            volume_db: 0.0,
            effects: Vec::new(),
            extra: BTreeMap::new(),
        },
        extra: BTreeMap::new(),
    }
}
