// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Timestamped project backups (docs/03 §3.8 layout, §3.9.2 save step;
//! REQ-PRJ-004): one `Backups\<Name>_<yyyyMMdd-HHmmss>.alvs` per manual
//! save, pruned to `alvs.general.backupsKeep`, listable with time and
//! size, restorable into a new file.

use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::error::StoreError;
use crate::utc;

/// The `Backups\` folder of a project (03 §3.8).
pub fn backups_dir(project: &Path) -> PathBuf {
    project
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("Backups")
}

/// Write one timestamped backup of the just-saved bytes (03 §3.9.2) and
/// return its path. Two saves within the same second share the name; the
/// atomic write makes that an overwrite, never a torn file.
pub fn write_backup(project: &Path, bytes: &[u8], now_ms: i64) -> Result<PathBuf, StoreError> {
    let stem = project
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stamp = utc::compact_utc(now_ms).ok_or_else(|| StoreError::SaveFailed {
        detail: "backup timestamp out of representable range".into(),
    })?;
    let dir = backups_dir(project);
    std::fs::create_dir_all(&dir).map_err(|e| StoreError::from_io(&e))?;
    let target = dir.join(format!("{stem}_{stamp}.alvs"));
    atomic_write(&target, bytes)?;
    Ok(target)
}

/// One listed backup with the data DLG-16 shows (REQ-PRJ-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupEntry {
    /// The backup file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size: u64,
    /// Last-modified time, ISO-8601 UTC.
    pub modified_utc: String,
    /// The creation stamp parsed from the name (`yyyyMMdd-HHmmss`),
    /// milliseconds since the epoch — the backup's canonical ordering key.
    pub created_ms: Option<i64>,
}

/// Parse `<stem>_<yyyyMMdd-HHmmss>.alvs` back to the stamp's epoch
/// milliseconds (None for foreign names).
fn stamp_of(path: &Path) -> Option<i64> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(".alvs")?;
    let stamp = stem.rsplit_once('_')?.1;
    utc::parse_compact_utc(stamp)
}

/// List backups newest-first by their name stamp with time and size
/// (REQ-PRJ-004; DLG-16 data). Files whose names carry no parseable stamp
/// sort last (path order).
pub fn list(project: &Path) -> Result<Vec<BackupEntry>, StoreError> {
    let dir = backups_dir(project);
    let mut entries: Vec<BackupEntry> = Vec::new();
    let read_dir = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
        Err(e) => return Err(StoreError::from_io(&e)),
    };
    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => return Err(StoreError::from_io(&e)),
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("alvs") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => return Err(StoreError::from_io(&e)),
        };
        let modified = meta
            .modified()
            .ok()
            .map(|t| {
                let ms = t
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
                    .unwrap_or(0);
                utc::iso_utc(ms).unwrap_or_default()
            })
            .unwrap_or_default();
        entries.push(BackupEntry {
            created_ms: stamp_of(&path),
            path,
            size: meta.len(),
            modified_utc: modified,
        });
    }
    // Newest stamp first; same-stamp (same-second overwrite) by path for
    // determinism; unparseable names last.
    entries.sort_by(|a, b| match (a.created_ms, b.created_ms) {
        (Some(x), Some(y)) => y.cmp(&x).then_with(|| a.path.cmp(&b.path)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.path.cmp(&b.path),
    });
    Ok(entries)
}

