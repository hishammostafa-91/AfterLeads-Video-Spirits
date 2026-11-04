// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T-REL-003 (09 §9.10): log rotation and redaction — REQ-REL-003 over
//! docs/02 §2.11 and docs/03 §3.9.8. Rotation at 10 MB × 10 files, the
//! canonical JSON fields, level from `alvs.advanced.logLevel`, and user
//! paths redacted (the `alvs-core::redact` helper is the redaction
//! point). The global subscriber is installed once; nextest's per-test
//! processes keep that safe.

use std::io::Write;
use std::path::PathBuf;

use alvs_store::logging::{KEEP_LOG_FILES, MAX_LOG_BYTES, SizeRotatingWriter, init_logging};

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-t-rel-003-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

#[test]
fn t_rel_003_rotation_at_10mb_keeps_ten_files() {
    let dir = scratch("rotate");
    let mut writer = SizeRotatingWriter::new(&dir, "app", MAX_LOG_BYTES, KEEP_LOG_FILES)
        .unwrap_or_else(|e| panic!("new: {e}"));
    // ~96 MB of lines in 4 KB chunks: past nine rotations (the last file
    // sits just under the limit, so nine shifts land `app.log.9`).
    let chunk = format!("{}\n", "x".repeat(4095));
    for _ in 0..24_576u32 {
        writer
            .write_all(chunk.as_bytes())
            .unwrap_or_else(|e| panic!("write: {e}"));
    }
    writer.flush().unwrap_or(());
    let names: Vec<String> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        names.len() <= KEEP_LOG_FILES as usize,
        "at most 10 files exist: {names:?}"
    );
    assert!(names.contains(&"app.log".to_string()));
    assert!(
        names.contains(&"app.log.9".to_string()),
        "nine rotations deep: {names:?}"
    );
    assert!(!names.contains(&"app.log.10".to_string()));
    // No file exceeds the size limit by more than one line.
    for name in &names {
        let len = std::fs::metadata(dir.join(name))
            .map(|m| m.len())
            .unwrap_or(0);
        assert!(
            len <= MAX_LOG_BYTES + 4096,
            "{name} is {len} bytes (limit {MAX_LOG_BYTES} + one line)"
        );
    }
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_rel_003_json_lines_carry_the_canonical_fields() {
    let dir = scratch("fields");
    let logs = dir.join("logs");
    let guard = match init_logging(&logs, "info") {
        Ok(g) => g,
        Err(e) => panic!("init: {e}"),
    };
    tracing::info!(
        job_id = "J-7",
        project_id = "P-1",
        code = "E8003",
        "cache rebuilt"
    );
    let redacted = alvs_core::redact::redact_paths(r"opened C:\Users\me\p.alvs");
    tracing::info!("path {redacted}");
    tracing::trace!("this level is filtered out");
    drop(guard); // flush the non-blocking writer

    let content = std::fs::read_to_string(logs.join("app.log")).unwrap_or_default();
    let mut saw_canonical = false;
    let mut saw_redacted = false;
    for line in content.lines() {
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => panic!("not a JSON line ({e}): {line}"),
        };
        assert!(
            value["ts"].as_str().is_some_and(|t| t.ends_with('Z')),
            "{line}"
        );
        assert!(value["level"].is_string(), "{line}");
        assert!(value["target"].is_string(), "{line}");
        assert!(value["msg"].is_string(), "{line}");
        if value["msg"] == "cache rebuilt" {
            saw_canonical = true;
            assert_eq!(value["job_id"], "J-7", "known fields ride along");
            assert_eq!(value["project_id"], "P-1");
            assert_eq!(value["code"], "E8003");
            assert_eq!(value["level"], "INFO");
            assert!(
                value["target"]
                    .as_str()
                    .is_some_and(|t| t.contains("t_rel_003")),
                "the target names the emitting module"
            );
        }
        if value["msg"].as_str().is_some_and(|m| m.contains("path")) {
            saw_redacted = true;
            assert!(
                !content.contains(r"Users\me"),
                "the raw user path must never appear"
            );
        }
    }
    assert!(saw_canonical, "the structured event was written");
    assert!(saw_redacted, "the redacted-path line was written");
    assert!(!content.contains("filtered out"), "trace is below info");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}

#[test]
fn t_rel_003_level_debug_emits_debug_lines() {
    let dir = scratch("debug");
    let logs = dir.join("logs");
    let guard = match init_logging(&logs, "debug") {
        Ok(g) => g,
        Err(e) => panic!("init: {e}"),
    };
    tracing::debug!("detail line");
    drop(guard);
    let content = std::fs::read_to_string(logs.join("app.log")).unwrap_or_default();
    assert!(content.contains("detail line"), "debug is emitted at debug");
    std::fs::remove_dir_all(&dir).unwrap_or(());
}
