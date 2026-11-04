// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The SQLite cache database (docs/03 §3.7; 02 §2.5 step 7): open with the
//! mandated pragmas, run the embedded `NNN_<name>.sql` migrations in one
//! transaction with the version in `meta`, and rebuild a corrupt database
//! (renamed `cache.sqlite.corrupt-<timestamp>`) with E8003 (research D8).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;

use crate::error::StoreError;
use crate::recent::RecentSqlite;
use crate::utc;

/// The embedded migrations in `NNN_<name>.sql` order; the version lives
/// in `meta.schema_version` (03 §3.7).
const MIGRATIONS: &[(u32, &str)] = &[(1, include_str!("../migrations/001_init.sql"))];

/// The current schema version (the last migration's number).
pub const SCHEMA_VERSION: u32 = MIGRATIONS[MIGRATIONS.len() - 1].0;

/// An opened cache database.
pub struct Cache {
    conn: Arc<Mutex<Connection>>,
}

impl Cache {
    /// Open per 03 §3.7: pragmas, migrations, integrity check. Returns
    /// the cache plus whether a corrupt database was rebuilt (the caller
    /// surfaces E8003 from the flag; the cache is usable either way).
    pub fn open(db_path: &Path) -> Result<(Self, bool), StoreError> {
        if db_path.exists() {
            match Self::try_open(db_path) {
                Ok(cache) => return Ok((cache, false)),
                Err(StoreError::CacheRebuilt { .. }) => {
                    let (cache, _) = Self::rebuild(db_path)?;
                    return Ok((cache, true));
                }
                Err(e) => return Err(e),
            }
        }
        let cache = Self::open_fresh(db_path)?;
        Ok((cache, false))
    }

    /// Open and integrity-check an existing database; a failed
    /// `quick_check` yields [`StoreError::CacheRebuilt`] semantics (the
    /// caller rebuilds).
    fn try_open(db_path: &Path) -> Result<Self, StoreError> {
        let conn = Self::connect(db_path)?;
        Self::run_migrations(&conn)?;
        let check: String = conn
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(|e| StoreError::SaveFailed {
                detail: format!("quick_check failed: {e}"),
            })?;
        if check != "ok" {
            return Err(StoreError::CacheRebuilt {
                detail: "integrity check failed".into(),
            });
        }
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Rename the corrupt database aside (with its WAL siblings), recreate
    /// from scratch, and return the rebuilt cache (03 §3.7; E8003).
    fn rebuild(db_path: &Path) -> Result<(Self, bool), StoreError> {
        let stamp = utc::compact_utc(utc::epoch_ms_now()).unwrap_or_else(|| "unknown".into());
        let corrupt = db_path.with_extension(format!("sqlite.corrupt-{stamp}"));
        // The main file keeps the evidence under the corrupt name; the
        // WAL siblings are disposable.
        if db_path.exists()
            && let Err(e) = std::fs::rename(db_path, &corrupt)
        {
            return Err(StoreError::from_io(&e));
        }
        for sibling in [wal_path(db_path), shm_path(db_path)] {
            if sibling.exists() {
                let _ = std::fs::remove_file(sibling);
            }
        }
        let cache = Self::open_fresh(db_path)?;
        Ok((cache, true))
    }

    /// Create a fresh database with the pragmas and migrations.
    fn open_fresh(db_path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StoreError::from_io(&e))?;
        }
        let conn = Self::connect(db_path)?;
        Self::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Connect with the §3.7 pragmas: `journal_mode=WAL`,
    /// `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`.
    /// A file whose header is not a database (`SQLITE_NOTADB`/corrupt)
    /// maps to the rebuild path (03 §3.7).
    fn connect(db_path: &Path) -> Result<Connection, StoreError> {
        let conn = Connection::open(db_path).map_err(|e| StoreError::SaveFailed {
            detail: format!("cannot open cache database: {e}"),
        })?;
        conn.busy_timeout(Duration::from_millis(5000))
            .map_err(|e| sqlite_err("busy_timeout", e))?;
        // journal_mode returns the resulting mode as a row; a corrupt
        // header surfaces here first (before any quick_check could run).
        let mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|e| match sqlite_corrupt_kind(&e) {
                Some(detail) => StoreError::CacheRebuilt { detail },
                None => sqlite_err("journal_mode=WAL", e),
            })?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(StoreError::SaveFailed {
                detail: format!("journal_mode refused WAL: {mode}"),
            });
        }
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| sqlite_err("foreign_keys", e))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| sqlite_err("synchronous", e))?;
        Ok(conn)
    }

    /// Run pending migrations inside one transaction; the version is
    /// recorded in `meta.schema_version` (03 §3.7).
    fn run_migrations(conn: &Connection) -> Result<(), StoreError> {
        let current = read_schema_version(conn)?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| StoreError::SaveFailed {
                detail: format!("cannot begin migration transaction: {e}"),
            })?;
        for (version, sql) in MIGRATIONS {
            if *version <= current {
                continue;
            }
            tx.execute_batch(sql)
                .map_err(|e| StoreError::MigrationFailed {
                    detail: format!("migration {version} failed: {e}"),
                })?;
        }
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [SCHEMA_VERSION.to_string()],
        )
        .map_err(|e| StoreError::MigrationFailed {
            detail: format!("cannot record schema version: {e}"),
        })?;
        tx.commit().map_err(|e| StoreError::MigrationFailed {
            detail: format!("cannot commit migrations: {e}"),
        })
    }

    /// The recent-projects service over this database (03 §3.7; D9).
    pub fn recent(&self) -> RecentSqlite {
        RecentSqlite::new(Arc::clone(&self.conn), 20)
    }

    /// The recent-projects service with an explicit keep limit
    /// (`alvs.general.recentLimit`).
    pub fn recent_with_limit(&self, limit: u32) -> RecentSqlite {
        RecentSqlite::new(Arc::clone(&self.conn), limit)
    }

    /// The schema version recorded in `meta`.
    pub fn schema_version(&self) -> Result<u32, StoreError> {
        let conn = lock_conn(&self.conn)?;
        read_schema_version(&conn)
    }

    /// Read a pragma through this cache's connection (per-connection
    /// settings like `foreign_keys`/`synchronous` are invisible to a
    /// different connection; §3.7).
    pub fn pragma(&self, name: &str) -> Result<String, StoreError> {
        let conn = lock_conn(&self.conn)?;
        conn.query_row(&format!("PRAGMA {name}"), [], |row| {
            let as_int: Option<i64> = row.get(0).ok();
            let as_text: Option<String> = row.get(0).ok();
            Ok(as_text
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| as_int.map(|i| i.to_string()).unwrap_or_default()))
        })
        .map_err(|e| sqlite_err(name, e))
    }
}