/// Prune to the newest `keep` backups (REQ-PRJ-004) and return the removed
/// paths (the oldest tail).
pub fn prune(project: &Path, keep: u32) -> Result<Vec<PathBuf>, StoreError> {
    let mut entries = list(project)?;
    let keep = keep as usize;
    if entries.len() <= keep {
        return Ok(Vec::new());
    }
    let tail: Vec<BackupEntry> = entries.split_off(keep);
    let mut removed = Vec::new();
    for entry in &tail {
        match std::fs::remove_file(&entry.path) {
            Ok(()) => removed.push(entry.path.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(StoreError::from_io(&e)),
        }
    }
    Ok(removed)
}

/// Restore a backup into a **new** file next to the original (REQ-PRJ-004:
/// "restores into a new file"); the backup itself is untouched.
pub fn restore(backup: &Path, new_target: &Path) -> Result<PathBuf, StoreError> {
    let bytes = std::fs::read(backup).map_err(|e| StoreError::from_io(&e))?;
    atomic_write(new_target, &bytes)?;
    Ok(new_target.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alvs-backup-{tag}-{}",
            uuid::Uuid::now_v7().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap_or(());
        dir
    }

    #[test]
    fn writes_timestamped_backups_and_prunes_to_keep() {
        let dir = scratch("prune");
        let project = dir.join("doc.alvs");
        // 5 saves at distinct seconds, keep 3.
        let stamps: Vec<String> = (0..5i64)
            .map(|i| {
                let now = 1_700_000_000_000 + i * 1000;
                let stamp = match utc::compact_utc(now) {
                    Some(s) => s,
                    None => panic!("stamp of {now}"),
                };
                write_backup(&project, format!("v{i}").as_bytes(), now)
                    .map(drop)
                    .unwrap_or_else(|e| panic!("backup {i}: {e}"));
                stamp
            })
            .collect();
        let entries = match list(&project) {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(entries.len(), 5);
        // Newest first (stamps 4..0), each entry carrying its parsed stamp.
        for (i, entry) in entries.iter().enumerate() {
            let want = &stamps[4 - i];
            assert!(
                entry.path.to_string_lossy().contains(want.as_str()),
                "position {i}: {:?} should carry stamp {want}",
                entry.path
            );
            assert_eq!(
                entry.created_ms,
                Some(1_700_000_000_000 + (4 - i) as i64 * 1000)
            );
            assert!(entry.size > 0, "size is listed for DLG-16");
            assert!(!entry.modified_utc.is_empty(), "time is listed");
        }
        let removed = match prune(&project, 3) {
            Ok(r) => r,
            Err(e) => panic!("prune: {e}"),
        };
        assert_eq!(removed.len(), 2);
        // The OLDEST two (stamps 0 and 1) are removed; the newest stay.
        for old in [&stamps[0], &stamps[1]] {
            assert!(
                removed
                    .iter()
                    .any(|p| p.to_string_lossy().contains(old.as_str())),
                "stamp {old} should be pruned: {removed:?}"
            );
        }
        let entries = match list(&project) {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(entries.len(), 3);
        for kept in [&stamps[2], &stamps[3], &stamps[4]] {
            assert!(
                entries
                    .iter()
                    .any(|e| e.path.to_string_lossy().contains(kept.as_str())),
                "stamp {kept} should be kept"
            );
        }
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn restore_copies_into_a_new_file_untouched() {
        let dir = scratch("restore");
        let project = dir.join("r.alvs");
        let backup = match write_backup(&project, b"backup-bytes", 1_700_000_000_000) {
            Ok(p) => p,
            Err(e) => panic!("backup: {e}"),
        };
        let new_target = dir.join("restored.alvs");
        let out = match restore(&backup, &new_target) {
            Ok(p) => p,
            Err(e) => panic!("restore: {e}"),
        };
        assert_eq!(out, new_target);
        assert_eq!(
            std::fs::read_to_string(&new_target).unwrap_or_default(),
            "backup-bytes"
        );
        assert!(backup.exists(), "the backup itself is untouched");
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn same_second_saves_overwrite_atomically() {
        let dir = scratch("same-second");
        let project = dir.join("s.alvs");
        let now = 1_700_000_000_000;
        write_backup(&project, b"a", now).map(drop).unwrap_or(());
        write_backup(&project, b"b", now).map(drop).unwrap_or(());
        let entries = match list(&project) {
            Ok(l) => l,
            Err(e) => panic!("list: {e}"),
        };
        assert_eq!(entries.len(), 1, "same second → one name, overwritten");
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }
}
