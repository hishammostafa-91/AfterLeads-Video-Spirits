// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#![deny(missing_docs)]
//! The project model: project, sequence, track, clip, effect, keyframe, caption,
//! and marker types, `EditOp` application, invariants and validation, undo/redo,
//! (de)serialization, and ts-rs exports (CMP-02, docs/02 §2.4).

pub mod asset;
pub mod bin;
pub mod caption;
pub mod effect;
pub mod marker;
pub mod mask;
pub mod media;
pub mod project;
pub mod schema;
pub mod sequence;
pub mod title;
pub mod track;
pub mod transition;
pub mod validate;
