//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Tracks and clips (docs/03 §3.4.6), including intrinsic transform/crop and
//! the audio clip object.

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::effect::AnimatableValue;
use crate::effect::EffectInstance;
use crate::marker::Marker;
use crate::mask::Mask;
use crate::title::TitleDocument;
use crate::transition::Transition;

/// Track kind (03 §3.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TrackKind {
    /// Video track.
    Video,
    /// Audio track.
    Audio,
}

/// Clip type (03 §3.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ClipType {
    /// A media clip.
    Media,
    /// A title clip.
    Title,
    /// A generator clip.
    Generator,
    /// A nested sequence clip.
    Sequence,
}

/// Deinterlace mode (03 §3.4.6; REQ-MED-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Deinterlace {
    /// Detect automatically.
    Auto,
    /// Never deinterlace.
    Off,
    /// Always deinterlace.
    Force,
}

/// Intrinsic transform (03 §3.4.6): every clip's own geometry, before
/// effects. Positions/sizes in normalized space (03 §3.3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Transform {
    /// Center position, normalized (0.5, 0.5 default).
    pub position: AnimatableValue,
    /// Uniform scale percent (100 default).
    pub scale: AnimatableValue,
    /// Horizontal scale percent.
    pub scale_x: AnimatableValue,
    /// Vertical scale percent.
    pub scale_y: AnimatableValue,
    /// Lock uniform scaling.
    pub uniform_scale: bool,
    /// Rotation in degrees, clockwise (03 §3.3.3).
    pub rotation: AnimatableValue,
    /// Anchor point, normalized.
    pub anchor: AnimatableValue,
    /// Horizontal flip.
    pub flip_h: bool,
    /// Vertical flip.
    pub flip_v: bool,
}

/// Intrinsic crop (03 §3.4.6), normalized to frame heights (03 §3.3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Crop {
    /// Left inset.
    pub left: AnimatableValue,
    /// Top inset.
    pub top: AnimatableValue,
    /// Right inset.
    pub right: AnimatableValue,
    /// Bottom inset.
    pub bottom: AnimatableValue,
    /// Edge feather.
    pub feather: AnimatableValue,
}

/// The intrinsic block of a video clip (03 §3.4.6; 12 §12.1.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Intrinsic {
    /// Geometry.
    pub transform: Transform,
    /// Crop.
    pub crop: Crop,
    /// Opacity percent.
    pub opacity: AnimatableValue,
    /// Blend mode id (BLM catalog, 12 §12.4).
    pub blend_mode: String,
}

/// Audio clip settings (03 §3.4.6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AudioClipSettings {
    /// Gain in dB.
    pub gain_db: f64,
    /// Volume in dB (animatable; −200 = −∞, 03 §3.3.4).
    pub volume: AnimatableValue,
    /// Pan −1..+1 (animatable).
    pub pan: AnimatableValue,
    /// Fade-in duration in flicks.
    #[ts(type = "number")]
    pub fade_in: i64,
    /// Fade-out duration in flicks.
    #[ts(type = "number")]
    pub fade_out: i64,
    /// Fade shape (03 §3.4.6).
    pub fade_curve: FadeCurve,
    /// Channel mode (REQ-AUD-008).
    pub channel_mode: ChannelMode,
    /// Preserve pitch under speed changes.
    pub preserve_pitch: bool,
}

/// Fade shape (03 §3.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FadeCurve {
    /// Linear.
    Linear,
    /// Constant power (equal-power) crossfade.
    ConstantPower,
}

/// Channel mode (03 §3.4.6; REQ-AUD-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ChannelMode {
    /// Normal stereo.
    Stereo,
    /// Left channel to both.
    LeftToBoth,
    /// Right channel to both.
    RightToBoth,
    /// Mono mixdown.
    MonoMix,
    /// Swapped channels.
    Swap,
}

/// Generator parameters (03 §3.4.6): `generatorId` plus its params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GeneratorParams {
    /// Generator id (GEN catalog, 12 §12.12).
    pub generator_id: String,
    /// Generator parameters (manifest-typed, 03 §3.4.7 value types).
    pub params: BTreeMap<String, AnimatableValue>,
}

