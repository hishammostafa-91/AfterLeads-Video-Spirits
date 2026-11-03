// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Application path resolution (docs/03 §3.8): the identifier-based folder
//! layout from `APPDATA`/`LOCALAPPDATA`/`TEMP` env roots, portable mode
//! (`portable.flag` beside the executable → `<exe folder>\data\…`), and the
//! SEC-04 containment roots every store write must stay inside.

use std::path::{Path, PathBuf};

use crate::error::StoreError;

/// Tauri identifier whose folder names 03 §3.8 prescribes (verified against
/// `app/src-tauri/tauri.conf.json`).
pub const IDENTIFIER: &str = "com.afterleads.videospirits";

/// Folder name under `%TEMP%` (03 §3.8): `AfterLeads-Video-Spirits\<pid>\`.
pub const TEMP_FOLDER: &str = "AfterLeads-Video-Spirits";

/// The user-library subfolders of 03 §3.8.
pub const LIBRARY_SUBFOLDERS: [&str; 4] = [
    "effect-presets",
    "export-presets",
    "title-templates",
    "keymaps",
];

/// Every application location of 03 §3.8, resolved once at startup
/// (docs/02 §2.5 step 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    roaming_root: PathBuf,
    local_root: PathBuf,
    temp_root: PathBuf,
    default_project_folder: PathBuf,
    portable: bool,
}

