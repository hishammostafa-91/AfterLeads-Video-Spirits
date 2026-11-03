# WP-03 Research — Storage and application services

Method per G-10: repository → lockfile/registry → crates.io/docs.rs (verified 2026-11-03). Zero unresolved clarifications; the single canonical conflict (card vs docs/02 §2.4 CMP-08) was decided by the owner as **Option A** (D2). Every decision cites its source.

## D1 — Dependency set (all baseline rows, 00 §0.8; versions verified 2026-11-03)

| Crate | Version | Baseline row | Evidence |
|---|---|---|---|
| `rusqlite` (feature `bundled`) | 0.41.1 | Data (202) | crates.io max_stable 0.41.1 (2026-10-28); pulls `libsqlite3-sys` (MIT) via `cc` 1.10.0 (already locked) |
| `tracing` | 0.1.44 | Core (204) | already in Cargo.lock (transitive) |
| `tracing-subscriber` | 0.3.23 | Core (204) | crates.io max_stable 0.3.23; features used: `default` + `json` + `time` (D3) |
| `tracing-appender` | 0.2.5 | Core (204) | crates.io max_stable 0.2.5; MIT; new transitive `symlink` 0.1.0 is MIT/Apache-2.0 (both on the deny.toml allowlist); `time` 0.3.55 already transitive in the lock |
| `windows` | 0.62.2 | Core (206) | already in Cargo.lock; features `Win32_Storage_FileSystem` + `Win32_System_Threading` + `Win32_Foundation` exist in the 0.62 line (local registry) |
| `serde` / `serde_json` / `thiserror` / `uuid` | 1.x / 1.0.154 / 2.0.21 / 1.29.0 | Data/Core | locked versions used by WP-02 siblings |
| internal: `alvs-core` | workspace | CMP-08 row | docs/02 §2.4 |
| dev: `alvs-model`, `proptest` | workspace / 1.x | — | dev-dependencies are exempt from CMP row checks (tools/check_deps.py rule 1); proptest is the Tests row (209) |

**Explicitly not used**: `tokio` (D17), `tauri` (rule-2 exclusivity: tauri is CMP-12-only — so folder resolution cannot go through Tauri, D6), `time`/`chrono` as *direct* deps (not baseline rows, G-11; D5), `tempfile`/`dirs` (not baseline; tests use std temp dirs with uuid names).

## D2 — Save/open architecture: the DocumentHooks seam (owner decision, 2026-11-03)

- **Decision**: `alvs-store`'s regular internal dependencies stay exactly `alvs-core` (docs/02 §2.4 CMP-08; enforced by tools/check_deps.py rule 1 and CI). The model-aware halves of the save/open pipeline (serialize `Project`, parse+validate per 03 §3.5, run the WP-02 migration driver, resolve media rel-paths inside the parsed model) are behind one small trait defined in `alvs-store`:
  `DocumentHooks { serialize; parse_and_validate; migrate; fixup_media_paths }`.
  WP-03 proves the whole pipeline in `alvs-store`'s own integration tests via a dev-dependency on `alvs-model` (legal: check_deps exempts dev-deps from row checks; rule 2 untouched). The production implementation for `Project` is a thin composition module (`project_io`) in the app backend `app/src-tauri` (CMP-12 may depend on CMP-01..CMP-10; check_deps: `allowed_internal_app = INTERNAL_ALLOWED − {alvs-testkit}`), ready for WP-05 to call.
- **Rationale**: honors the card deliverable ("project save/open with relative-path fallback" is delivered and tested by WP-03) *and* the canonical dependency row and the WP-01 gate, with no doc change and no gate edit.
- **Soundness checks**: no dependency cycle (`alvs-model → alvs-core` only); `cargo deny` unaffected (no new licenses); app crate already builds in the workspace.
- **Alternatives rejected**: (B) amending docs/02 CMP-08 to add CMP-02 — a canonical layering change plus a gate edit for convenience; (C) moving the whole pipeline into the app crate — leaves `alvs-store` as plain file utilities and moves REQ-PRJ-002's logic away from the card's parenthetical.

## D3 — Logging pipeline: JSON lines, custom size rotation (02 §2.11; 03 §3.6/§3.8/§3.9.8; REQ-REL-003)

