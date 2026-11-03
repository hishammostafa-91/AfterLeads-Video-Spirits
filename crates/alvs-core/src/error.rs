//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Error model: per-crate `thiserror` enums mapping every variant to exactly
//! one catalog code (docs/12 §12.13) and the UI error envelope (docs/04 §4.9).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Severity of an error envelope (docs/04 §4.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Toast, auto-dismiss.
    Info,
    /// Toast, persistent in the notification center.
    Warning,
    /// Toast with details; modal only when data could be lost.
    Error,
}

/// Catalog error codes shared by the crates (docs/12 §12.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// E0001 — an unexpected internal error occurred.
    E0001,
    /// E0002 — an operation was rolled back because it would corrupt the project.
    E0002,
    /// E1003 — the project file failed validation.
    E1003,
    /// E1004 — newer application version; opens read-only.
    E1004,
    /// E1005 — the project could not be upgraded; the original is unchanged.
    E1005,
    /// E1006 — a sequence cannot contain itself.
    E1006,
    /// E1007 — the project is open in another window or computer (03 §3.9.4).
    E1007,
    /// E1008 — the project could not be saved (03 §3.9.1).
    E1008,
    /// E1102 — that edit is not possible here (locked track or overlap).
    E1102,
    /// E1104 — some items were moved to match the new frame rate.
    E1104,
    /// E2001 — media is offline.
    E2001,
    /// E5003 — writing the export failed (disk full or removed); WP-22's half
    /// of REQ-REL-007 (12 §12.13).
    E5003,
    /// E8001 — the disk is full (03 §3.9.1; REQ-REL-007).
    E8001,
    /// E8002 — settings were damaged and have been restored (03 §3.6).
    E8002,
    /// E8003 — the cache was damaged and has been rebuilt (03 §3.7).
    E8003,
    /// E8004 — access to a path was denied (03 §3.9.1; REQ-REL-007).
    E8004,
    /// E9005 — a file path was rejected for security reasons (05 §5.7).
    E9005,
}

impl ErrorCode {
    /// The catalog code string (docs/12 §12.13).
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::E0001 => "E0001",
            ErrorCode::E0002 => "E0002",
            ErrorCode::E1003 => "E1003",
            ErrorCode::E1004 => "E1004",
            ErrorCode::E1005 => "E1005",
            ErrorCode::E1006 => "E1006",
            ErrorCode::E1007 => "E1007",
            ErrorCode::E1008 => "E1008",
            ErrorCode::E1102 => "E1102",
            ErrorCode::E1104 => "E1104",
            ErrorCode::E2001 => "E2001",
            ErrorCode::E5003 => "E5003",
            ErrorCode::E8001 => "E8001",
            ErrorCode::E8002 => "E8002",
            ErrorCode::E8003 => "E8003",
            ErrorCode::E8004 => "E8004",
            ErrorCode::E9005 => "E9005",
        }
    }

    /// Severity per the catalog (docs/12 §12.13).
    pub fn severity(self) -> Severity {
        match self {
            ErrorCode::E0001
            | ErrorCode::E0002
            | ErrorCode::E1003
            | ErrorCode::E1005
            | ErrorCode::E1008
            | ErrorCode::E5003
            | ErrorCode::E8001
            | ErrorCode::E8004 => Severity::Error,
            ErrorCode::E9005 => Severity::Error,
            ErrorCode::E1004
            | ErrorCode::E1104
            | ErrorCode::E2001
            | ErrorCode::E1007
            | ErrorCode::E8002
            | ErrorCode::E8003 => Severity::Warning,
            ErrorCode::E1006 => Severity::Warning,
            ErrorCode::E1102 => Severity::Info,
        }
    }

    /// The localization key is always `error.<code>` (docs/04 §4.9).
    pub fn message_key(self) -> String {
        format!("error.{}", self.as_str())
    }
}

