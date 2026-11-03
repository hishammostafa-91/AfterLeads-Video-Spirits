//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Assets such as imported LUTs (docs/03 §3.4.4).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// LUT dimensions (03 §3.4.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LutInfo {
    /// `3d` or `1d`.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub lut_type: String,
    /// Size (e.g. 33 for a 33×33×33 3D LUT).
    #[ts(type = "number")]
    pub size: u32,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// An imported asset copied into the project folder (03 §3.4.4; REQ-FX-008).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Asset {
    /// Asset id (UUID v7).
    pub id: String,
    /// Asset kind; `lut` in 1.0.
    pub kind: String,
    /// Display name.
    pub name: String,
    /// Path relative to the project folder.
    pub rel_path: String,
    /// SHA-256 of the asset file (validated on load; E2001 when missing).
    pub sha256: String,
    /// LUT metadata for `kind = lut`.
    pub lut_info: Option<LutInfo>,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}
