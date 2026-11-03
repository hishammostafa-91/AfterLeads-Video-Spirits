//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Path policy SEC-04 (docs/05 §5.7): canonicalize; reject Windows device
//! paths and reserved names; never write outside allowed roots; UNC paths
//! from a project file require explicit confirmation before access.

use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Outcome of a path check (SEC-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathDecision {
    /// The path is inside an allowed root and safe to use.
    Allowed,
    /// The path is a UNC path that came from a project file; the user must
    /// confirm before access (SEC-04, THR-13).
    NeedsConfirmation {
        /// The UNC path awaiting confirmation.
        path: PathBuf,
    },
    /// The path violates the policy; the carried error explains which rule.
    Denied(AppError),
}

/// Where a path came from — project files get stricter treatment (SEC-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathOrigin {
    /// The path was read from a project/library file (untrusted, THR-02).
    ProjectFile,
    /// The path came from a user-picked dialog (trusted choice).
    UserPicked,
}

/// Windows reserved base names (SEC-04): with or without any extension.
const RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Check `candidate` per SEC-04: device-path and reserved-name rejection
/// first, then canonicalization, UNC confirmation for project-file origins,
/// and containment within `roots`.
pub fn check_path(candidate: &Path, origin: PathOrigin, roots: &[PathBuf]) -> PathDecision {
    let raw = candidate.to_string_lossy();

    // Device paths are rejected outright (SEC-04).
    if raw.starts_with(r"\\.\") || raw.starts_with(r"\\?\GLOBALROOT") {
        return PathDecision::Denied(AppError::PathRejected {
            detail: format!("device path rejected: {}", redact(&raw)),
        });
    }

    // Reserved Windows names apply to the final component, with or without an
    // extension (CON, CON.txt, … are all reserved).
    if let Some(name) = candidate
        .file_name()
        .map(|s| s.to_string_lossy().to_uppercase())
    {
        let stem = name.split('.').next().unwrap_or(&name);
        if RESERVED_NAMES.contains(&stem) {
            return PathDecision::Denied(AppError::PathRejected {
                detail: format!("reserved name rejected: {}", redact(&raw)),
            });
        }
    }

    // UNC paths from a project file require explicit confirmation (SEC-04,
    // THR-13: Windows would leak NTLM credentials to the SMB server).
    let is_unc = raw.starts_with(r"\\") && !raw.starts_with(r"\\?\");
    if is_unc && origin == PathOrigin::ProjectFile {
        return PathDecision::NeedsConfirmation {
            path: candidate.to_path_buf(),
        };
    }

    // Canonicalize; a path that does not exist cannot be verified as contained.
    let Ok(canonical) = std::fs::canonicalize(candidate) else {
        return PathDecision::Denied(AppError::PathRejected {
            detail: format!("path cannot be canonicalized: {}", redact(&raw)),
        });
    };

    let canonical_roots: Vec<PathBuf> = roots
        .iter()
        .filter_map(|r| std::fs::canonicalize(r).ok())
        .collect();
    if canonical_roots.is_empty() {
        return PathDecision::Denied(AppError::PathRejected {
            detail: "no accessible root to contain the path".into(),
        });
    }
    if canonical_roots
        .iter()
        .any(|root| canonical.starts_with(root))
    {
        return PathDecision::Allowed;
    }
    PathDecision::Denied(AppError::PathRejected {
        detail: format!("path outside allowed roots: {}", redact(&raw)),
    })
}

/// Paths appear redacted in detail messages (04 §4.9: never full user paths).
fn redact(raw: &str) -> String {
    let tail = raw.rsplit(['\\', '/']).next().unwrap_or("?");
    if tail.is_empty() {
        "…".into()
    } else {
        format!("…{tail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_device_paths() {
        let d = check_path(
            Path::new(r"\\.\PhysicalDrive0"),
            PathOrigin::ProjectFile,
            &[],
        );
        assert!(matches!(d, PathDecision::Denied(_)));
        let d = check_path(
            Path::new(r"\\?\GLOBALROOT\Device\Harddisk0"),
            PathOrigin::ProjectFile,
            &[],
        );
        assert!(matches!(d, PathDecision::Denied(_)));
    }

    #[test]
    fn rejects_reserved_names_with_and_without_extension() {
        for name in ["CON", "con.txt", "AUX.ini", "NUL", "com7.log", "LPT3"] {
            let d = check_path(
                Path::new(r"C:\temp").join(name).as_path(),
                PathOrigin::UserPicked,
                &[],
            );
            assert!(
                matches!(d, PathDecision::Denied(_)),
                "{name} must be denied"
            );
        }
    }

    #[test]
    fn unc_from_project_file_needs_confirmation_but_user_picked_is_checked() {
        let unc = Path::new(r"\\server\share\clip.mp4");
        assert!(matches!(
            check_path(unc, PathOrigin::ProjectFile, &[]),
            PathDecision::NeedsConfirmation { .. }
        ));
        // A user-picked UNC is not auto-allowed; it still has to canonicalize
        // and be contained, which a nonexistent share is not.
        assert!(matches!(
            check_path(unc, PathOrigin::UserPicked, &[]),
            PathDecision::Denied(_)
        ));
    }

    #[test]
    fn containment_boundary_is_exact() {
        let base = std::env::temp_dir().join("alvs-path-policy-test");
        fs::create_dir_all(&base).unwrap_or(());
        let inside = base.join("project.alvs");
        fs::write(&inside, b"{}").unwrap_or(());
        let sibling_of_base = std::env::temp_dir().join("alvs-path-policy-test-other");
        fs::create_dir_all(&sibling_of_base).unwrap_or(());

        assert!(matches!(
            check_path(
                &inside,
                PathOrigin::ProjectFile,
                std::slice::from_ref(&base)
            ),
            PathDecision::Allowed
        ));
        // Prefix similarity must not count as containment: "test-other" is a
        // different directory even though it shares a string prefix.
        let outside = sibling_of_base.join("x.alvs");
        fs::write(&outside, b"{}").unwrap_or(());
        assert!(matches!(
            check_path(
                &outside,
                PathOrigin::ProjectFile,
                std::slice::from_ref(&base)
            ),
            PathDecision::Denied(_)
        ));
        fs::remove_dir_all(&base).unwrap_or(());
        fs::remove_dir_all(&sibling_of_base).unwrap_or(());
    }

    #[test]
    fn nonexistent_path_is_denied_not_allowed() {
        let d = check_path(
            Path::new(r"C:\definitely\missing\file.alvs"),
            PathOrigin::ProjectFile,
            &[PathBuf::from(r"C:\definitely")],
        );
        assert!(matches!(d, PathDecision::Denied(_)));
    }
}