/// `alvs-core` error type: every variant maps to exactly one code
/// (docs/02 §2.11).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    /// Unexpected internal failure (E0001).
    #[error("internal error: {detail}")]
    Internal {
        /// Diagnostic detail for logs; never contains secrets (04 §4.9).
        detail: String,
    },
    /// An edit was rolled back to protect the project (E0002, 03 §3.5).
    #[error("operation rolled back: {detail}")]
    RolledBack {
        /// Which invariant or rule was violated.
        detail: String,
    },
    /// Project data failed validation (E1003).
    #[error("validation failed: {detail}")]
    ValidationFailed {
        /// Summary of the validation issues.
        detail: String,
    },
    /// The file is from a newer application version (E1004, 03 §3.9.6).
    #[error("newer schema version: {found}")]
    ReadOnlyNewer {
        /// The schema version found in the file.
        found: u32,
    },
    /// A migration step failed (E1005, 03 §3.9.6).
    #[error("migration failed: {detail}")]
    MigrationFailed {
        /// Which step failed and why.
        detail: String,
    },
    /// Nested sequences would form a cycle (E1006).
    #[error("sequence nesting cycle: {detail}")]
    NestingCycle {
        /// The cycle path description.
        detail: String,
    },
    /// The edit is not possible at this position (E1102).
    #[error("edit blocked: {detail}")]
    EditBlocked {
        /// Why the edit is not possible.
        detail: String,
    },
    /// Items were adjusted to the new frame grid (E1104, REQ-PRJ-005).
    #[error("items adjusted to the new frame rate: {count}")]
    FrameRateAdjusted {
        /// Number of items moved by the snap.
        count: usize,
    },
    /// A referenced file is offline (E2001).
    #[error("media offline: {detail}")]
    MediaOffline {
        /// Which item is offline (path redacted per 04 §4.9).
        detail: String,
    },
    /// A file path was rejected by the SEC-04 policy (E9005, 05 §5.7).
    #[error("path rejected: {detail}")]
    PathRejected {
        /// Which SEC-04 rule rejected the path.
        detail: String,
    },
}

impl AppError {
    /// The single catalog code for this variant (docs/02 §2.11).
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::Internal { .. } => ErrorCode::E0001,
            AppError::RolledBack { .. } => ErrorCode::E0002,
            AppError::ValidationFailed { .. } => ErrorCode::E1003,
            AppError::ReadOnlyNewer { .. } => ErrorCode::E1004,
            AppError::MigrationFailed { .. } => ErrorCode::E1005,
            AppError::NestingCycle { .. } => ErrorCode::E1006,
            AppError::EditBlocked { .. } => ErrorCode::E1102,
            AppError::FrameRateAdjusted { .. } => ErrorCode::E1104,
            AppError::MediaOffline { .. } => ErrorCode::E2001,
            AppError::PathRejected { .. } => ErrorCode::E9005,
        }
    }
}

/// A recovery action offered in the UI (docs/04 §4.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// A command ID from docs/12 §12.10 or a dialog action.
    #[serde(rename = "commandId")]
    pub command_id: String,
    /// Localization key for the action label.
    #[serde(rename = "labelKey")]
    pub label_key: String,
}

/// The error envelope crossing to the UI (docs/04 §4.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    /// Catalog code.
    pub code: ErrorCode,
    /// Severity of the presentation.
    pub severity: Severity,
    /// Always `error.<code>`.
    #[serde(rename = "messageKey")]
    pub message_key: String,
    /// Parameters for the localized message.
    pub params: std::collections::BTreeMap<String, String>,
    /// Log/diagnostics detail; shown under "Details", never secrets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Whether the operation can be retried.
    pub retryable: bool,
    /// Recovery actions offered in the UI.
    #[serde(default)]
    pub recovery: Vec<RecoveryAction>,
}

