// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The atomic write primitive (docs/03 §3.9.1): write `<target>.tmp` in the
//! same folder → flush (`FlushFileBuffers`) → replace with
//! `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`. Any
//! failure keeps the original file untouched and reports E8001/E8004/E1008
//! (REQ-REL-007). The `Fs` seam is always compiled (never `#[cfg(test)]`-gated)
//! so unit tests can inject failures at each step (research D12/D13).

use std::io;
use std::path::{Path, PathBuf};

use alvs_core::path::{PathDecision, check_write_path};

use crate::error::StoreError;

/// The filesystem steps of §3.9.1, as a seam for step-wise failure
/// injection (research D13). Production code uses [`STD_FS`].
pub(crate) trait Fs {
    /// Write `bytes` to the temporary file (created fresh).
    fn write_tmp(&self, tmp: &Path, bytes: &[u8]) -> io::Result<()>;
    /// Flush the temporary file's buffers to disk (`FlushFileBuffers`).
    fn flush(&self, tmp: &Path) -> io::Result<()>;
    /// Atomically replace `target` with `tmp` (§3.9.1 flag pair).
    fn replace(&self, tmp: &Path, target: &Path) -> io::Result<()>;
    /// Best-effort cleanup of a failed temporary file.
    fn remove(&self, tmp: &Path) -> io::Result<()>;
}

/// The production filesystem (std + the Win32 replace call).
pub(crate) struct StdFs;

impl Fs for StdFs {
    fn write_tmp(&self, tmp: &Path, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(tmp, bytes)
    }

    fn flush(&self, tmp: &Path) -> io::Result<()> {
        // FlushFileBuffers requires a handle with write access (Win32);
        // a read-only File::open handle fails with ERROR_ACCESS_DENIED.
        std::fs::OpenOptions::new()
            .write(true)
            .open(tmp)?
            .sync_all()
    }

    fn replace(&self, tmp: &Path, target: &Path) -> io::Result<()> {
        replace_file(tmp, target)
    }