- **Decision**: `tracing_subscriber::fmt().json()` writing through **our own size-rotating writer** (rotate at 10 MB; keep `app.log` + `app.log.1` … `app.log.9`; delete older), wrapped in `tracing_appender::non_blocking` (WorkerGuard flushed at shutdown). The `ts` field uses `UtcTime::rfc_3339()` (tracing-subscriber feature `time`). Level from `alvs.advanced.logLevel`. Paths are logged only at debug and always pass through `redact` (D15); media content/caption text never logged.
- **Rationale (the load-bearing finding)**: docs.rs tracing-appender 0.2.5 — `RollingFileAppender` "rotate[s] log files **at a fixed schedule**" (`Rotation::NEVER/MINUTELY/HOURLY/DAILY`); there is **no size-based rotation**. REQ-REL-003 mandates "10 files × 10 MB". The custom writer is therefore required by the canonical numbers, while `tracing-appender` still supplies the prescribed non-blocking write path (02 §2.11 names the crate; we use it for what it actually provides). `time` arrives only as tracing-subscriber's own feature-gated optional dependency (already in the lock at 0.3.55), not a direct dep.
- **Alternatives**: time-based rotation + `max_log_files` — violates the 10 MB half; writing directly (no non_blocking) — blocks callers, loses the guard's flush semantics.

## D4 — Error catalog extension (02 §2.11; 12 §12.13; 04 §4.9)

- **Decision**: `alvs-core::error::ErrorCode` gains the seven catalog codes WP-02 did not yet define — **E1007, E1008, E5003, E8001, E8002, E8003, E8004** — each with its documented severity (E1007 warning, E1008 error, E5003 error, E8001 error, E8002 warning, E8003 warning, E8004 error; rows verified in 12 §12.13). `alvs-store` defines its own `StoreError` (`thiserror`) with exactly one variant per code via `fn code(&self)`, plus `ErrorEnvelope` conversion (04 §4.9). `AppError` (model-side) is untouched.
- **Rationale**: 02 §2.11 "each crate defines thiserror error enums; every variant maps to exactly one code"; the shared `ErrorCode` catalog lives in `alvs-core` (CMP-01) so every crate maps into the same table.

## D5 — Wall-clock timestamps without a new dependency

