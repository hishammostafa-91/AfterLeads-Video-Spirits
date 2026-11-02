// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#![deny(missing_docs)]
//! The project model: project, sequence, track, clip, effect, keyframe, caption,
//! and marker types, `EditOp` application, invariants and validation, undo/redo,
//! (de)serialization, and ts-rs exports (CMP-02, docs/02 §2.4).

/// Demo of a lint violation for the WP-01 failure demonstration (T023).
pub fn demo_clippy() -> u32 {
    let unused_demo = 7;
    42
}