/// `-wal` sibling path of a database.
fn wal_path(db: &Path) -> PathBuf {
    let mut name = db
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str("-wal");
    db.parent().unwrap_or_else(|| Path::new(".")).join(name)
}

/// `-shm` sibling path of a database.
fn shm_path(db: &Path) -> PathBuf {
    let mut name = db
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str("-shm");
    db.parent().unwrap_or_else(|| Path::new(".")).join(name)
}

fn lock_conn(
    conn: &Arc<Mutex<Connection>>,
) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
    conn.lock().map_err(|_| StoreError::SaveFailed {
        detail: "cache connection lock poisoned".into(),
    })
}

/// Map a rusqlite failure with the failing statement's name (helper).
fn sqlite_err(what: &str, e: rusqlite::Error) -> StoreError {
    StoreError::SaveFailed {
        detail: format!("{what} failed: {e}"),
    }
}

/// Whether a rusqlite failure is database corruption (`SQLITE_NOTADB`
/// extended code 26 or `SQLITE_CORRUPT` 11); returns the rebuild detail.
fn sqlite_corrupt_kind(e: &rusqlite::Error) -> Option<String> {
    if let rusqlite::Error::SqliteFailure(ffi, _) = e
        && (ffi.extended_code == 26 || ffi.extended_code == 11)
    {
        return Some(format!("database is corrupt ({})", ffi.extended_code));
    }
    None
}

/// Read `meta.schema_version` (0 when the table does not exist yet).
fn read_schema_version(conn: &Connection) -> Result<u32, StoreError> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='meta')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StoreError::SaveFailed {
            detail: format!("cannot inspect meta: {e}"),
        })?;
    if !exists {
        return Ok(0);
    }
    let version: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StoreError::SaveFailed {
            detail: format!("cannot read schema version: {e}"),
        })?;
    version
        .and_then(|v| v.parse().ok())
        .ok_or(StoreError::SaveFailed {
            detail: "schema_version is not a number".into(),
        })
}
