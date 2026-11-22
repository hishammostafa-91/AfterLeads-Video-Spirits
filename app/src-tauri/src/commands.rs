// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The application commands (04 s4.3.1): `app_info`, `app_ready`,
//! `app_quit`. Every command returns `Result<_, ErrorEnvelope>`
//! (04 s4.2 rule 3); validation happens on the pure plan/state types
//! below so the Tauri layer stays thin.

use serde::{Deserialize, Serialize};

use crate::dto::{AppInfo, RendererInfo};
use crate::locale;
use crate::state::AppState;
use alvs_core::error::{AppError, ErrorEnvelope};

/// `app_info` - the boot report (04 s4.3.1). Fields the engine work
/// packages own carry their honest unknown marker until they report
/// in (renderer: WP-09).
#[tauri::command]
pub fn app_info(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppInfo, ErrorEnvelope> {
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        commit: env!("ALVS_COMMIT").to_owned(),
        build_date: env!("ALVS_BUILD_DATE").to_owned(),
        identifier: app.config().identifier.clone(),
        renderer: RendererInfo {
            name: "unknown".to_owned(),
            is_warp: false,
        },
        safe_mode: state.boot.safe_mode,
        portable: false,
        locale: locale::display_locale().to_owned(),
    })
}

/// `app_ready` - the UI signals it mounted; the core emits current
/// state events (04 s4.3.1). No state events exist before their
/// features ship (notifications: WP-05 phase 6), so WP-05 records
/// readiness only.
#[tauri::command]
pub fn app_ready(state: tauri::State<AppState>) -> Result<(), ErrorEnvelope> {
    state.mark_ready();
    Ok(())
}

/// `app_quit` request (04 s4.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuitRequest {
    /// True when the UI already resolved the dirty gate (DLG-20) and
    /// running jobs are cancelled without further confirmation.
    pub force: bool,
}

/// What `app_quit` does with background jobs (02 s2.5 step 3). Only
/// the cancel policy can exist before jobs do (WP-07 adds the
/// "finish in background" option).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobPolicy {
    Cancel,
}

/// The shutdown steps that apply at this work package's state,
/// checked off from 02 s2.5:
///
/// 1. dirty prompt - the UI's gate (DLG-20) before this command;
/// 2. stop playback - no playback engine yet (WP-11);
/// 3. running jobs - [`Self::job_policy`];
/// 4. flush settings/autosave, release the project lock - no loaded
///    settings store or open project yet (WP-24 / WP-05 phase 5);
/// 5. stop workers, destroy GPU, close the database, remove the
///    session marker - not yet owned (WP-07 / WP-09 / WP-03
///    integration).
/// 6. (the remaining sequence collapses into 5 above for now.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownPlan {
    /// What happens to background jobs (02 s2.5 step 3).
    pub job_policy: JobPolicy,
}

impl ShutdownPlan {
    /// The plan for a quit request.
    pub fn for_request(_request: &QuitRequest) -> Self {
        // Both force paths cancel jobs in WP-05: there is nothing to
        // keep running yet and no confirmation dialog to show.
        Self {
            job_policy: JobPolicy::Cancel,
        }
    }
}

/// `app_quit` - run the shutdown sequence (02 s2.5) and exit.
#[tauri::command]
pub fn app_quit(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    request: QuitRequest,
) -> Result<(), ErrorEnvelope> {
    // Validation (04 s4.2 rule 3): the request must be well-formed,
    // which deserialization already proved; the plan is deterministic.
    let plan = ShutdownPlan::for_request(&request);
    let _ = plan;
    state.mark_quitting();
    // cleanup_before_exit runs on exit (the single-instance lock
    // releases with it, per the plugin's exit contract).
    app.exit(0);
    Ok(())
}

/// Map a validation failure to the envelope (kept for the request
/// types of the next phases; E1003 is the catalog's
/// failed-validation code in 12 s12.13).
pub fn envelope_for_invalid(detail: String) -> ErrorEnvelope {
    ErrorEnvelope::from_error(&AppError::ValidationFailed { detail })
}

#[cfg(test)]
mod tests {
    use super::{JobPolicy, QuitRequest, ShutdownPlan};

    #[test]
    fn both_quit_paths_cancel_jobs_for_now() {
        for force in [false, true] {
            assert_eq!(
                ShutdownPlan::for_request(&QuitRequest { force }).job_policy,
                JobPolicy::Cancel
            );
        }
    }
}