impl AppPaths {
    /// Resolve from the process environment, honoring portable mode when
    /// `portable.flag` sits beside the current executable (03 §3.8).
    pub fn resolve() -> Result<Self, StoreError> {
        let missing = |name: &str| StoreError::SaveFailed {
            detail: format!("environment variable {name} is not set"),
        };
        let roaming = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| missing("APPDATA"))?;
        let local = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| missing("LOCALAPPDATA"))?;
        let temp = std::env::var_os("TEMP")
            .map(PathBuf::from)
            .ok_or_else(|| missing("TEMP"))?;
        let userprofile = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .ok_or_else(|| missing("USERPROFILE"))?;
        if let Ok(exe) = std::env::current_exe()
            && let Some(exe_dir) = exe.parent()
            && Self::portable_flag_present(exe_dir)
        {
            return Self::portable(exe_dir);
        }
        Self::installed(&roaming, &local, &temp, &userprofile)
    }

    /// Installed layout: `%APPDATA%\<identifier>` and
    /// `%LOCALAPPDATA%\<identifier>` (03 §3.8). Pure — the test seam.
    pub fn installed(
        roaming: &Path,
        local: &Path,
        temp: &Path,
        userprofile: &Path,
    ) -> Result<Self, StoreError> {
        if roaming.as_os_str().is_empty()
            || local.as_os_str().is_empty()
            || temp.as_os_str().is_empty()
            || userprofile.as_os_str().is_empty()
        {
            return Err(StoreError::SaveFailed {
                detail: "base folders must not be empty".into(),
            });
        }
        Ok(Self {
            roaming_root: roaming.join(IDENTIFIER),
            local_root: local.join(IDENTIFIER),
            temp_root: temp.join(TEMP_FOLDER).join(pid_segment()),
            default_project_folder: userprofile.join("Videos").join(TEMP_FOLDER),
            portable: false,
        })
    }

    /// Portable layout: every folder of 03 §3.8 under `<exe folder>\data\…`.
    pub fn portable(exe_dir: &Path) -> Result<Self, StoreError> {
        if exe_dir.as_os_str().is_empty() {
            return Err(StoreError::SaveFailed {
                detail: "executable folder must not be empty".into(),
            });
        }
        let data = exe_dir.join("data");
        Ok(Self {
            roaming_root: data.clone(),
            local_root: data.clone(),
            temp_root: data.join("temp").join(pid_segment()),
            default_project_folder: data.join("Projects"),
            portable: true,
        })
    }

    /// Whether `portable.flag` exists next to the executable (03 §3.8).
    pub fn portable_flag_present(exe_dir: &Path) -> bool {
        exe_dir.join("portable.flag").exists()
    }

    /// True when the portable layout is active.
    pub fn is_portable(&self) -> bool {
        self.portable
    }

    /// `%APPDATA%\…\settings.json` (03 §3.8).
    pub fn settings_file(&self) -> PathBuf {
        self.roaming_root.join("settings.json")
    }

    /// `settings.json.bak`, the last-good copy (03 §3.6; REQ-SET-003).
    pub fn settings_backup(&self) -> PathBuf {
        self.roaming_root.join("settings.json.bak")
    }

    /// User library root `…\library\` (03 §3.8).
    pub fn library_root(&self) -> PathBuf {
        self.roaming_root.join("library")
    }

    /// One of the four library subfolders (03 §3.8).
    ///
    /// Returns the root itself for any other name; callers pass
    /// [`LIBRARY_SUBFOLDERS`] members.
    pub fn library_subfolder(&self, name: &str) -> PathBuf {
        self.library_root().join(name)
    }

    /// Cache folder `…\cache\` (03 §3.8) — the settings key
    /// `alvs.media.cacheFolder` may override it at the open site (§3.6).
    pub fn cache_dir(&self) -> PathBuf {
        self.local_root.join("cache")
    }

    /// `…\cache\cache.sqlite` (03 §3.7).
    pub fn cache_db(&self) -> PathBuf {
        self.cache_dir().join("cache.sqlite")
    }

    /// Speech-model folder `…\models\` (03 §3.8).
    pub fn models_dir(&self) -> PathBuf {
        self.local_root.join("models")
    }

    /// Logs folder `…\logs\` (03 §3.8).
    pub fn logs_dir(&self) -> PathBuf {
        self.local_root.join("logs")
    }

    /// Crash dumps and panic reports `…\crashes\` (03 §3.8).
    pub fn crashes_dir(&self) -> PathBuf {
        self.local_root.join("crashes")
    }

    /// `…\session.marker` (03 §3.8; §3.9.5).
    pub fn session_marker(&self) -> PathBuf {
        self.local_root.join("session.marker")
    }

    /// Previous-installer rollback `…\updates\` (03 §3.8).
    pub fn updates_dir(&self) -> PathBuf {
        self.local_root.join("updates")
    }

    /// Per-process temporary folder `%TEMP%\AfterLeads-Video-Spirits\<pid>\`
    /// (03 §3.8; 03 §3.1 job snapshots).
    pub fn temp_dir(&self) -> PathBuf {
        self.temp_root.clone()
    }

    /// Default new-project folder (03 §3.8): the
    /// `alvs.general.defaultProjectFolder` setting default.
    pub fn default_project_folder(&self) -> PathBuf {
        self.default_project_folder.clone()
    }

    /// SEC-04 containment roots for application writes: the app config
    /// folder, the local-data folder, and the per-process temp folder
    /// (05 §5.7; the project folder and user-picked folders are supplied
    /// per operation by callers).
    pub fn containment_roots(&self) -> Vec<PathBuf> {
        if self.portable {
            vec![self.roaming_root.clone(), self.temp_root.clone()]
        } else {
            vec![
                self.roaming_root.clone(),
                self.local_root.clone(),
                self.temp_root.clone(),
            ]
        }
    }
}

