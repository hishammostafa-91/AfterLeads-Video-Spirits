//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Effect instances and animatable values (docs/03 §3.4.7).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An animatable value: static `{ "v": … }` or keyframed `{ "k": […] }`
/// (03 §3.4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum AnimatableValue {
    /// A static value.
    V {
        /// The value.
        v: AnimatablePrimitive,
    },
    /// Keyframes over the clip.
    K {
        /// Keyframes sorted by `t`.
        k: Vec<Keyframe>,
    },
}

/// Value types carried by parameters and keyframes (03 §3.4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum AnimatablePrimitive {
    /// A number (parameters are f64 per 03 §3.2.5).
    Number(f64),
    /// `[x, y]` in normalized space (03 §3.3.3).
    Point([f64; 2]),
    /// `[r, g, b, a]` in 0–1.
    Color([f64; 4]),
    /// An enum/catalog string, including LUT references `"asset:<assetId>"`
    /// (03 §3.4.7 value types: strings are one type; the prefix is
    /// convention, validated against the manifest by rule 9).
    Str(String),
    /// A boolean (hold-eased when keyframed, 03 §3.5.9).
    Bool(bool),
    /// A curve `[[x, y], …]`.
    Curve(Vec<[f64; 2]>),
    /// A wheel offset `{x, y, luma}` (color tools).
    Wheel {
        /// Red/cyan offset.
        x: f64,
        /// Blue/yellow offset.
        y: f64,
        /// Luma offset.
        luma: f64,
    },
}

/// A keyframe (03 §3.4.7): `t` flicks relative to the clip start; `e` easing
/// id applies from this keyframe to the next; `h` only when `e = "bezier"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Keyframe {
    /// Time in flicks (`number`, 03 §3.2.6).
    #[ts(type = "number")]
    pub t: i64,
    /// The value at `t`.
    pub v: AnimatablePrimitive,
    /// Easing id (EAS catalog, 12 §12.5).
    pub e: String,
    /// Bezier handles `[x1, y1, x2, y2]`, only for `e = "bezier"`.
    pub h: Option<[f64; 4]>,
}

/// An effect instance (03 §3.4.7). Missing parameters take manifest
/// defaults; out-of-range values are clamped on load with E1003 detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EffectInstance {
    /// Instance id (UUID v7).
    pub id: String,
    /// Effect id (VFX/AFX catalog, 12 §12.1–12.2).
    pub effect_id: String,
    /// Manifest version the parameters were authored against.
    #[ts(type = "number")]
    pub version: u32,
    /// Bypass flag.
    pub enabled: bool,
    /// Parameters, keyed by manifest name.
    pub params: BTreeMap<String, AnimatableValue>,
    /// Restricts the effect to these masks of the same clip (REQ-FX-006).
    pub mask_ids: Vec<String>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create an effect instance with manifest-default (empty) parameters; the
/// manifest defaults themselves arrive with the effects framework (WP-14).
pub fn new_effect_instance(effect_id: &str) -> EffectInstance {
    EffectInstance {
        id: id::new_id(),
        effect_id: effect_id.into(),
        version: 1,
        enabled: true,
        params: BTreeMap::new(),
        mask_ids: Vec::new(),
        extra: BTreeMap::new(),
    }
}