- **Decision**: instants are epoch milliseconds (`SystemTime`); the civil forms WP-03 needs — backup names `yyyyMMdd-HHmmss` (03 §3.9.2), lock `startedUtc` and SQL `*_utc` TEXT columns as ISO-8601 UTC — come from a small, proptest-verified civil-date conversion (days-from-civil algorithm, implemented here, AGPL) in `alvs-store`. The log `ts` field is tracing-subscriber's timer (D3). Comparisons (autosave-newer, lock-age ≥ 24 h) use epoch ms.
- **Rationale**: `time`/`chrono` are not baseline rows (G-11); the math is ~40 well-tested lines; deterministic and offline (constitution III).
- **Alternatives**: adding `time` as a direct dep (needs an ADR for no functional gain); epoch-only strings (unreadable in `Backups\` names, which 03 §3.8 pins to the human format).

## D6 — App folder resolution without tauri (03 §3.8)

- **Decision**: resolve `%APPDATA%\com.afterleads.videospirits` and `%LOCALAPPDATA%\com.afterleads.videospirits\{cache,models,logs,crashes,updates}` + `session.marker`, and `%TEMP%\AfterLeads-Video-Spirits\<pid>\`, from the `APPDATA`/`LOCALAPPDATA`/`TEMP` env vars; portable mode = `portable.flag` beside `std::env::current_exe()` → every folder becomes `<exe folder>\data\…`; all writes pass through `alvs-core::path` (SEC-04 canonicalization/long-path `\\?\`).
- **Rationale**: check_deps rule 2 makes `tauri` CMP-12-exclusive, so the library cannot ask Tauri for its dirs; the identifier-based paths 03 §3.8 describes are exactly these env-var locations (Tauri's own app-data resolution reads the same roots) — same bytes, no shell. The identifier string is verified against `app/src-tauri/tauri.conf.json` at implementation start (G-10).
- **Alternatives**: `dirs`/`directories` crates — not baseline; taking paths as parameters — pushes WP-03's own deliverable onto callers.

## D7 — Table-driven settings store (03 §3.6; REQ-SET-003)

- **Decision**: one static `KEY` table mirrors §3.6's 68 rows exactly (key, kind, range/enum, default, page). The document is `{ "settingsVersion": 1, … }` over a `BTreeMap<String, serde_json::Value>` (deterministic order). Load: version ≠ 1 or unreadable JSON → corrupt path (restore `settings.json.bak`; if none → all defaults + one E8002 notice — spec assumption); per-key validation walks the table — invalid values are replaced by defaults and collected into **one** E8002 notice; unknown keys ride through untouched. Write: 500 ms debounce on a writer thread, atomic write (D12), `.bak` refreshed on every successful write. Typed getters are provided for the keys WP-03 itself consumes (autosaveIntervalSec, backupsKeep, recentLimit, logLevel, redactDiagnostics, cacheFolder, defaultProjectFolder); full typed surface is WP-24's.
- **Rationale**: the canonical table *is* the schema (CMP-08 "settings (JSON + schema)"); a hand-written 68-field struct duplicates it and drifts; free-form rows (`keyboard.overrides`, `workspaces.saved`) validate shape only, as their rows say.

## D8 — SQLite cache database (03 §3.7; 02 §2.5 step 7)

- **Decision**: open `cache.sqlite` with `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL` (rusqlite `pragma_update`) and `busy_timeout` 5,000 ms (`Connection::busy_timeout`). Migrations are `.sql` files in `crates/alvs-store/migrations/NNN_<name>.sql`, embedded with `include_str!`, executed in order inside **one transaction** per run; the version lives in `meta['schema_version']`. `001_init.sql` contains the §3.7 DDL verbatim (ten tables + `jobs_by_state` index + CHECK constraints). Integrity: `PRAGMA quick_check` on open; on failure close, rename the file (and its `-wal`/`-shm` siblings) to `cache.sqlite.corrupt-<timestamp>`, recreate + migrate, return E8003.
- **Rationale**: §3.7 names the pragmas, the migrations location/transactionality, and the corrupt-rename contract verbatim; embedding keeps the shipped binary self-contained (no runtime file loads).
- **Alternatives**: an ORM or a builder-API migration set — more machinery than 15 `CREATE` statements need.

## D9 — Recent projects (03 §3.7, §3.9.2)

- **Decision**: `record(path, name, thumbnail_file, pinned)` called by save/open, `list()` newest-first by `opened_utc`, trim to `alvs.general.recentLimit` after each record; a `set_pinned` accessor exists for the later UI.
- **Rationale**: §3.7 defines the table; §3.9.2 names the update point; `recentLimit` (§3.6) is the trim.

## D10 — Autosave scheduler (03 §3.9.3; REQ-PRJ-003; NFR-REL-02)

- **Decision**: the decision logic is a pure function of `(dirty, ms_since_last_autosave, interval_sec)` → write/skip; a driver thread with an injectable `Clock` trait (`now_ms`) sleeps and ticks it. Tests drive the pure core + fake clock — no wall-clock dependence (constitution III). The sidecar is written with the atomic primitive and the main file is never touched.
- **Rationale**: REQ-PRJ-003's timing contract becomes an exact, testable predicate; NFR-REL-02's "≤ one interval lost" is the direct corollary.

## D11 — Lock files (03 §3.9.4; REQ-PRJ-011)

- **Decision**: create `<Name>.alvs.lock` with `OpenOptions::create_new` (maps to `CREATE_NEW` — exclusive). Content: `{ pid, host, user, startedUtc, appVersion }` (host/user from `COMPUTERNAME`/`USERNAME`). On existing lock: same host + process alive → read-only outcome (E1007); process dead → take-over allowed; different host → take-over only when the lock's `startedUtc` is ≥ 24 h old; lock removed on close. Liveness: `windows` `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` + `WaitForSingleObject(handle, 0)` (`WAIT_OBJECT_0` = exited) + `CloseHandle`.
- **Rationale**: §3.9.4 names every field and rule; `create_new` gives exclusivity without inventing locking machinery; the Win32 probe is the standard PID-liveness check and the `windows` crate is the baseline Win32 row.
- **Alternatives**: `tasklist` subprocess (spawns a process — violates the no-shell posture), `sysinfo` crate (not baseline).

## D12 — Atomic write primitive (03 §3.9.1; REQ-REL-007 save path)

- **Decision**: write `<target>.tmp` in the same folder → `File::sync_all()` (std maps this to `FlushFileBuffers` on Windows) → replace via `windows::Win32::Storage::FileSystem::MoveFileExW` with `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`. Any failure keeps the original untouched; the `.tmp` is best-effort removed; error mapping: full-disk kinds → E8001, permission → E8004, others → E1008 (save context). Applied to every user/app file this package writes (project, sidecar, backups, settings, `.bak`, marker, lock).
- **Rationale**: std's `fs::rename` on Windows calls `MoveFileExW` **without** `MOVEFILE_WRITE_THROUGH`, so the exact §3.9.1 flag pair needs the `windows` crate; `sync_all` already *is* `FlushFileBuffers`, so no unsafe handle plumbing for the flush half.
- **Alternatives**: `tempfile`-style crate — not baseline; `replace_file`/POSIX-style — not the Windows contract §3.9.1 states.

## D13 — Failure injection for the "simulated failure" tests (T-PRJ-002, T-REL-007)

- **Decision**: the atomic-write primitive is generic over a small `Fs` seam (`write_tmp`, `flush`, `replace`); production uses the std/windows implementation; tests inject failures at each step (tmp write, flush, replace) plus one *real* permission-denied case (read-only directory). No `#[cfg(test)]` in production code; the seam is `pub(crate)`.
- **Rationale**: disk-full cannot be reproduced deterministically on CI; REQ-REL-007 demands proof that previous files survive — injection at the exact steps is the honest equivalent; real ACL denial covers the permission half natively.

## D14 — Session marker and crash-recovery service (03 §3.9.5; REQ-REL-001 services)

- **Decision**: `session.marker` (content `{ pid, startedUtc }`) created at startup, removed on clean exit. `evaluate(main_path, autosave_path)` compares file modification times and returns the DLG-15 data: `Recovery { autosave_ts, saved_ts }` when a marker was left **and** the autosave is newer, else `None`. Choosing the autosave opens it as an unsaved document (its next save writes the main file) — that policy is expressed by the returned offer plus an `open_recovery` path that loads the sidecar without locking semantics changes.
- **Rationale**: §3.9.5 verbatim; the service exposes exactly what the (later) dialog renders; timestamps are compared, not existence (spec edge case).

## D15 — Redaction helper in `alvs-core` (CMP-01 "logging setup helpers"; REQ-RL-003)

- **Decision**: `alvs-core::redact::redact_paths(&str) -> String` replaces path-shaped substrings — drive-letter absolutes (`X:\…`), UNC (`\\server\share\…`), and the known env roots — with `<path>`; pure string work, no deps. Debug-level log sites and diagnostics export call it; diagnostics application is gated by `alvs.advanced.redactDiagnostics` (default true).
- **Rationale**: CMP-01's own description places "logging setup helpers" in `alvs-core`; every crate can then log paths safely without depending on `alvs-store` (which most rows forbid anyway).

## D16 — Scope fences (spec FR-019)

Excluded and owned elsewhere: recovery/backup/settings **dialogs** and banners (WP-05+ UI), export-path disk errors (WP-22), cache **content** writers (thumbnails/waveforms/proxies/analysis) and LRU eviction (WP-07/08), job re-queue (WP-07), model downloads (WP-21), single-instance forwarding + startup/shutdown orchestration (WP-05), `.alvs` serialization types + migration step registry (WP-02, consumed here), settings UI (WP-24).

## D17 — No async runtime in `alvs-store`

- **Decision**: std threads + `std::sync::mpsc` for the settings-debounce writer and the autosave driver; no `tokio`, no `crossbeam`. The crate stays runtime-agnostic; CMP-12 owns async composition (02 §2.13 thread table).
- **Rationale**: smallest surface, deterministic tests; the baseline's tokio row serves the app backend, not this library.

## D18 — Test and coverage posture

No coverage threshold is in the WP-03 card (unlike WP-02's) — none is invented (G-02). Suites: the card's seven (T-PRJ-002/003/004/008, T-SET-003, T-REL-003, T-REL-007 save part) as named filters plus unit coverage of every module; all deterministic, offline, no wall-clock (Clock/Fs injection, proptest only for the civil-date math). Workspace gates (verify.ps1: docs, commits, fmt, clippy, dependency-policy, crate-rules, placeholders, rust-test, bindings-drift) must stay green with the new crate.