impl ErrorEnvelope {
    /// Build the envelope for an error, deriving code, severity, and key.
    pub fn from_error(err: &AppError) -> Self {
        let code = err.code();
        Self {
            code,
            severity: code.severity(),
            message_key: code.message_key(),
            params: BTreeMap::new(),
            detail: Some(err.to_string()),
            retryable: false,
            recovery: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_maps_to_exactly_one_code() {
        let errors = [
            AppError::Internal { detail: "x".into() },
            AppError::RolledBack { detail: "x".into() },
            AppError::ValidationFailed { detail: "x".into() },
            AppError::ReadOnlyNewer { found: 2 },
            AppError::MigrationFailed { detail: "x".into() },
            AppError::NestingCycle { detail: "x".into() },
            AppError::EditBlocked { detail: "x".into() },
            AppError::FrameRateAdjusted { count: 3 },
            AppError::MediaOffline { detail: "x".into() },
            AppError::PathRejected { detail: "x".into() },
        ];
        let mut codes: Vec<ErrorCode> = errors.iter().map(|e| e.code()).collect();
        codes.sort_by_key(|c| c.as_str());
        codes.dedup();
        // All ten variants map to ten distinct codes (02 §2.11).
        assert_eq!(codes.len(), errors.len());
        assert_eq!(errors[0].code().as_str(), "E0001");
        assert_eq!(errors[7].code().as_str(), "E1104");
    }

    #[test]
    fn severities_match_the_catalog() {
        // E1102 is info (auto-dismiss) and E9005 is error (12 §12.13).
        assert_eq!(ErrorCode::E1102.severity(), Severity::Info);
        assert_eq!(ErrorCode::E9005.severity(), Severity::Error);
        assert_eq!(ErrorCode::E9005.as_str(), "E9005");
        assert_eq!(ErrorCode::E1006.severity(), Severity::Warning);
        // The seven WP-03 codes (12 §12.13 rows; research D4).
        assert_eq!(ErrorCode::E1007.severity(), Severity::Warning);
        assert_eq!(ErrorCode::E1008.severity(), Severity::Error);
        assert_eq!(ErrorCode::E5003.severity(), Severity::Error);
        assert_eq!(ErrorCode::E8001.severity(), Severity::Error);
        assert_eq!(ErrorCode::E8002.severity(), Severity::Warning);
        assert_eq!(ErrorCode::E8003.severity(), Severity::Warning);
        assert_eq!(ErrorCode::E8004.severity(), Severity::Error);
        assert_eq!(ErrorCode::E1007.as_str(), "E1007");
        assert_eq!(ErrorCode::E8004.as_str(), "E8004");
        assert_eq!(ErrorCode::E1007.message_key(), "error.E1007");
    }

    #[test]
    fn envelope_json_shape_matches_04_4_9() {
        let envelope = ErrorEnvelope {
            code: ErrorCode::E2001,
            severity: Severity::Warning,
            message_key: "error.E2001".into(),
            params: BTreeMap::from([("name".into(), "interview.mp4".into())]),
            detail: Some("path redacted".into()),
            retryable: false,
            recovery: vec![RecoveryAction {
                command_id: "media.relink".into(),
                label_key: "action.relink".into(),
            }],
        };
        let json = serde_json::to_value(&envelope).unwrap_or_default();
        assert_eq!(json["code"], "E2001");
        assert_eq!(json["severity"], "warning");
        assert_eq!(json["messageKey"], "error.E2001");
        assert_eq!(json["params"]["name"], "interview.mp4");
        assert_eq!(json["retryable"], false);
        assert_eq!(json["recovery"][0]["commandId"], "media.relink");
        assert_eq!(json["recovery"][0]["labelKey"], "action.relink");
    }

    #[test]
    fn envelope_from_error_derives_catalog_fields() {
        let err = AppError::ReadOnlyNewer { found: 3 };
        let envelope = ErrorEnvelope::from_error(&err);
        assert_eq!(envelope.code, ErrorCode::E1004);
        assert_eq!(envelope.severity, Severity::Warning);
        assert_eq!(envelope.message_key, "error.E1004");
    }
}
