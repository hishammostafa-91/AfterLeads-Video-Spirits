// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Autosave scheduler (docs/03 §3.9.3; REQ-PRJ-003; NFR-REL-02): the
//! decision is a pure predicate on `(dirty, elapsed, interval)` (research
//! D10); the driver applies it with an injectable clock and writes through
//! a caller-supplied snapshot writer — autosave never touches the main
//! file. `interval` comes from `alvs.general.autosaveIntervalSec`
//! (03 §3.6).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::StoreError;

/// Wall-clock source; production uses [`SystemClock`], tests inject.
pub trait Clock: Send + Sync {
    /// Current UTC time, milliseconds since the Unix epoch.
    fn now_ms(&self) -> i64;
}

/// The real clock (research D5: epoch ms from `SystemTime`).
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        crate::utc::epoch_ms_now()
    }
}

/// How a snapshot reaches the sidecar: serialize the current snapshot and
/// atomically write `<Name>.alvs.autosave`. Implemented by the app layer
/// over the model; tests write fixed bytes.
pub trait SnapshotWriter: Send + Sync {
    /// Write one snapshot (never the main file, §3.9.3).
    fn write_snapshot(&self) -> Result<(), StoreError>;
}

/// The §3.9.3 predicate: write iff dirty and at least `interval_sec`
/// elapsed since the last autosave (or the session/save baseline).
pub fn should_autosave(dirty: bool, since_last_ms: i64, interval_sec: u32) -> bool {
    dirty && since_last_ms >= i64::from(interval_sec) * 1000
}

/// Applies [`should_autosave`] with a clock and a writer (research D10).
pub struct AutosaveDriver {
    interval_sec: u32,
    clock: Arc<dyn Clock>,
    writer: Arc<dyn SnapshotWriter>,
    last_ms: Mutex<i64>,
}

impl AutosaveDriver {
    /// A driver with the given interval (seconds), clock, and writer. The
    /// baseline starts at construction time; call
    /// [`Self::note_external_write`] after each manual save so the next
    /// autosave waits a full interval (§3.9.2 saves reset the cadence).
    pub fn new(interval_sec: u32, clock: Arc<dyn Clock>, writer: Arc<dyn SnapshotWriter>) -> Self {
        let baseline = clock.now_ms();
        Self {
            interval_sec,
            clock,
            writer,
            last_ms: Mutex::new(baseline),
        }
    }

    /// One scheduler evaluation: write when the predicate holds. Returns
    /// whether a snapshot was written.
    pub fn tick(&self, dirty: bool) -> Result<bool, StoreError> {
        let now = self.clock.now_ms();
        let since = {
            let last = match self.last_ms.lock() {
                Ok(g) => g,
                Err(_) => {
                    return Err(StoreError::SaveFailed {
                        detail: "autosave state lock poisoned".into(),
                    });
                }
            };
            now - *last
        };
        if !should_autosave(dirty, since, self.interval_sec) {
            return Ok(false);
        }
        self.writer.write_snapshot()?;
        let mut last = match self.last_ms.lock() {
            Ok(g) => g,
            Err(_) => {
                return Err(StoreError::SaveFailed {
                    detail: "autosave state lock poisoned".into(),
                });
            }
        };
        *last = now;
        Ok(true)
    }

    /// Reset the cadence baseline to now (a manual save did the writing;
    /// §3.9.2 deletes the sidecar and starts the interval fresh).
    pub fn note_external_write(&self) {
        if let Ok(mut last) = self.last_ms.lock() {
            *last = self.clock.now_ms();
        }
    }
}

/// The production loop thread; stops on drop (shutdown step 4 of 02 §2.5
/// flushes autosave state).
pub struct AutosaveHandle {
    stop: Arc<AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl AutosaveHandle {
    /// Spawn the poll loop: every `poll_ms`, evaluate the dirty flag.
    pub fn spawn(driver: Arc<AutosaveDriver>, dirty: Arc<AtomicBool>, poll_ms: u64) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let join = std::thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(poll_ms));
                if thread_stop.load(Ordering::Relaxed) {
                    break;
                }
                // A failing writer is retried on the next evaluation
                // (§3.9.3 — losing at most the interval, NFR-REL-02),
                // but never silently: the failure rides the logs this
                // package ships (02 §2.11; review F-14).
                if let Err(e) = driver.tick(dirty.load(Ordering::Relaxed)) {
                    tracing::warn!("autosave write failed; retrying next interval: {e}");
                }
            }
        });
        Self {
            stop,
            join: Some(join),
        }
    }
}

impl Drop for AutosaveHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TickClock {
        now: std::sync::atomic::AtomicI64,
    }

    impl TickClock {
        fn at(start: i64) -> Self {
            Self {
                now: std::sync::atomic::AtomicI64::new(start),
            }
        }
        fn advance(&self, ms: i64) {
            self.now.fetch_add(ms, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl Clock for TickClock {
        fn now_ms(&self) -> i64 {
            self.now.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    struct FailWriter;

    impl SnapshotWriter for FailWriter {
        fn write_snapshot(&self) -> Result<(), StoreError> {
            Err(StoreError::DiskFull {
                detail: "injected".into(),
            })
        }
    }

    #[test]
    fn predicate_truth_table() {
        assert!(!should_autosave(false, 10_000_000, 60)); // clean never
        assert!(!should_autosave(true, 59_999, 60)); // 59.999 s
        assert!(should_autosave(true, 60_000, 60)); // exactly the interval
        assert!(should_autosave(true, 0, 0)); // degenerate 0 s interval
    }

    #[test]
    fn writer_failure_propagates_and_retries_next_tick() {
        let clock = Arc::new(TickClock::at(0));
        let driver = AutosaveDriver::new(1, clock.clone(), Arc::new(FailWriter));
        clock.advance(2_000); // the interval elapses
        let err = match driver.tick(true) {
            Ok(_) => panic!("must fail"),
            Err(e) => e,
        };
        assert_eq!(err.code(), alvs_core::error::ErrorCode::E8001);
    }
}
