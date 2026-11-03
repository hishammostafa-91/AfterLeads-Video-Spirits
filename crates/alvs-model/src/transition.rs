//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Transitions (docs/03 §3.4.10).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::effect::AnimatableValue;

/// Transition alignment over the edit point (03 §3.4.10): `center` covers
/// `[edit − d/2, edit + d/2]`, `start` `[edit, edit + d]`, `end`
/// `[edit − d, edit]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TransitionAlignment {
    /// Centered on the edit.
    Center,
    /// Starts at the edit.
    Start,
    /// Ends at the edit.
    End,
}

/// A transition between two adjacent clips on a track (03 §3.4.10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Transition {
    /// Transition id (UUID v7).
    pub id: String,
    /// Transition id (TRS catalog, 12 §12.3).
    pub transition_id: String,
    /// The outgoing clip.
    pub from_clip_id: String,
    /// The incoming clip.
    pub to_clip_id: String,
    /// Duration in flicks (0 < d ≤ available handles, 03 §3.5.8).
    #[ts(type = "number")]
    pub duration: i64,
    /// Alignment over the edit point.
    pub alignment: TransitionAlignment,
    /// Transition parameters.
    pub params: BTreeMap<String, AnimatableValue>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a transition (03 §3.4.10).
pub fn new_transition(
    transition_id: &str,
    from_clip_id: &str,
    to_clip_id: &str,
    duration: i64,
) -> Transition {
    Transition {
        id: id::new_id(),
        transition_id: transition_id.into(),
        from_clip_id: from_clip_id.into(),
        to_clip_id: to_clip_id.into(),
        duration,
        alignment: TransitionAlignment::Center,
        params: BTreeMap::new(),
        extra: BTreeMap::new(),
    }
}
