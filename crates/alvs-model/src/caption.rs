//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Caption tracks and items (docs/03 §3.4.10).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Caption box style (03 §3.4.10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionBox {
    /// Whether the box is drawn.
    pub enabled: bool,
    /// Box color (with alpha).
    pub color: [f64; 4],
    /// Padding in frame heights.
    pub padding: f64,
    /// Corner radius in frame heights.
    pub radius: f64,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Caption text style (03 §3.4.10); sizes in frame heights (03 §3.3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionStyle {
    /// Font family (FNT catalog).
    pub font: String,
    /// Font size in frame heights.
    pub size: f64,
    /// Weight (100–900).
    #[ts(type = "number")]
    pub weight: u32,
    /// Text color.
    pub color: [f64; 4],
    /// Outline color.
    pub outline_color: [f64; 4],
    /// Outline width in frame heights.
    pub outline_width: f64,
    /// Shadow color.
    pub shadow_color: [f64; 4],
    /// Shadow offset `[x, y]`.
    pub shadow_offset: [f64; 2],
    /// Shadow blur.
    pub shadow_blur: f64,
    /// Background box.
    pub box_: Option<CaptionBox>,
    /// Alignment.
    pub align: String,
    /// Vertical position (0–1, 0.9 default).
    pub vertical_position: f64,
    /// Maximum wrapped lines.
    #[ts(type = "number")]
    pub max_lines: u32,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A caption item (03 §3.4.10): sorted, non-overlapping, `start < end`,
/// text 1–1,000 characters (03 §3.5.11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionItem {
    /// Item id (UUID v7).
    pub id: String,
    /// Start in flicks.
    #[ts(type = "number")]
    pub start: i64,
    /// End in flicks.
    #[ts(type = "number")]
    pub end: i64,
    /// Caption text (may be RTL, e.g. Arabic).
    pub text: String,
    /// Optional position override, normalized.
    pub position: Option<[f64; 2]>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A caption track (03 §3.4.10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionTrack {
    /// Track id (UUID v7).
    pub id: String,
    /// Display name (e.g. "Arabic").
    pub name: String,
    /// BCP-47 language tag.
    pub language: String,
    /// Enabled flag.
    pub enabled: bool,
    /// Text style.
    pub style: CaptionStyle,
    /// Caption items.
    pub items: Vec<CaptionItem>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// The default caption style of 03 §3.4.10.
pub fn default_caption_style() -> CaptionStyle {
    CaptionStyle {
        font: "Cairo".into(),
        size: 0.045,
        weight: 600,
        color: [1.0, 1.0, 1.0, 1.0],
        outline_color: [0.0, 0.0, 0.0, 1.0],
        outline_width: 0.003,
        shadow_color: [0.0, 0.0, 0.0, 0.6],
        shadow_offset: [0.002, 0.002],
        shadow_blur: 0.003,
        box_: Some(CaptionBox {
            enabled: false,
            color: [0.0, 0.0, 0.0, 0.6],
            padding: 0.01,
            radius: 0.005,
            extra: BTreeMap::new(),
        }),
        align: "center".into(),
        vertical_position: 0.9,
        max_lines: 2,
        extra: BTreeMap::new(),
    }
}

/// Create a caption track with the default style (03 §3.4.10).
pub fn new_caption_track(name: &str, language: &str) -> CaptionTrack {
    CaptionTrack {
        id: id::new_id(),
        name: name.into(),
        language: language.into(),
        enabled: true,
        style: default_caption_style(),
        items: Vec::new(),
        extra: BTreeMap::new(),
    }
}
