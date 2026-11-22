// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! AfterLeads-Video-Spirits application backend (CMP-12, docs/02
//! s2.3-s2.5): the GUI branch. `main.rs` owns the mode dispatch; this
//! module builds the Tauri app - single-instance enforcement first
//! (02 s2.5 step 4), the managed boot state, and the `app_*`
//! commands (04 s4.3.1).

pub mod commands;
pub mod dto;
pub mod locale;
pub mod project_io;
pub mod state;

use state::AppState;

/// The forwarded-open event (04 s4.10): payload `{ path }`.
pub const APP_OPEN_FILE_EVENT: &str = "app:open-file";

/// The GUI entry point (02 s2.5 startup order, the steps that exist).
pub mod gui {
    use super::*;
    use crate::state::Boot;
    use tauri::Manager;

    /// Run the GUI until the windows close (or `app_quit`).
    ///
    /// # Errors
    /// Tauri's run errors (window/WebView2 failures) propagate.
    pub fn run(boot: &mut Boot) -> Result<(), tauri::Error> {
        let state = AppState::new(boot.clone());
        tauri::Builder::default()
            // Single instance goes first (02 s2.5 step 4): a second
            // launch forwards its project path and exits.
            .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
                if let Some(path) = project_path_from_args(&args) {
                    let _ = tauri::Emitter::emit(
                        app,
                        APP_OPEN_FILE_EVENT,
                        serde_json::json!({ "path": path }),
                    );
                }
                // Focus the running instance's windows (the plugin's
                // contract: the second process never reaches run).
            }))
            .manage(state)
            .invoke_handler(tauri::generate_handler![
                commands::app_info,
                commands::app_ready,
                commands::app_quit
            ])
            .setup(|app| {
                // A launch-time project path takes the same road as
                // the single-instance forwarding: the UI owns opening
                // it (Phase 5) behind the same event.
                let boot_path = app.state::<AppState>().boot.path.clone();
                if let Some(path) = boot_path {
                    tauri::Emitter::emit(
                        app.app_handle(),
                        APP_OPEN_FILE_EVENT,
                        serde_json::json!({ "path": path }),
                    )?;
                }
                Ok(())
            })
            .run(tauri::generate_context!())
    }
}

/// The `.alvs` argument of a second launch (argv[0] is the exe
/// path); the first match wins, case-insensitive.
pub fn project_path_from_args(args: &[String]) -> Option<String> {
    args.iter()
        .skip(1)
        .find(|arg| arg.to_ascii_lowercase().ends_with(".alvs"))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::project_path_from_args;

    fn argv(rest: &[&str]) -> Vec<String> {
        std::iter::once("app.exe".to_owned())
            .chain(rest.iter().map(|s| (*s).to_owned()))
            .collect()
    }

    #[test]
    fn finds_the_first_alvs_argument() {
        let args = argv(&["--flag", "C:\\A.ALVS", "second.alvs"]);
        assert_eq!(project_path_from_args(&args), Some("C:\\A.ALVS".to_owned()));
    }

    #[test]
    fn ignores_non_project_arguments_and_the_exe() {
        assert_eq!(project_path_from_args(&argv(&["--worker"])), None);
        assert_eq!(project_path_from_args(&argv(&["cli.alvs.exe"])), None);
        assert_eq!(project_path_from_args(&[]), None);
    }
}
