// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Logging setup (docs/02 §2.11; docs/03 §3.8/§3.9.8; REQ-REL-003):
//! JSON lines with the canonical fields (`ts`, `level`, `target`, `msg`,
//! plus `job_id`/`project_id`/`code` when known), a custom size-rotating
//! writer (10 MB × 10 files — `tracing-appender` rotates by schedule
//! only, research D3), and the non-blocking write path with a guard that
//! flushes on shutdown. File naming `app.log` + `app.log.1…9` is this
//! package's convention (spec Assumptions).

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tracing::field::Visit;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer as FormatWriter;
use tracing_subscriber::fmt::time::{FormatTime, UtcTime};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};

use crate::error::StoreError;

/// The rotation contract of REQ-REL-003 (03 §3.9.8): 10 MB per file, 10
/// files kept.
pub const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
/// How many log files exist at most (`app.log` + `.1`…`.9`).
pub const KEEP_LOG_FILES: u32 = 10;

/// A size-rotating JSON-lines writer (research D3): writes to
/// `<dir>/<prefix>.log`; when a write would pass `max_bytes`, the files
/// shift (`app.log.8` → `app.log.9`, …, `app.log` → `app.log.1`) and a
/// fresh `app.log` starts; at most `keep` files exist.
pub struct SizeRotatingWriter {
    dir: PathBuf,
    prefix: String,
    max_bytes: u64,
    keep: u32,
    inner: Option<File>,
    current_size: u64,
}

impl SizeRotatingWriter {
    /// A writer for `<dir>/<prefix>.log` rotating at `max_bytes` keeping
    /// `keep` files.
    pub fn new(dir: &Path, prefix: &str, max_bytes: u64, keep: u32) -> Result<Self, StoreError> {
        std::fs::create_dir_all(dir).map_err(|e| StoreError::from_io(&e))?;
        let mut writer = Self {
            dir: dir.to_path_buf(),
            prefix: prefix.to_string(),
            max_bytes,
            keep: keep.max(1),
            inner: None,
            current_size: 0,
        };
        writer.open_current()?;
        Ok(writer)
    }

    fn log_path(&self) -> PathBuf {
        self.dir.join(format!("{}.log", self.prefix))
    }

    fn numbered_path(&self, n: u32) -> PathBuf {
        self.dir.join(format!("{}.log.{n}", self.prefix))
    }

    fn open_current(&mut self) -> Result<(), StoreError> {
        let path = self.log_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| StoreError::from_io(&e))?;
        self.current_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        self.inner = Some(file);
        Ok(())
    }

    /// Shift the numbered files up and start a fresh `app.log`. `keep`
    /// counts `app.log` plus `.1`…`.keep−1` (03 §3.9.8).
    fn rotate(&mut self) -> Result<(), StoreError> {
        self.inner = None; // close the current file first
        for n in (1..self.keep - 1).rev() {
            let from = self.numbered_path(n);
            let to = self.numbered_path(n + 1);
            if from.exists() {
                if to.exists() {
                    let _ = std::fs::remove_file(&to);
                }
                std::fs::rename(&from, &to).map_err(|e| StoreError::from_io(&e))?;
            }
        }
        let first = self.numbered_path(1);
        let _ = std::fs::remove_file(&first);
        std::fs::rename(self.log_path(), &first).map_err(|e| StoreError::from_io(&e))?;
        self.open_current()
    }
}

impl Write for SizeRotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.current_size + buf.len() as u64 > self.max_bytes {
            // A single line larger than the limit gets its own file.
            let _ = self.rotate();
        }
        let file = match self.inner.as_mut() {
            Some(f) => f,
            None => {
                return Err(io::Error::other("log file is not open"));
            }
        };
        let written = file.write(buf)?;
        self.current_size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner.as_mut() {
            Some(f) => f.flush(),
            None => Ok(()),
        }
    }
}

/// The docs/02 §2.11 canonical JSON line: `ts`, `level`, `target`, `msg`,
/// plus the event's other fields at the top level (`job_id`,
/// `project_id`, `code`, … when known). The default tracing-subscriber
/// JSON format nests the message under `fields`, so this formatter writes
/// the canonical shape directly.
struct CanonicalJson;

impl CanonicalJson {
    fn new() -> Self {
        Self
    }
}

struct FieldCollector {
    msg: Option<String>,
    extras: serde_json::Map<String, serde_json::Value>,
}

