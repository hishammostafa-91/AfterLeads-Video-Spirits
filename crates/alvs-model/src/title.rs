//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Title documents (docs/03 §3.4.9).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::effect::{AnimatablePrimitive, AnimatableValue};

/// Text alignment (03 §3.4.9), resolved against paragraph direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TextAlign {
    /// Start of the paragraph direction.
    Start,
    /// Centered.
    Center,
    /// End of the paragraph direction.
    End,
}

/// A title layer (03 §3.4.9). Sizes are in frame heights (03 §3.3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TitleLayer {
    /// Layer id (UUID v7).
    pub id: String,
    /// The text (may mix RTL and LTR, e.g. Arabic and Latin).
    pub text: String,
    /// Font family (FNT catalog, 12 §12.16).
    pub font: String,
    /// Font size in frame heights.
    pub size: AnimatableValue,
    /// Weight (100–900).
    #[ts(type = "number")]
    pub weight: u32,
    /// Italic.
    pub italic: bool,
    /// Fill color.
    pub fill: AnimatableValue,
    /// Optional stroke.
    pub stroke: Option<TitleStroke>,
    /// Optional shadow (shape preserved verbatim from 03 §3.4.9).
    pub shadow: Option<serde_json::Value>,
    /// Optional background box.
    pub box_: Option<TitleBox>,
    /// Alignment.
    pub align: TextAlign,
    /// Line height multiplier.
    pub line_height: f64,
    /// Letter spacing.
    pub letter_spacing: f64,
    /// Position, normalized.
    pub position: AnimatableValue,
    /// Rotation, degrees clockwise.
    pub rotation: AnimatableValue,
    /// Opacity percent.
    pub opacity: AnimatableValue,
}

/// Title stroke (03 §3.4.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TitleStroke {
    /// Stroke color.
    pub color: AnimatableValue,
    /// Stroke width in frame heights.
    pub width: AnimatableValue,
}

/// Title background box (03 §3.4.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TitleBox {
    /// Box color (with alpha).
    pub color: AnimatableValue,
    /// Padding in frame heights.
    pub padding: f64,
    /// Corner radius in frame heights.
    pub radius: f64,
}

/// A title animation (03 §3.4.9): `tan.*` catalog id, duration, easing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TitleAnimation {
    /// Animation id (TAN catalog, 12 §12.12).
    pub animation_id: String,
    /// Duration in flicks.
    #[ts(type = "number")]
    pub duration: i64,
    /// Easing id (EAS catalog).
    pub easing: String,
}

/// A title document (03 §3.4.9).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct TitleDocument {
    /// Layers, bottom to top.
    pub layers: Vec<TitleLayer>,
    /// In-animation.
    pub anim_in: Option<TitleAnimation>,
    /// Out-animation.
    pub anim_out: Option<TitleAnimation>,
    /// Originating template (TPL catalog), when created from one.
    pub template_id: Option<String>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a starter title layer with the defaults of 03 §3.4.9.
pub fn new_title_layer(text: &str, font: &str) -> TitleLayer {
    let num = |v: f64| AnimatableValue::V {
        v: AnimatablePrimitive::Number(v),
    };
    let point = |x: f64, y: f64| AnimatableValue::V {
        v: AnimatablePrimitive::Point([x, y]),
    };
    TitleLayer {
        id: id::new_id(),
        text: text.into(),
        font: font.into(),
        size: num(0.08),
        weight: 700,
        italic: false,
        fill: AnimatableValue::V {
            v: AnimatablePrimitive::Color([1.0, 1.0, 1.0, 1.0]),
        },
        stroke: None,
        shadow: None,
        box_: None,
        align: TextAlign::Center,
        line_height: 1.2,
        letter_spacing: 0.0,
        position: point(0.5, 0.8),
        rotation: num(0.0),
        opacity: num(100.0),
    }
}
