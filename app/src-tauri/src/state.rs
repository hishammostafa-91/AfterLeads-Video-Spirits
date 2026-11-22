// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The managed app state (02 s2.5 step 1 inputs plus the lifecycle
//! marks `app_ready` / `app_quit` set).

use std::sync::Mutex;

/// The launch inputs the GUI branch received (02 s2.5 step 1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Boot {
    /// Safe mode was requested (flag or Shift held at launch).
    pub safe_mode: bool,
    /// A project path passed as the launch argument; the shell opens
    /// it (Phase 5) - emitted as `app:open-file` for the UI either
    /// way, exactly like the single-instance forwarding.
    pub path: Option<String>,
}

impl Boot {
    /// Boot inputs from the parsed flags.
    pub fn from_flags(safe_mode: bool, path: Option<String>) -> Self {
        Self { safe_mode, path }
    }
}

/// Lifecycle marks for tests and later status reporting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Lifecycle {
    /// `app_ready` arrived.
    pub ready: bool,
    /// `app_quit` ran.
    pub quitting: bool,
}

/// The state managed by the Tauri app.
pub struct AppState {
    /// Launch inputs (02 s2.5 step 1).
    pub boot: Boot,
    lifecycle: Mutex<Lifecycle>,
}

impl AppState {
    /// State over the parsed boot inputs.
    pub fn new(boot: Boot) -> Self {
        Self {
            boot,
            lifecycle: Mutex::new(Lifecycle::default()),
        }
    }

    /// Record `app_ready`.
    pub fn mark_ready(&self) {
        if let Ok(mut lifecycle) = self.lifecycle.lock() {
            lifecycle.ready = true;
        }
    }

    /// Record `app_quit`.
    pub fn mark_quitting(&self) {
        if let Ok(mut lifecycle) = self.lifecycle.lock() {
            lifecycle.quitting = true;
        }
    }

    /// The current lifecycle marks.
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle.lock().map(|l| *l).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, Boot, Lifecycle};

    #[test]
    fn records_lifecycle_marks() {
        let state = AppState::new(Boot::from_flags(true, Some("a.alvs".into())));
        assert_eq!(state.lifecycle(), Lifecycle::default());
        state.mark_ready();
        assert!(state.lifecycle().ready);
        assert!(!state.lifecycle().quitting);
        state.mark_quitting();
        assert!(state.lifecycle().ready);
        assert!(state.lifecycle().quitting);
    }
}
