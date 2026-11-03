//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Masks (docs/03 §3.4.8).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::effect::AnimatableValue;

/// Mask usage (03 §3.4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MaskUsage {
    /// Masks the whole clip.
    Clip,
    /// Only referenced through an effect's `maskIds` (REQ-FX-006).
    Effect,
}

/// A mask (03 §3.4.8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Mask {
    /// Mask id (UUID v7).
    pub id: String,
    /// Mask shape id (MSK catalog, 12 §12.6).
    pub mask_id: String,
    /// Shape parameters.
    pub params: BTreeMap<String, AnimatableValue>,
    /// Edge feather (normalized).
    pub feather: AnimatableValue,
    /// Expansion.
    pub expansion: AnimatableValue,
    /// Opacity percent.
    pub opacity: AnimatableValue,
    /// Invert the mask.
    pub invert: bool,
    /// Combine mode (`add`, `subtract`, … MSK catalog semantics).
    pub mode: String,
    /// Whether the mask applies to the clip or only via `maskIds`.
    pub usage: MaskUsage,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a mask with default fields (03 §3.4.8).
pub fn new_mask(mask_id: &str) -> Mask {
    Mask {
        id: id::new_id(),
        mask_id: mask_id.into(),
        params: BTreeMap::new(),
        feather: AnimatableValue::V {
            v: crate::effect::AnimatablePrimitive::Number(0.0),
        },
        expansion: AnimatableValue::V {
            v: crate::effect::AnimatablePrimitive::Number(0.0),
        },
        opacity: AnimatableValue::V {
            v: crate::effect::AnimatablePrimitive::Number(100.0),
        },
        invert: false,
        mode: "add".into(),
        usage: MaskUsage::Clip,
        extra: BTreeMap::new(),
    }
}
