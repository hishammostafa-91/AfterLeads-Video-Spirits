// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The startup services the shell owns (02 s2.5, the steps that
//! exist at WP-05): resolved app paths and the settings document
//! (step 3; read for defaults until the settings IPC ships with
//! WP-24) and the cache database with the recent-projects service
//! (step 7). Steps 2/6/8/9/11 belong to their work packages.

use std::sync::Arc;

use alvs_store::cache::Cache;
use alvs_store::paths::AppPaths;
use alvs_store::recent::{MemoryRecent, RecentProjects};
use alvs_store::settings::Settings;

/// The composed startup services.
pub struct Services {
    /// Resolved application paths (03 s3.8; portable layout honored).
    pub paths: AppPaths,
    /// The settings document (defaults + user values; 03 s3.6).
    pub settings: Settings,
    /// The recent-projects service (SQLite over the cache database).
    pub recent: Arc<dyn RecentProjects>,
    /// The cache handle when it opened; `None` after an open failure
    /// (the cache rebuild flow is the cache WP's, E8003 - the shell
    /// only degrades recents to in-memory until restart).
    pub cache: Option<Arc<Cache>>,
}

impl Services {
    /// Compose per the startup order. The cache opens with its
    /// migrations (02 s2.5 step 7); a failure degrades the recents
    /// service in memory rather than blocking the shell.
    pub fn open() -> Self {
        let paths = AppPaths::resolve().unwrap_or_else(|error| {
            eprintln!("app paths resolution failed ({error}); falling back to the portable layout");
            AppPaths::portable(
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
                    .as_deref()
                    .unwrap_or(std::path::Path::new(".")),
            )
            .expect("a non-empty executable folder always resolves")
        });
        let settings = Settings::load(&paths.settings_file(), &paths.settings_backup());
        let recent_limit = settings
            .get("alvs.general.recentLimit")
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(10);
        match Cache::open(&paths.cache_db()) {
            Ok((cache, _migrated)) => {
                let cache = Arc::new(cache);
                let recent: Arc<dyn RecentProjects> =
                    Arc::new(cache.recent_with_limit(recent_limit));
                Self {
                    paths,
                    settings,
                    recent,
                    cache: Some(cache),
                }
            }
            Err(error) => {
                eprintln!(
                    "cache database open failed ({error}); recents are in-memory for this session"
                );
                Self {
                    paths,
                    settings,
                    recent: Arc::new(MemoryRecent::new(
                        usize::try_from(recent_limit).unwrap_or(10),
                    )),
                    cache: None,
                }
            }
        }
    }
}
