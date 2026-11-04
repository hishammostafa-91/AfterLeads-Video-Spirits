// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Project lock files (docs/03 §3.9.4; REQ-PRJ-011). This module owns the
//! exclusive acquire (`CREATE_NEW` semantics via `OpenOptions::create_new`)
//! writing `{ pid, host, user, startedUtc, appVersion }`, the release on
//! close, and the lock-info reading. The liveness probe and full
//! take-over/read-only outcome rules arrive with T022 (US5) per the plan.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::StoreError;
use crate::utc;

/// Lock acquisition outcome (03 §3.9.4). The takeover refinement (dead
/// process, other host ≥ 24 h) is applied by the caller with
/// [`lock_decision`] (T022).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockOutcome {
    /// We created the lock exclusively.
    Acquired,
    /// A live same-host process holds it: open read-only (E1007).
    ReadOnlyHeld,
    /// The holder is dead, or another host older than 24 h: take-over is
    /// allowed after confirmation (03 §3.9.4).
    TakeoverAvailable,
}

/// The lock file body (03 §3.9.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockInfo {
    /// Holder process id.
    pub pid: u32,
    /// Holder host name.
    pub host: String,
    /// Holder user name.
    pub user: String,
    /// Start time, ISO-8601 UTC (03 §3.9.4).
    #[serde(rename = "startedUtc")]
    pub started_utc: String,
    /// Epoch milliseconds of `started_utc` for age comparisons.
    #[serde(rename = "startedMs")]
    pub started_ms: i64,
    /// Application version of the holder.
    #[serde(rename = "appVersion")]
    pub app_version: String,
}

/// A held (or just acquired) `<Name>.alvs.lock` (03 §3.8).
#[derive(Debug)]
pub struct LockFile {
    path: PathBuf,
}

/// The lock path next to the project file (03 §3.8).
pub fn lock_path(project: &Path) -> PathBuf {
    let mut name = project
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str(".lock");
    project
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name)
}

/// Gather this holder's identity for a fresh lock (03 §3.9.4).
fn holder_info() -> LockInfo {
    let now = utc::epoch_ms_now();
    LockInfo {
        pid: std::process::id(),
        host: std::env::var("COMPUTERNAME").unwrap_or_default(),
        user: std::env::var("USERNAME").unwrap_or_default(),
        started_utc: utc::iso_utc(now).unwrap_or_default(),
        started_ms: now,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Write the lock body with an immediate flush (visible to a colliding
/// opener right away).
fn write_info(path: &Path, info: &LockInfo) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec_pretty(info).map_err(|e| StoreError::SaveFailed {
        detail: format!("lock serialization failed: {e}"),
    })?;
    let mut file = match OpenOptions::new().write(true).open(path) {
        Ok(f) => f,
        Err(e) => return Err(StoreError::from_io(&e)),
    };
    if let Err(e) = file.write_all(&bytes) {
        return Err(StoreError::from_io(&e));
    }
    let _ = file.flush();
    Ok(())
}

impl LockFile {
    /// Acquire the lock exclusively (03 §3.9.4). `Ok((lock, Acquired))`
    /// means we created it; an existing lock is read back and classified
    /// by [`lock_decision`] (dead process / host age rules, T022 refines
    /// the liveness probe).
    pub fn acquire(project: &Path) -> Result<(Self, LockOutcome, Option<LockInfo>), StoreError> {
        let path = lock_path(project);
        let info = holder_info();
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => {
                write_info(&path, &info)?;
                Ok((Self { path }, LockOutcome::Acquired, None))
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing = read_info(&path)?;
                let decision = lock_decision(existing.as_ref());
                Ok((Self { path }, decision, existing))
            }
            Err(e) => Err(StoreError::from_io(&e)),
        }
    }

    /// Take over a lock whose holder is gone or expired (03 §3.9.4):
    /// remove the stale file and re-acquire exclusively.
    pub fn take_over(project: &Path) -> Result<(Self, LockOutcome, Option<LockInfo>), StoreError> {
        let stale = lock_path(project);
        match std::fs::remove_file(&stale) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(StoreError::from_io(&e)),
        }
        Self::acquire(project)
    }

    /// Release on close: remove the lock file (03 §3.9.4).
    pub fn release(self) -> Result<(), StoreError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::from_io(&e)),
        }
    }

    /// The lock file's path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Read a lock's body; a torn/unreadable lock is reported as a save
/// failure rather than guessed at.
pub fn read_info(path: &Path) -> Result<Option<LockInfo>, StoreError> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let info = serde_json::from_slice(&bytes).map_err(|e| StoreError::SaveFailed {
                detail: format!("lock file is unreadable: {e}"),
            })?;
            Ok(Some(info))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(StoreError::from_io(&e)),
    }
}

/// Classify an existing lock per 03 §3.9.4 with the injected liveness
/// verdict and the 24-hour other-host rule.
///
/// * dead process (any host) → [`LockOutcome::TakeoverAvailable`]
/// * live same-host process → [`LockOutcome::ReadOnlyHeld`]
/// * other host, lock older than 24 h → [`LockOutcome::TakeoverAvailable`]
/// * other host, younger → [`LockOutcome::ReadOnlyHeld`]
pub fn lock_decision_with(info: &LockInfo, holder_alive: bool, now_ms: i64) -> LockOutcome {
    if !holder_alive {
        return LockOutcome::TakeoverAvailable;
    }
    let same_host = info.host == std::env::var("COMPUTERNAME").unwrap_or_default();
    if same_host {
        return LockOutcome::ReadOnlyHeld;
    }
    let age = Duration::from_millis((now_ms - info.started_ms).max(0) as u64);
    if age >= Duration::from_secs(24 * 60 * 60) {
        LockOutcome::TakeoverAvailable
    } else {
        LockOutcome::ReadOnlyHeld
    }
}