impl Visit for FieldCollector {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.msg = Some(value.to_string());
        } else {
            self.extras.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        if field.name() == "message" {
            self.msg = Some(rendered);
        } else {
            self.extras.insert(
                field.name().to_string(),
                serde_json::Value::String(rendered),
            );
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.extras
            .insert(field.name().to_string(), serde_json::Value::from(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.extras
            .insert(field.name().to_string(), serde_json::Value::from(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.extras
            .insert(field.name().to_string(), serde_json::Value::from(value));
    }
}

impl<S, N> FormatEvent<S, N> for CanonicalJson
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: FormatWriter<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let mut ts = String::new();
        {
            // `UtcTime::rfc_3339()` needs no named `time` type here; the
            // timer is constructed and used inline.
            let timer = UtcTime::rfc_3339();
            let mut time_writer = FormatWriter::new(&mut ts);
            if timer.format_time(&mut time_writer).is_err() {
                ts.push_str("unavailable");
            }
        }
        let mut collector = FieldCollector {
            msg: None,
            extras: serde_json::Map::new(),
        };
        event.record(&mut collector);
        let mut object = serde_json::Map::new();
        object.insert("ts".into(), serde_json::Value::String(ts));
        object.insert(
            "level".into(),
            serde_json::Value::String(event.metadata().level().to_string()),
        );
        object.insert(
            "target".into(),
            serde_json::Value::String(event.metadata().target().to_string()),
        );
        object.insert(
            "msg".into(),
            serde_json::Value::String(collector.msg.unwrap_or_default()),
        );
        for (key, value) in collector.extras {
            object.insert(key, value);
        }
        // The format Writer implements fmt::Write: serialize to a string
        // first, then one line per event.
        let line = serde_json::to_string(&serde_json::Value::Object(object))
            .map_err(|_| std::fmt::Error)?;
        writer.write_str(&line).map_err(|_| std::fmt::Error)?;
        writer.write_str("\n").map_err(|_| std::fmt::Error)
    }
}

/// The logging guard: dropping it flushes the non-blocking writer
/// (02 §2.5 shutdown step 6).
pub struct LogGuard {
    _worker: tracing_appender::non_blocking::WorkerGuard,
}

/// Build the JSON-lines subscriber with rotation and the level from
/// `alvs.advanced.logLevel` (03 §3.6: `info` | `debug` | `trace`), plus
/// the guard that flushes the non-blocking writer. Composable: tests
/// drive it with [`tracing::subscriber::with_default`] without touching
/// the global default.
pub fn subscriber_for(
    logs_dir: &Path,
    level: &str,
) -> Result<
    (
        tracing_appender::non_blocking::WorkerGuard,
        impl tracing::Subscriber,
    ),
    StoreError,
> {
    let filter = match level {
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };
    let writer = SizeRotatingWriter::new(logs_dir, "app", MAX_LOG_BYTES, KEEP_LOG_FILES)?;
    let (non_blocking, worker) = tracing_appender::non_blocking(writer);
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(filter)
        .event_format(CanonicalJson::new())
        .with_writer(non_blocking)
        .finish();
    Ok((worker, subscriber))
}

/// Install the global JSON-lines subscriber (one call per process; 02
/// §2.5 startup). See [`subscriber_for`] for the composable form.
pub fn init_logging(logs_dir: &Path, level: &str) -> Result<LogGuard, StoreError> {
    let (worker, subscriber) = subscriber_for(logs_dir, level)?;
    tracing::subscriber::set_global_default(subscriber).map_err(|e| StoreError::SaveFailed {
        detail: format!("cannot install the log subscriber: {e}"),
    })?;
    Ok(LogGuard { _worker: worker })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alvs-logging-{tag}-{}",
            uuid::Uuid::now_v7().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap_or(());
        dir
    }

    #[test]
    fn rotation_keeps_only_the_newest_files() {
        let dir = scratch("rotate");
        let mut writer =
            SizeRotatingWriter::new(&dir, "app", 120, 4).unwrap_or_else(|e| panic!("new: {e}"));
        for i in 0..40 {
            let line = format!("line-{i:03} padding padding padding\n");
            writer
                .write_all(line.as_bytes())
                .unwrap_or_else(|e| panic!("write {i}: {e}"));
        }
        writer.flush().unwrap_or(());
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        // app.log + .1 .2 .3 (keep 4) — no more.
        assert_eq!(names.len(), 4, "{names:?}");
        assert!(names.contains(&"app.log".to_string()));
        assert!(names.contains(&"app.log.3".to_string()));
        assert!(!names.contains(&"app.log.4".to_string()));
        // The newest content is in app.log; the oldest slice in .3.
        let current = std::fs::read_to_string(dir.join("app.log")).unwrap_or_default();
        assert!(current.contains("line-039"), "newest line lives in app.log");
        let oldest = std::fs::read_to_string(dir.join("app.log.3")).unwrap_or_default();
        assert!(
            !oldest.contains("line-039"),
            "the oldest file carries earlier lines"
        );
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }

    #[test]
    fn oversized_single_line_gets_its_own_file() {
        let dir = scratch("oversize-line");
        let mut writer =
            SizeRotatingWriter::new(&dir, "app", 10, 3).unwrap_or_else(|e| panic!("new: {e}"));
        let big = "x".repeat(50);
        writer
            .write_all(big.as_bytes())
            .unwrap_or_else(|e| panic!("write: {e}"));
        writer.flush().unwrap_or(());
        let current = std::fs::read(dir.join("app.log")).unwrap_or_default();
        assert_eq!(current.len(), 50, "the big line is kept whole");
        std::fs::remove_dir_all(&dir).unwrap_or(());
    }
}
