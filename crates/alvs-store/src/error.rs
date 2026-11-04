// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Store errors: every variant maps to exactly one catalog code
//! (docs/02 §2.11; docs/12 §12.13) and crosses to the UI as the
//! docs/04 §4.9 envelope.

use alvs_core::error::{ErrorCode, ErrorEnvelope, RecoveryAction};
use std::collections::BTreeMap;

/// `alvs-store` error type (02 §2.11): one variant per catalog code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    /// E1004 — the file is from a newer application version; read-only.
    #[error("newer schema version: {found}")]
    ReadOnlyNewer {
        /// The schema version found in the file.
        found: u32,
    },
    /// E1005 — the project could not be upgraded; the original is unchanged.
    #[error("migration failed: {detail}")]
    MigrationFailed {
        /// Which step failed and why.
        detail: String,
    },
    /// E1003 — the project file failed validation (03 §3.5; the raw size
    /// gate of rule 15 surfaces here on open).
    #[error("validation failed: {detail}")]
    ValidationFailed {
        /// The validation issues (or size-gate reason).
        detail: String,
    },
    /// E1007 — the project is open in another window or computer (03 §3.9.4).
    #[error("locked: {detail}")]
    Locked {
        /// The lock holder and liveness verdict.
        detail: String,
    },
    /// E1008 — the project could not be saved (03 §3.9.1).
    #[error("save failed: {detail}")]
    SaveFailed {
        /// Which write failed and why.
        detail: String,
    },
    /// E8001 — the disk is full (REQ-REL-007).
    #[error("disk full: {detail}")]
    DiskFull {
        /// Which write hit the limit.
        detail: String,
    },
    /// E8002 — settings were damaged and have been restored (03 §3.6).
    #[error("settings restored: {keys}")]
    SettingsRestored {
        /// The invalid keys replaced by defaults (collective notice).
        keys: String,
    },
    /// E8003 — the cache was damaged and has been rebuilt (03 §3.7).
    #[error("cache rebuilt: {detail}")]
    CacheRebuilt {
        /// The corrupt file and rename target.
        detail: String,
    },
    /// E8004 — access to a path was denied (REQ-REL-007).
    #[error("access denied: {path}")]
    AccessDenied {
        /// The denied path (redacted per 04 §4.9 when crossing to the UI).
        path: String,
    },
    /// E5003 — a write failed (disk full or removed); the export-side
    /// wording of REQ-REL-007's second half (WP-22).
    #[error("write failed: {detail}")]
    WriteFailed {
        /// Which write failed and why.
        detail: String,
    },
    /// E9005 — a file path was rejected by the SEC-04 policy (05 §5.7).
    #[error("path rejected: {detail}")]
    PathRejected {
        /// Which SEC-04 rule rejected the path.
        detail: String,
    },
}

impl StoreError {
    /// The single catalog code for this variant (02 §2.11).
    pub fn code(&self) -> ErrorCode {
        match self {
            StoreError::ReadOnlyNewer { .. } => ErrorCode::E1004,
            StoreError::MigrationFailed { .. } => ErrorCode::E1005,
            StoreError::ValidationFailed { .. } => ErrorCode::E1003,
            StoreError::Locked { .. } => ErrorCode::E1007,
            StoreError::SaveFailed { .. } => ErrorCode::E1008,
            StoreError::DiskFull { .. } => ErrorCode::E8001,
            StoreError::SettingsRestored { .. } => ErrorCode::E8002,
            StoreError::CacheRebuilt { .. } => ErrorCode::E8003,
            StoreError::AccessDenied { .. } => ErrorCode::E8004,
            StoreError::WriteFailed { .. } => ErrorCode::E5003,
            StoreError::PathRejected { .. } => ErrorCode::E9005,
        }
    }