/// A clip on a track (03 §3.4.6). The per-type payload fields are `Option`:
/// `mediaId`/`streamIndex` for `media`, `title` for `title`, `generator` for
/// `generator`, `sequenceId` for `sequence`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Clip {
    /// Clip id (UUID v7).
    pub id: String,
    /// Clip type.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub clip_type: ClipType,
    /// Source media for `media` clips.
    pub media_id: Option<String>,
    /// Stream used by this clip (a linked A/V pair shares `linkGroupId`).
    pub stream_index: Option<u32>,
    /// Timeline start in flicks.
    #[ts(type = "number")]
    pub start: i64,
    /// Timeline duration in flicks.
    #[ts(type = "number")]
    pub duration: i64,
    /// Source in-point in flicks (0 for titles and generators).
    #[ts(type = "number")]
    pub source_in: i64,
    /// Speed as a reduced rational (1/10 ≤ ≤ 16, 03 §3.5.6).
    pub speed: Rational,
    /// Reverse playback.
    pub reverse: bool,
    /// Freeze at this source position for the whole duration (flicks).
    #[ts(type = "number")]
    pub freeze_at: Option<i64>,
    /// Disabled clips are skipped in render and mix.
    pub enabled: bool,
    /// Display-name override.
    pub name: Option<String>,
    /// Label color.
    pub label: Option<String>,
    /// Link group (A/V of the same source), shared by paired clips.
    pub link_group_id: Option<String>,
    /// Arbitrary grouping (REQ-EDT-007).
    pub group_id: Option<String>,
    /// Intrinsic block (video clips only).
    pub intrinsic: Option<Intrinsic>,
    /// Audio settings (audio clips only).
    pub audio: Option<AudioClipSettings>,
    /// Deinterlace handling (REQ-MED-010).
    pub deinterlace: Deinterlace,
    /// Effect instances.
    pub effects: Vec<EffectInstance>,
    /// Masks.
    pub masks: Vec<Mask>,
    /// Source-time markers.
    pub markers: Vec<Marker>,
    /// Title document for `title` clips (03 §3.4.9).
    pub title: Option<TitleDocument>,
    /// Generator payload for `generator` clips.
    pub generator: Option<GeneratorParams>,
    /// Nested sequence for `sequence` clips.
    pub sequence_id: Option<String>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

use alvs_core::rational::Rational;

/// A track (03 §3.4.6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Track {
    /// Track id (UUID v7).
    pub id: String,
    /// Track kind.
    pub kind: TrackKind,
    /// Display name (V1, A1, …).
    pub name: String,
    /// Height in pixels (32–240).
    #[ts(type = "number")]
    pub height: u32,
    /// Locked against edits (04 §4.4 `insertClips` respects it).
    pub locked: bool,
    /// Visible (video).
    pub visible: bool,
    /// Muted (audio).
    pub muted: bool,
    /// Solo (audio).
    pub solo: bool,
    /// Track volume in dB (−200..+12).
    pub volume_db: f64,
    /// Pan −1..+1.
    pub pan: f64,
    /// Source-monitor targeting.
    pub targeted: bool,
    /// Track effects (audio tracks only in 1.0).
    pub effects: Vec<EffectInstance>,
    /// Clips sorted by `start`, never overlapping (03 §3.5.4).
    pub items: Vec<Clip>,
    /// Transitions between adjacent clips (03 §3.4.10).
    pub transitions: Vec<Transition>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a new track with the given kind and default fields (03 §3.4.6).
pub fn new_track(kind: TrackKind, name: &str) -> Track {
    Track {
        id: id::new_id(),
        kind,
        name: name.into(),
        height: 64,
        locked: false,
        visible: true,
        muted: false,
        solo: false,
        volume_db: 0.0,
        pan: 0.0,
        targeted: true,
        effects: Vec::new(),
        items: Vec::new(),
        transitions: Vec::new(),
        extra: BTreeMap::new(),
    }
}
