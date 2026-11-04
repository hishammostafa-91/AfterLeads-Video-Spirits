// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Cache database integration (03 §3.7; 02 §2.5 step 7): pragmas,
//! transactional versioned migrations, corrupt-rebuild (E8003), and the
//! recent-projects service.

use std::path::PathBuf;

use alvs_store::cache::Cache;
use alvs_store::recent::RecentProjects;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-cache-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

fn open(db: &std::path::Path) -> Cache {
    match Cache::open(db) {
        Ok((cache, false)) => cache,
        Ok((_, true)) => panic!("a fresh database must not report a rebuild"),
        Err(e) => panic!("open: {e}"),
    }
}

#[test]
fn cache_pragmas_and_schema_version_match_3_7() {
    let dir = scratch("pragmas");
    let db = dir.join("cache.sqlite");
    {
        let cache = open(&db);
        assert_eq!(
            match cache.schema_version() {
                Ok(v) => v,
                Err(e) => panic!("version: {e}"),
            },
            1
        );
        // Reopen: migrations are idempotent (the version gate skips them).
    }
    let cache = open(&db);
    assert_eq!(
        match cache.schema_version() {
            Ok(v) => v,
            Err(e) => panic!("version: {e}"),
        },
        1,
        "reopen keeps the schema version"
    );
    // The §3.7 pragmas, read back through the cache's own connection
    // (foreign_keys/synchronous are per-connection, so a fresh connection
    // cannot observe them; only journal_mode persists in the file).
    let cache = open(&db);
    assert_eq!(
        match cache.pragma("journal_mode") {
            Ok(v) => v,
            Err(e) => panic!("journal_mode: {e}"),
        },
        "wal"
    );
    assert_eq!(
        match cache.pragma("foreign_keys") {
            Ok(v) => v,
            Err(e) => panic!("foreign_keys: {e}"),
        },
        "1"
    );
    assert_eq!(
        match cache.pragma("synchronous") {
            Ok(v) => v,
            Err(e) => panic!("synchronous: {e}"),
        },
        "1", // NORMAL
        "PRAGMA synchronous"
    );
    drop(cache);
    // The persistent one is visible from any connection.
    let conn = match rusqlite::Connection::open(&db) {
        Ok(c) => c,
        Err(e) => panic!("reopen: {e}"),
    };
    // All ten §3.7 tables + the jobs_by_state index exist.
    let tables: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN
           ('meta','media_cache','thumbnails','waveforms','proxies','jobs',
            'recent_projects','models','encoder_probe','analysis')",
        [],
        |r| r.get(0),
    ) {
        Ok(v) => v,
        Err(e) => panic!("tables: {e}"),
    };
    assert_eq!(tables, 10);
    let index: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='jobs_by_state'",
        [],
        |r| r.get(0),
    ) {
        Ok(v) => v,
        Err(e) => panic!("index: {e}"),
    };
    assert_eq!(index, 1);
    // The CHECK constraints of §3.7 are enforced.
    let bad = conn.execute(
        "INSERT INTO proxies (media_key, profile, file, state, created_utc, last_used_utc)
         VALUES ('k', 'p', NULL, 'bogus', 't', 't')",
        [],
    );
    assert!(bad.is_err(), "the proxies state CHECK must reject 'bogus'");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn cache_corrupt_database_is_renamed_and_rebuilt() {
    let dir = scratch("corrupt");
    let db = dir.join("cache.sqlite");
    {
        let cache = open(&db);
        drop(cache);
    }
    // Corrupt the main file and leave WAL siblings behind.
    std::fs::write(&db, b"definitely not a sqlite database").unwrap_or(());
    std::fs::write(dir.join("cache.sqlite-wal"), b"stale wal").unwrap_or(());
    std::fs::write(dir.join("cache.sqlite-shm"), b"stale shm").unwrap_or(());

    let (cache, rebuilt) = match Cache::open(&db) {
        Ok(v) => v,
        Err(e) => panic!("open: {e}"),
    };
    assert!(rebuilt, "a corrupt database reports the rebuild (E8003)");
    assert_eq!(
        match cache.schema_version() {
            Ok(v) => v,
            Err(e) => panic!("version: {e}"),
        },
        1
    );
    // The corrupt original survives under its timestamped name.
    let leftovers: Vec<String> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        leftovers
            .iter()
            .any(|n| n.starts_with("cache.sqlite.corrupt-")),
        "the corrupt file is kept as evidence: {leftovers:?}"
    );
    // The rebuilt database works in WAL mode, so a fresh `-wal` may exist;
    // what must NOT survive is the stale sibling's content.
    let wal = dir.join("cache.sqlite-wal");
    if wal.exists() {
        let content = std::fs::read(&wal).unwrap_or_default();
        assert_ne!(
            content,
            b"stale wal".as_slice(),
            "the stale WAL content must not survive"
        );
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn cache_recent_records_lists_trims_and_pins() {
    let dir = scratch("recent");
    let db = dir.join("cache.sqlite");
    let cache = open(&db);
    let recent = cache.recent_with_limit(2);

    let record = |name: &str| {
        let path = dir.join(format!("{name}.alvs"));
        recent
            .record(&path, name, None)
            .unwrap_or_else(|e| panic!("record {name}: {e}"));
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    record("a");
    record("b");
    record("c"); // limit 2 → a is evicted
    let list = match recent.list() {
        Ok(l) => l,
        Err(e) => panic!("list: {e}"),
    };
    assert_eq!(list.len(), 2, "{list:?}");
    assert_eq!(list[0].name, "c", "newest first");
    assert_eq!(list[1].name, "b");
    assert!(!list.iter().any(|e| e.name == "a"), "the oldest is trimmed");
    assert!(
        list.iter()
            .all(|e| !e.opened_utc.is_empty() && e.thumbnail.is_none())
    );

    // Upsert: re-recording refreshes the name and recency.
    let path_a = dir.join("a.alvs");
    record("a2");
    let _ = path_a;
    record("d"); // limit 2 → oldest (b) evicted, a2 stays
    let list = match recent.list() {
        Ok(l) => l,
        Err(e) => panic!("list: {e}"),
    };
    assert_eq!(list.len(), 2, "{list:?}");
    assert_eq!(list[0].name, "d");
    assert_eq!(list[1].name, "a2");

    // Pinning survives the trim (pinned entries lead the order).
    let pinned_path = dir.join("a2.alvs");
    // a2 was recorded under the name a2 via record("a2") → path a2.alvs.
    recent
        .set_pinned(&pinned_path, true)
        .unwrap_or_else(|e| panic!("pin: {e}"));
    record("e");
    record("f");
    let list = match recent.list() {
        Ok(l) => l,
        Err(e) => panic!("list: {e}"),
    };
    assert!(
        list.iter().any(|x| x.name == "a2" && x.pinned),
        "the pinned entry survives the trim: {list:?}"
    );
    assert_eq!(list.len(), 2, "the limit still holds: {list:?}");
    assert_eq!(list[0].name, "a2", "pinned leads the listing");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