    /// The docs/04 §4.9 envelope for crossing to the UI; the message key is
    /// always `error.<code>` and the recovery action uses the catalog row.
    pub fn envelope(self) -> ErrorEnvelope {
        let code = self.code();
        let detail = self.to_string();
        let recovery = match code {
            ErrorCode::E1007 => vec![
                RecoveryAction {
                    command_id: "project.openReadOnly".into(),
                    label_key: "action.openReadOnly".into(),
                },
                RecoveryAction {
                    command_id: "project.takeOverLock".into(),
                    label_key: "action.takeOverLock".into(),
                },
            ],
            ErrorCode::E1008 => vec![RecoveryAction {
                command_id: "project.saveAs".into(),
                label_key: "action.saveAs".into(),
            }],
            ErrorCode::E5003 => vec![RecoveryAction {
                command_id: "retry".into(),
                label_key: "action.retry".into(),
            }],
            ErrorCode::E8001 => vec![RecoveryAction {
                command_id: "disk.freeSpace".into(),
                label_key: "action.freeSpace".into(),
            }],
            ErrorCode::E8004 => vec![RecoveryAction {
                command_id: "chooseLocation".into(),
                label_key: "action.chooseAnotherLocation".into(),
            }],
            _ => Vec::new(),
        };
        ErrorEnvelope {
            code,
            severity: code.severity(),
            message_key: code.message_key(),
            params: BTreeMap::new(),
            detail: Some(detail),
            retryable: matches!(code, ErrorCode::E5003),
            recovery,
        }
    }

    /// Map an I/O failure per docs/03 §3.9.1: full-disk kinds → E8001,
    /// permission → E8004, anything else → E1008. Call sites add context.
    pub fn from_io(err: &std::io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::StorageFull | ErrorKind::WriteZero | ErrorKind::QuotaExceeded => {
                StoreError::DiskFull {
                    detail: err.to_string(),
                }
            }
            ErrorKind::PermissionDenied => StoreError::AccessDenied {
                path: err.to_string(),
            },
            _ => StoreError::SaveFailed {
                detail: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_maps_to_exactly_one_code() {
        let errors = [
            StoreError::ReadOnlyNewer { found: 2 },
            StoreError::MigrationFailed { detail: "x".into() },
            StoreError::ValidationFailed { detail: "x".into() },
            StoreError::Locked { detail: "x".into() },
            StoreError::SaveFailed { detail: "x".into() },
            StoreError::DiskFull { detail: "x".into() },
            StoreError::SettingsRestored { keys: "a,b".into() },
            StoreError::CacheRebuilt { detail: "x".into() },
            StoreError::AccessDenied { path: "p".into() },
            StoreError::WriteFailed { detail: "x".into() },
            StoreError::PathRejected { detail: "x".into() },
        ];
        let mut codes: Vec<ErrorCode> = errors.iter().map(|e| e.code()).collect();
        codes.sort_by_key(|c| c.as_str());
        codes.dedup();
        // All eleven variants map to eleven distinct codes (02 §2.11).
        assert_eq!(codes.len(), errors.len());
        assert_eq!(errors[0].code().as_str(), "E1004");
        assert_eq!(errors[2].code().as_str(), "E1003");
        assert_eq!(errors[3].code().as_str(), "E1007");
        assert_eq!(errors[9].code().as_str(), "E5003");
    }

    #[test]
    fn envelope_carries_catalog_fields_and_recovery() {
        let envelope = StoreError::Locked {
            detail: "held".into(),
        }
        .envelope();
        assert_eq!(envelope.code, ErrorCode::E1007);
        assert_eq!(envelope.message_key, "error.E1007");
        assert!(!envelope.recovery.is_empty());
        let envelope = StoreError::WriteFailed { detail: "d".into() }.envelope();
        assert_eq!(envelope.code, ErrorCode::E5003);
        assert!(envelope.retryable);
        let envelope = StoreError::SettingsRestored { keys: "k".into() }.envelope();
        assert_eq!(envelope.code, ErrorCode::E8002);
        assert!(envelope.recovery.is_empty());
    }

    #[test]
    fn io_kinds_map_to_the_391_codes() {
        let full = std::io::Error::from(std::io::ErrorKind::StorageFull);
        assert_eq!(StoreError::from_io(&full).code(), ErrorCode::E8001);
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(StoreError::from_io(&denied).code(), ErrorCode::E8004);
        let other = std::io::Error::from(std::io::ErrorKind::AlreadyExists);
        assert_eq!(StoreError::from_io(&other).code(), ErrorCode::E1008);
    }
}
