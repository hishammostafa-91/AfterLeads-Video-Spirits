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

/// The SQLite-backed [`RecentProjects`] over `recent_projects` (03 §3.7;
/// research D9): upsert on save/open with `opened_utc`, newest-first
/// listing with pinned entries leading, and trimming to the limit where
/// pinned entries always survive (the canonical trim names the count, not
/// the eviction of pins).
pub struct RecentSqlite {
    conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    limit: u32,
}

impl RecentSqlite {
    /// Crate constructor: the connection comes from [`crate::cache::Cache`].
    pub(crate) fn new(
        conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
        limit: u32,
    ) -> Self {
        Self { conn, limit }
    }
}

impl RecentProjects for RecentSqlite {
    fn record(&self, path: &Path, name: &str, thumb: Option<&Path>) -> Result<(), StoreError> {
        let opened = crate::utc::iso_utc(crate::utc::epoch_ms_now()).unwrap_or_default();
        let conn = self.conn.lock().map_err(|_| StoreError::SaveFailed {
            detail: "recent-projects lock poisoned".into(),
        })?;
        conn.execute(
            "INSERT INTO recent_projects (path, name, thumbnail_file, opened_utc, pinned)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(path) DO UPDATE
               SET name = excluded.name,
                   thumbnail_file = excluded.thumbnail_file,
                   opened_utc = excluded.opened_utc",
            rusqlite::params![
                path.to_string_lossy(),
                name,
                thumb.map(|t| t.to_string_lossy().into_owned()),
                opened,
            ],
        )
        .map_err(|e| StoreError::SaveFailed {
            detail: format!("recent insert failed: {e}"),
        })?;
        // Trim to the newest `limit` rows overall; pinned entries lead
        // the ordering, so they always survive (03 §3.6
        // `alvs.general.recentLimit`).
        conn.execute(
            "DELETE FROM recent_projects
             WHERE path NOT IN (
               SELECT path FROM recent_projects
                ORDER BY pinned DESC, opened_utc DESC, rowid DESC
                LIMIT ?1
             )",
            [self.limit.max(1) as i64],
        )
        .map_err(|e| StoreError::SaveFailed {
            detail: format!("recent trim failed: {e}"),
        })?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<RecentEntry>, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::SaveFailed {
            detail: "recent-projects lock poisoned".into(),
        })?;
        let mut stmt = conn
            .prepare(
                "SELECT path, name, thumbnail_file, opened_utc, pinned
                 FROM recent_projects
                 ORDER BY pinned DESC, opened_utc DESC, rowid DESC",
            )
            .map_err(|e| StoreError::SaveFailed {
                detail: format!("recent select failed: {e}"),
            })?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RecentEntry {
                    path: PathBuf::from(row.get::<_, String>(0).unwrap_or_default()),
                    name: row.get::<_, String>(1).unwrap_or_default(),
                    thumbnail: row
                        .get::<_, Option<String>>(2)
                        .ok()
                        .flatten()
                        .map(PathBuf::from),
                    opened_utc: row.get::<_, String>(3).unwrap_or_default(),
                    pinned: row.get::<_, i64>(4).unwrap_or(0) != 0,
                })
            })
            .map_err(|e| StoreError::SaveFailed {
                detail: format!("recent query failed: {e}"),
            })?;
        let mut entries = Vec::new();
        for row in rows {
            let entry = row.map_err(|e| StoreError::SaveFailed {
                detail: format!("recent row failed: {e}"),
            })?;
            entries.push(entry);
        }
        Ok(entries)
    }

    fn set_pinned(&self, path: &Path, pinned: bool) -> Result<(), StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::SaveFailed {
            detail: "recent-projects lock poisoned".into(),
        })?;
        conn.execute(
            "UPDATE recent_projects SET pinned = ?2 WHERE path = ?1",
            rusqlite::params![path.to_string_lossy(), i64::from(pinned)],
        )
        .map_err(|e| StoreError::SaveFailed {
            detail: format!("recent pin failed: {e}"),
        })?;
        Ok(())
    }
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
        // Preserve the pinned flag on re-record, mirroring the SQL
        // upsert (the two trait implementations share one semantics;
        // review F-15).
        let pinned = entries
            .iter()
            .find(|e| e.path == path)
            .map(|e| e.pinned)
            .unwrap_or(false);
        entries.retain(|e| e.path != path);
        entries.insert(
            0,
            RecentEntry {
                path: path.to_path_buf(),
                name: name.to_string(),
                thumbnail: thumb.map(Path::to_path_buf),
                opened_utc: crate::utc::iso_utc(crate::utc::epoch_ms_now()).unwrap_or_default(),
                pinned,
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
        // Re-recording (a new open/save) must NOT drop the pin — the SQL
        // upsert preserves it, so the memory implementation does too
        // (review F-15).
        recent
            .record(Path::new("/p/x.alvs"), "X2", None)
            .unwrap_or(());
        let list = match recent.list() {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(list[0].name, "X2");
        assert!(list[0].pinned, "re-record preserves the pin");
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
