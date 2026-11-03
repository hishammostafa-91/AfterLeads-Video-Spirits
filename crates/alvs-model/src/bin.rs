//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Bins (docs/03 §3.4.3): a tree, no cycles.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A media bin (03 §3.4.3); `parentId` forms a tree without cycles and a
/// media item belongs to at most one bin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Bin {
    /// Bin id (UUID v7).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Parent bin, or none at the root.
    pub parent_id: Option<String>,
    /// Label color (`CLR-xx`) or none.
    pub label: Option<String>,
    /// UI expansion state.
    pub expanded: bool,
    /// Unknown fields (03 §3.2.3).
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}
