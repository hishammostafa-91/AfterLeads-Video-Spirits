// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Recent projects (docs/03 §3.7): the service trait every `ProjectStore`
//! records through, plus an in-memory implementation for tests and
//! composition before the SQLite cache arrives (research D2/D9). The
//! SQLite implementation lands with the cache (T025).

use std::path::{Path, PathBuf};

use crate::error::StoreError;

/// One `recent_projects` row (03 §3.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentEntry {
    /// Project file path (primary key).
    pub path: PathBuf,
    /// Display name.
    pub name: String,
    /// Thumbnail file reference, written by later packages (nullable).
    pub thumbnail: Option<PathBuf>,
    /// Last-opened time (ISO-8601 UTC).
    pub opened_utc: String,
    /// Pinned flag.
    pub pinned: bool,
}

/// The recent-projects service (03 §3.7; §3.9.2): `record` upserts on
/// save/open and trims to `alvs.general.recentLimit` (D9).
pub trait RecentProjects: Send + Sync {
    /// Upsert a project (newest `opened_utc`), trimming to the keep limit.
    fn record(&self, path: &Path, name: &str, thumb: Option<&Path>) -> Result<(), StoreError>;
    /// Newest-first listing.
    fn list(&self) -> Result<Vec<RecentEntry>, StoreError>;
    /// Pin or unpin an entry.
    fn set_pinned(&self, path: &Path, pinned: bool) -> Result<(), StoreError>;
}

/// In-memory [`RecentProjects`] for tests and early composition (the
/// SQLite implementation is the production one, T025).
#[derive(Debug, Default)]
pub struct MemoryRecent {
    entries: std::sync::Mutex<Vec<RecentEntry>>,
    limit: usize,
}

impl MemoryRecent {
    /// A memory store holding at most `limit` entries.
    pub fn new(limit: usize) -> Self {
        Self {
            entries: std::sync::Mutex::new(Vec::new()),
            limit,
        }
    }
}

impl RecentProjects for MemoryRecent {
    fn record(&self, path: &Path, name: &str, thumb: Option<&Path>) -> Result<(), StoreError> {
        let mut entries = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => {
                return Err(StoreError::SaveFailed {
                    detail: "recent-projects lock poisoned".into(),
                });
            }
        };
        entries.retain(|e| e.path != path);
        entries.insert(
            0,
            RecentEntry {
                path: path.to_path_buf(),
                name: name.to_string(),
                thumbnail: thumb.map(Path::to_path_buf),
                opened_utc: crate::utc::iso_utc(crate::utc::epoch_ms_now()).unwrap_or_default(),
                pinned: false,
            },
        );
        entries.truncate(self.limit.max(1));
        Ok(())
    }

    fn list(&self) -> Result<Vec<RecentEntry>, StoreError> {
        let entries = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => {
                return Err(StoreError::SaveFailed {
                    detail: "recent-projects lock poisoned".into(),
                });
            }
        };
        Ok(entries.clone())
    }

    fn set_pinned(&self, path: &Path, pinned: bool) -> Result<(), StoreError> {
        let mut entries = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => {
                return Err(StoreError::SaveFailed {
                    detail: "recent-projects lock poisoned".into(),
                });
            }
        };
        for e in entries.iter_mut() {
            if e.path == path {
                e.pinned = pinned;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_upserts_newest_first_and_trims() {
        let recent = MemoryRecent::new(2);
        recent
            .record(Path::new("/p/a.alvs"), "A", None)
            .unwrap_or(());
        recent
            .record(Path::new("/p/b.alvs"), "B", None)
            .unwrap_or(());
        recent
            .record(Path::new("/p/a.alvs"), "A2", None)
            .unwrap_or(()); // upsert, not append
        let list = match recent.list() {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "A2");
        assert_eq!(list[1].name, "B");
        recent
            .record(Path::new("/p/c.alvs"), "C", None)
            .unwrap_or(()); // trims oldest
        let list = match recent.list() {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "C");
        assert_eq!(list[1].name, "A2");
    }

    #[test]
    fn pin_toggles_by_path() {
        let recent = MemoryRecent::new(5);
        recent
            .record(Path::new("/p/x.alvs"), "X", None)
            .unwrap_or(());
        recent
            .set_pinned(Path::new("/p/x.alvs"), true)
            .unwrap_or(());
        let list = match recent.list() {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert!(list[0].pinned);
        recent
            .set_pinned(Path::new("/p/x.alvs"), false)
            .unwrap_or(());
        let list = match recent.list() {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert!(!list[0].pinned);
    }
}