/// Per-PID folder segment (03 §3.8).
fn pid_segment() -> String {
    std::process::id().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvs_core::path::{PathDecision, PathOrigin, check_path};

    fn dirs(tag: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!("alvs-paths-{tag}-{}", pid_segment()));
        let (roaming, local, temp, user) = (
            base.join("roaming"),
            base.join("local"),
            base.join("tmp"),
            base.join("home-Ünïcode"), // Unicode roots must survive (03 §3.8)
        );
        for d in [&roaming, &local, &temp, &user] {
            std::fs::create_dir_all(d).unwrap_or(());
        }
        (roaming, local, temp, user)
    }

    #[test]
    fn installed_layout_matches_3_8_exactly() {
        let (roaming, local, temp, user) = dirs("installed");
        let Ok(p) = AppPaths::installed(&roaming, &local, &temp, &user) else {
            panic!("installed layout must resolve");
        };
        assert!(!p.is_portable());
        assert_eq!(
            p.settings_file(),
            roaming.join(IDENTIFIER).join("settings.json")
        );
        assert_eq!(
            p.settings_backup(),
            roaming.join(IDENTIFIER).join("settings.json.bak")
        );
        assert_eq!(p.library_root(), roaming.join(IDENTIFIER).join("library"));
        assert_eq!(
            p.library_subfolder("effect-presets"),
            roaming
                .join(IDENTIFIER)
                .join("library")
                .join("effect-presets")
        );
        assert_eq!(
            p.cache_db(),
            local.join(IDENTIFIER).join("cache").join("cache.sqlite")
        );
        assert_eq!(p.models_dir(), local.join(IDENTIFIER).join("models"));
        assert_eq!(p.logs_dir(), local.join(IDENTIFIER).join("logs"));
        assert_eq!(p.crashes_dir(), local.join(IDENTIFIER).join("crashes"));
        assert_eq!(
            p.session_marker(),
            local.join(IDENTIFIER).join("session.marker")
        );
        assert_eq!(p.updates_dir(), local.join(IDENTIFIER).join("updates"));
        assert_eq!(p.temp_dir(), temp.join(TEMP_FOLDER).join(pid_segment()));
        assert_eq!(
            p.default_project_folder(),
            user.join("Videos").join(TEMP_FOLDER)
        );
        std::fs::remove_dir_all(roaming.parent().unwrap_or(&temp)).unwrap_or(());
    }

    #[test]
    fn portable_layout_moves_everything_under_data() {
        let exe_dir = std::env::temp_dir().join(format!("alvs-portable-{}", pid_segment()));
        std::fs::create_dir_all(&exe_dir).unwrap_or(());
        let Ok(p) = AppPaths::portable(&exe_dir) else {
            panic!("portable layout must resolve");
        };
        assert!(p.is_portable());
        let data = exe_dir.join("data");
        assert_eq!(p.settings_file(), data.join("settings.json"));
        assert_eq!(p.library_root(), data.join("library"));
        assert_eq!(p.cache_db(), data.join("cache").join("cache.sqlite"));
        assert_eq!(p.logs_dir(), data.join("logs"));
        assert_eq!(p.session_marker(), data.join("session.marker"));
        assert!(p.temp_dir().starts_with(data.join("temp")));
        std::fs::remove_dir_all(&exe_dir).unwrap_or(());
    }

    #[test]
    fn portable_flag_detection() {
        let exe_dir = std::env::temp_dir().join(format!("alvs-flag-{}", pid_segment()));
        std::fs::create_dir_all(&exe_dir).unwrap_or(());
        assert!(!AppPaths::portable_flag_present(&exe_dir));
        std::fs::write(exe_dir.join("portable.flag"), b"").unwrap_or(());
        assert!(AppPaths::portable_flag_present(&exe_dir));
        std::fs::remove_dir_all(&exe_dir).unwrap_or(());
    }

    #[test]
    fn containment_roots_hold_their_own_targets() {
        let (roaming, local, temp, user) = dirs("contain");
        let Ok(p) = AppPaths::installed(&roaming, &local, &temp, &user) else {
            panic!("installed layout must resolve");
        };
        let logs = p.logs_dir();
        std::fs::create_dir_all(&logs).unwrap_or(());
        let target = logs.join("app.log");
        std::fs::write(&target, b"{}").unwrap_or(());
        assert!(matches!(
            check_path(&target, PathOrigin::UserPicked, &p.containment_roots()),
            PathDecision::Allowed
        ));
        // A file outside every root is denied.
        let outside = std::env::temp_dir().join("alvs-paths-outside.bin");
        std::fs::write(&outside, b"{}").unwrap_or(());
        assert!(matches!(
            check_path(&outside, PathOrigin::UserPicked, &p.containment_roots()),
            PathDecision::Denied(_)
        ));
        std::fs::remove_file(outside).unwrap_or(());
        std::fs::remove_dir_all(roaming.parent().unwrap_or(&temp)).unwrap_or(());
    }

    #[test]
    fn empty_roots_are_rejected() {
        assert!(
            AppPaths::installed(
                Path::new(""),
                Path::new("l"),
                Path::new("t"),
                Path::new("u")
            )
            .is_err()
        );
        assert!(AppPaths::portable(Path::new("")).is_err());
    }
}