/// Probe whether a process is alive (03 §3.9.4): open with query-only
/// access and wait with a zero timeout — a signaled process handle means
/// the process has terminated.
#[cfg(windows)]
pub fn process_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, WaitForSingleObject,
    };
    // SAFETY: (OD-11, docs/02 §2.14) OpenProcess is called with
    // QUERY_LIMITED_INFORMATION only — no handle duplication, no process
    // manipulation — on a raw u32 pid. The returned handle is used only
    // for a zero-timeout wait on ourselves and closed on every path.
    let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false; // cannot be opened: no such process (or no access)
    };
    // SAFETY: the handle comes from our own OpenProcess above and is valid
    // until CloseHandle; the wait never blocks (timeout 0).
    let wait = unsafe { WaitForSingleObject(handle, 0) };
    let alive = wait != WAIT_OBJECT_0; // signaled = terminated
    // SAFETY: closing the handle we opened above; the result is
    // deliberately ignored (the liveness verdict is already known).
    let _ = unsafe { CloseHandle(handle) };
    alive
}

/// Non-Windows development fallback (the product ships Windows-only):
/// conservatively reports unknown pids as alive so a lock is never
/// taken over on a false death verdict.
#[cfg(not(windows))]
pub fn process_alive(_pid: u32) -> bool {
    true
}

/// Classify an existing lock per 03 §3.9.4 with the real liveness probe
/// and the 24-hour other-host rule.
fn lock_decision(info: Option<&LockInfo>) -> LockOutcome {
    match info {
        Some(info) => lock_decision_with(info, process_alive(info.pid), utc::epoch_ms_now()),
        None => LockOutcome::TakeoverAvailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("alvs-lock-{tag}-{}", uuid::Uuid::now_v7().simple()));
        std::fs::create_dir_all(&dir).unwrap_or(());
        dir
    }

    #[test]
    fn lock_path_appends_the_suffix() {
        assert_eq!(
            lock_path(Path::new(r"C:\proj\demo.alvs")),
            PathBuf::from(r"C:\proj\demo.alvs.lock")
        );
        assert_eq!(
            lock_path(Path::new("bare.alvs")),
            PathBuf::from("bare.alvs.lock")
        );
    }

    #[test]
    fn acquire_is_exclusive_and_release_removes() {
        let dir = scratch("basic");
        let project = dir.join("p.alvs");
        let (lock, outcome, existing) = match LockFile::acquire(&project) {
            Ok(v) => v,
            Err(e) => panic!("acquire: {e}"),
        };
        assert_eq!(outcome, LockOutcome::Acquired);
        assert!(existing.is_none());
        // A second acquire sees the existing lock (conservative read-only
        // until the T022 liveness probe).
        let (_, outcome2, info2) = match LockFile::acquire(&project) {
            Ok(v) => v,
            Err(e) => panic!("second acquire: {e}"),
        };
        assert_eq!(outcome2, LockOutcome::ReadOnlyHeld);
        let info2 = match info2 {
            Some(i) => i,
            None => panic!("lock info must be readable"),
        };
        assert_eq!(info2.pid, std::process::id());
        assert!(!info2.host.is_empty());
        assert!(!info2.user.is_empty());
        assert!(!info2.started_utc.is_empty());
        assert!(!info2.app_version.is_empty());
        lock.release().unwrap_or(());
        assert!(!lock_path(&project).exists());
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn take_over_replaces_a_stale_lock() {
        let dir = scratch("takeover");
        let project = dir.join("q.alvs");
        let (first, _, _) = match LockFile::acquire(&project) {
            Ok(v) => v,
            Err(e) => panic!("acquire: {e}"),
        };
        let (second, outcome, _) = match LockFile::take_over(&project) {
            Ok(v) => v,
            Err(e) => panic!("take over: {e}"),
        };
        assert_eq!(outcome, LockOutcome::Acquired);
        second.release().unwrap_or(());
        first.release().unwrap_or(()); // already removed — NotFound is Ok
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn decision_rules_follow_3_9_4() {
        let info = LockInfo {
            pid: 1,
            host: "OTHER-HOST".into(),
            user: "u".into(),
            started_utc: "2026-01-01T00:00:00.000Z".into(),
            started_ms: 0,
            app_version: "0.1.0".into(),
        };
        let now = 25 * 60 * 60 * 1000; // 25 h later
        // Dead process: take-over regardless of host.
        assert_eq!(
            lock_decision_with(&info, false, now),
            LockOutcome::TakeoverAvailable
        );
        // Live other host, ≥ 24 h: take-over.
        assert_eq!(
            lock_decision_with(&info, true, now),
            LockOutcome::TakeoverAvailable
        );
        // Live other host, < 24 h: read-only.
        assert_eq!(
            lock_decision_with(&info, true, 60_000),
            LockOutcome::ReadOnlyHeld
        );
        // Live same host: read-only at any age.
        let mine = LockInfo {
            host: std::env::var("COMPUTERNAME").unwrap_or_default(),
            started_ms: 0,
            ..info.clone()
        };
        assert_eq!(
            lock_decision_with(&mine, true, 100 * 24 * 60 * 60 * 1000),
            LockOutcome::ReadOnlyHeld
        );
    }
}