    fn remove(&self, tmp: &Path) -> io::Result<()> {
        match std::fs::remove_file(tmp) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// The §3.9.1 replace on Windows: `MoveFileExW` with exactly the mandated
/// flag pair (`std::fs::rename` passes only `MOVEFILE_REPLACE_EXISTING`).
#[cfg(windows)]
fn replace_file(tmp: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let mut src: Vec<u16> = tmp.as_os_str().encode_wide().collect();
    src.push(0);
    let mut dst: Vec<u16> = target.as_os_str().encode_wide().collect();
    dst.push(0);
    // SAFETY: (OD-11, docs/02 §2.14) MoveFileExW is called with two valid
    // NUL-terminated UTF-16 paths alive for the call's duration and the
    // docs/03 §3.9.1 flag pair. The operation is atomic at the filesystem
    // level: `target` either keeps its previous content or becomes `tmp`'s
    // content; there is no intermediate state and no handle to leak.
    let result = unsafe {
        MoveFileExW(
            windows::core::PCWSTR::from_raw(src.as_ptr()),
            windows::core::PCWSTR::from_raw(dst.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    result.map_err(|e| {
        // windows-core captures GetLastError() as HRESULT_FROM_WIN32
        // (0x8007_0000 | code); decode it back to the raw Win32 code.
        let h = e.code().0 as u32;
        if (h & 0xFFFF_0000) == 0x8007_0000 {
            io::Error::from_raw_os_error((h & 0x0000_FFFF) as i32)
        } else {
            io::Error::other(e.to_string())
        }
    })
}

/// Non-Windows development fallback (the product ships Windows-only,
/// docs/00 §0.3): the same steps without the write-through flag.
#[cfg(not(windows))]
fn replace_file(tmp: &Path, target: &Path) -> io::Result<()> {
    std::fs::rename(tmp, target)
}

/// Atomically write `bytes` to `target` per docs/03 §3.9.1.
///
/// On any failure the previous content of `target` is intact, the
/// temporary file is removed best-effort, and the error is one of
/// E8001/E8004/E1008 (REQ-REL-007; research D12).
///
/// Crate-internal: callers inside `alvs-store` construct targets from
/// the §3.8 roots or the project folder (SEC-04 containment by
/// construction, proven by the G7 suite). External callers must use
/// [`atomic_write_contained`], which enforces the SEC-04 gate (review
/// F-11).
pub(crate) fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    atomic_write_with(&StdFs, target, bytes)
}

/// The SEC-04-checked public form (05 §5.7; FR-017; review F-11): the
/// target must be contained by one of `roots` (the project folder, an
/// app folder, or a user-picked folder) before anything is written;
/// a rejected target yields E9005 and touches nothing.
pub fn atomic_write_contained(
    target: &Path,
    bytes: &[u8],
    roots: &[PathBuf],
) -> Result<(), StoreError> {
    match check_write_path(target, roots) {
        PathDecision::Allowed => atomic_write(target, bytes),
        // Unreachable under writer semantics (the prelude confirms only
        // ProjectFile-origin UNC); kept for exhaustiveness and treated
        // conservatively.
        PathDecision::NeedsConfirmation { path } => Err(StoreError::PathRejected {
            detail: format!("UNC target needs confirmation: {}", path.display()),
        }),
        PathDecision::Denied(e) => Err(StoreError::PathRejected {
            detail: format!("write target rejected by SEC-04: {e}"),
        }),
    }
}

/// The seam-driven core (research D13); unit tests inject failures here.
pub(crate) fn atomic_write_with(
    fs: &dyn Fs,
    target: &Path,
    bytes: &[u8],
) -> Result<(), StoreError> {
    let tmp = tmp_path(target);
    let step = |e: &io::Error, what: &str| -> StoreError {
        let mut mapped = StoreError::from_io(e);
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match &mut mapped {
            StoreError::SaveFailed { detail }
            | StoreError::DiskFull { detail }
            | StoreError::WriteFailed { detail } => {
                *detail = format!("{what} of {name} failed: {e}");
            }
            StoreError::AccessDenied { path } => *path = name,
            _ => {}
        }
        mapped
    };
    if let Err(e) = fs.write_tmp(&tmp, bytes) {
        let _ = fs.remove(&tmp);
        return Err(step(&e, "temporary write"));
    }
    if let Err(e) = fs.flush(&tmp) {
        let _ = fs.remove(&tmp);
        return Err(step(&e, "flush"));
    }
    if let Err(e) = fs.replace(&tmp, target) {
        let _ = fs.remove(&tmp);
        return Err(step(&e, "replace"));
    }
    Ok(())
}

/// `<target>.tmp` in the same folder (§3.9.1).
fn tmp_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    target
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvs_core::error::ErrorCode;
    use std::fs;

    /// Fails one delegated step; all other steps hit the real filesystem.
    struct InjectedFs {
        fail_write: Option<io::ErrorKind>,
        fail_flush: bool,
        fail_replace: bool,
    }

    impl Fs for InjectedFs {
        fn write_tmp(&self, tmp: &Path, bytes: &[u8]) -> io::Result<()> {
            match self.fail_write {
                Some(kind) => Err(io::Error::from(kind)),
                None => StdFs.write_tmp(tmp, bytes),
            }
        }
        fn flush(&self, tmp: &Path) -> io::Result<()> {
            if self.fail_flush {
                Err(io::Error::other("injected flush failure"))
            } else {
                StdFs.flush(tmp)
            }
        }
        fn replace(&self, tmp: &Path, target: &Path) -> io::Result<()> {
            if self.fail_replace {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected replace failure",
                ))
            } else {
                StdFs.replace(tmp, target)
            }
        }
        fn remove(&self, tmp: &Path) -> io::Result<()> {
            StdFs.remove(tmp)
        }
    }

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alvs-atomic-{tag}-{}",
            uuid::Uuid::now_v7().simple()
        ));
        fs::create_dir_all(&dir).unwrap_or(());
        dir
    }

    fn content(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn writes_and_overwrites_atomically_leaving_no_tmp() {
        let dir = scratch("happy");
        let target = dir.join("project.alvs");
        atomic_write(&target, b"v1").unwrap_or(());
        assert_eq!(content(&target), "v1");
        atomic_write(&target, b"v2").unwrap_or(());
        assert_eq!(content(&target), "v2");
        assert!(!dir.join("project.alvs.tmp").exists());
        fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn empty_payload_writes_an_empty_file() {
        let dir = scratch("empty");
        let target = dir.join("empty.alvs");
        atomic_write(&target, b"").unwrap_or(());
        match fs::metadata(&target) {
            Ok(m) => assert_eq!(m.len(), 0),
            Err(e) => panic!("metadata: {e}"),
        }
        fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn injected_failures_at_every_step_keep_the_previous_file() {
        let dir = scratch("inject");
        let target = dir.join("keep.alvs");
        atomic_write(&target, b"original").unwrap_or(());

        let cases: Vec<InjectedFs> = vec![
            InjectedFs {
                fail_write: Some(io::ErrorKind::StorageFull),
                fail_flush: false,
                fail_replace: false,
            },
            InjectedFs {
                fail_write: None,
                fail_flush: true,
                fail_replace: false,
            },
            InjectedFs {
                fail_write: None,
                fail_flush: false,
                fail_replace: true,
            },
        ];
        let expected = [
            ErrorCode::E8001, // disk full at the tmp write (D12 mapping)
            ErrorCode::E1008, // flush failure is a generic save failure
            ErrorCode::E8004, // injected permission failure at replace
        ];
        for (i, fsx) in cases.iter().enumerate() {
            let err = match atomic_write_with(fsx, &target, b"new") {
                Ok(()) => panic!("case {i} must fail"),
                Err(e) => e,
            };
            assert_eq!(err.code(), expected[i], "case {i}");
            assert_eq!(content(&target), "original", "case {i} kept the original");
            assert!(
                !dir.join("keep.alvs.tmp").exists(),
                "case {i} cleaned the tmp"
            );
        }
        fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn tmp_colliding_with_a_directory_is_access_denied_for_real() {
        // Windows: creating a file where a directory exists yields
        // ERROR_ACCESS_DENIED (research D13's real-permission construct).
        let dir = scratch("tmpdir");
        let target = dir.join("t.alvs");
        atomic_write(&target, b"first").unwrap_or(());
        fs::create_dir(dir.join("t.alvs.tmp")).unwrap_or(());
        let err = match atomic_write(&target, b"second") {
            Ok(()) => panic!("must fail"),
            Err(e) => e,
        };
        assert_eq!(err.code(), ErrorCode::E8004);
        assert_eq!(content(&target), "first");
        fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn read_only_target_fails_the_replace_with_access_denied() {
        // Windows: MOVEFILE_REPLACE_EXISTING onto a read-only file yields
        // ERROR_ACCESS_DENIED; the read-only original stays intact.
        let dir = scratch("readonly");
        let target = dir.join("ro.alvs");
        atomic_write(&target, b"locked").unwrap_or(());
        let mut perms = fs::metadata(&target)
            .unwrap_or_else(|e| panic!("meta: {e}"))
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(&target, perms).unwrap_or(());
        let err = match atomic_write(&target, b"next") {
            Ok(()) => panic!("must fail"),
            Err(e) => e,
        };
        assert_eq!(err.code(), ErrorCode::E8004);
        let mut perms = fs::metadata(&target)
            .unwrap_or_else(|e| panic!("meta: {e}"))
            .permissions();
        // Test cleanup on Windows: clear the read-only bit so the scratch
        // dir can be removed (the lint's world-writable concern is Unix-only).
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(&target, perms).unwrap_or(());
        assert_eq!(content(&target), "locked");
        fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn tmp_path_is_sibling_with_tmp_suffix() {
        assert_eq!(
            tmp_path(Path::new(r"C:\proj\demo.alvs")),
            PathBuf::from(r"C:\proj\demo.alvs.tmp")
        );
        assert_eq!(
            tmp_path(Path::new("bare.alvs")),
            PathBuf::from("bare.alvs.tmp")
        );
    }
}
