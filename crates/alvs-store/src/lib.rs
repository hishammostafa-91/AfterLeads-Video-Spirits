// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Storage and application services (WP-03, docs/02 §2.4 CMP-08).
//!
//! This crate owns every byte the application persists outside the project
//! model itself (docs/03 §3.6–3.9): application paths including portable
//! mode (§3.8), the settings store (§3.6), the SQLite cache database with
//! embedded transactional migrations (§3.7), recent projects, the atomic
//! write primitive (§3.9.1), the project save/open pipeline generic over a
//! [`project::DocumentHooks`] seam, the autosave scheduler (§3.9.3),
//! timestamped backups (§3.9.2), project lock files (§3.9.4), the session
//! marker and crash-recovery offer (§3.9.5), and the logging setup with a
//! size-rotating JSON-lines writer (docs/02 §2.11, docs/03 §3.9.8).
//!
//! Internal crate dependency: `alvs-core` only, per docs/02 §2.4 CMP-08
//! (enforced by `tools/check_deps.py`). The model-aware halves of the
//! save/open pipeline live behind [`project::DocumentHooks`]; the
//! production implementation for `alvs_model::Project` is composed in the
//! app backend (CMP-12, `app/src-tauri/src/project_io.rs`).

pub mod atomic;
pub mod autosave;
pub mod backup;
pub mod error;
pub mod keys;
pub mod lock;
pub mod paths;
pub mod project;
pub mod recent;
pub mod session;
pub mod utc;
