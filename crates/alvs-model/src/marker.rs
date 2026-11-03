//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Markers (docs/03 §3.4.10).

use std::collections::BTreeMap;

use alvs_core::id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Marker kind (03 §3.4.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MarkerKind {
    /// A standard marker.
    Standard,
    /// A chapter marker (unique `t` per scope, 03 §3.5.12).
    Chapter,
}

/// A marker (03 §3.4.10): sequence time when owned by a sequence, source
/// time when owned by a clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Marker {
    /// Marker id (UUID v7).
    pub id: String,
    /// Position in flicks (≥ 0).
    #[ts(type = "number")]
    pub t: i64,
    /// Duration in flicks (≥ 0).
    #[ts(type = "number")]
    pub duration: i64,
    /// Display name.
    pub name: String,
    /// Color (CLR catalog, 12 §12.17).
    pub color: String,
    /// Free-form note.
    pub note: String,
    /// Marker kind.
    pub kind: MarkerKind,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Create a standard marker (03 §3.4.10).
pub fn new_marker(t: i64, name: &str) -> Marker {
    Marker {
        id: id::new_id(),
        t,
        duration: 0,
        name: name.into(),
        color: "CLR-03".into(),
        note: String::new(),
        kind: MarkerKind::Standard,
        extra: BTreeMap::new(),
    }
}
