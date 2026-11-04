---
description: "Task list for WP-03 — Storage and application services"
---

# Tasks: WP-03 — Storage and application services

**Input**: Design documents from `/specs/003-storage-services/` (plan.md, spec.md, research.md D1–D18, data-model.md, contracts/crate-apis.md, quickstart.md)

**Prerequisites**: plan.md (owner-approved 2026-11-03 incl. OD-11), spec.md (7 user stories P1–P7)

**Tests**: Included — spec FR-018 explicitly requires the card's seven acceptance suites (T-PRJ-002/003/004/008, T-SET-003, T-REL-003, T-REL-007 save part) plus unit coverage; all deterministic via the Clock/Fs seams (research D10/D13), proptest seeded (plan §Testing).

**Organization**: Phases 1–2 are shared infrastructure; Phases 3–9 are the user stories in priority order; Phase 10 is polish. Cross-story seams (`DocumentHooks`, `RecentProjects`, `Clock`, Fs) keep each story independently testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US7 per spec.md; setup/foundational/polish carry no label
- File paths are repository-relative

## Path Conventions

- New crate: `crates/alvs-store/src/`, `crates/alvs-store/tests/`, `crates/alvs-store/migrations/`
- Touched crates: `crates/alvs-core/src/` (additive), `app/src-tauri/src/` (composition module)
- Workspace root: `Cargo.toml` (members glob already covers the new crate)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The crate skeleton and the canonical SQL schema.

- [X] T001 Create `crates/alvs-store/Cargo.toml` with the D1 dependency set (rusqlite 0.41.1 `bundled`; tracing 0.1.44; tracing-subscriber 0.3.23 features `json`,`time`; tracing-appender 0.2.5; windows 0.62 features `Win32_Storage_FileSystem`,`Win32_System_Threading`,`Win32_Foundation`; serde/serde_json/thiserror/uuid at locked versions; internal `alvs-core`; dev-deps `alvs-model`, `proptest`; `[lints] workspace = true`) and `crates/alvs-store/src/lib.rs` with SPDX header + crate docs; `cargo build -p alvs-store --locked` and `python tools/check_deps.py` and `cargo deny check` all pass on the skeleton
- [X] T002 [P] Create `crates/alvs-store/migrations/001_init.sql` with the complete 03 §3.7 DDL verbatim: tables `meta`, `media_cache`, `thumbnails` (kind CHECK poster/filmstrip), `waveforms`, `proxies` (state CHECK), `jobs` (state CHECK), `recent_projects` (path PK), `models` (state CHECK), `encoder_probe`, `analysis` (kind CHECK), plus `CREATE INDEX jobs_by_state ON jobs (state, priority DESC, created_utc)`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Everything every story consumes: error codes, timestamps, paths, the atomic-write primitive, the settings key table.

**⚠ CRITICAL**: No user story work begins until this phase is complete.

- [X] T003 [P] Extend `crates/alvs-core/src/error.rs` with `ErrorCode::{E1007, E1008, E5003, E8001, E8002, E8003, E8004}` and their 12 §12.13 severities (E1007 warning, E1008 error, E5003 error, E8001 error, E8002 warning, E8003 warning, E8004 error) + doc comments quoting the catalog rows; extend the severity unit test (research D4)
- [X] T004 [P] Create `crates/alvs-store/src/error.rs`: `StoreError` (thiserror) with exactly one variant per code (ReadOnlyNewer E1004, MigrationFailed E1005, Locked E1007, SaveFailed E1008, DiskFull E8001, SettingsRestored E8002, CacheRebuilt E8003, AccessDenied E8004, WriteFailed E5003, PathRejected E9005), `fn code(&self)` + `fn envelope(self)` per 04 §4.9; unit test the 1:1 mapping and `error.<code>` message keys
- [X] T005 [P] Create `crates/alvs-store/src/utc.rs`: epoch-ms → ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SS.sssZ`) and → `yyyyMMdd-HHmmss` via the days-from-civil conversion (research D5); seeded proptest over a bounded era plus a fixed leap-year table (1996–2096 known dates); no `time`/`chrono` dependency
- [X] T006 [P] Create `crates/alvs-store/src/paths.rs`: `AppPaths::resolve()` from `APPDATA`/`LOCALAPPDATA`/`TEMP` + identifier `com.afterleads.videospirits` (verify against `app/src-tauri/tauri.conf.json`), `resolve_portable(exe_dir)` when `portable.flag` sits beside `current_exe()` → `<exe folder>\data\…`, one accessor per 03 §3.8 row (settings, library subfolders, cache db/files, models, logs, crashes, session marker, updates, per-PID temp, default project folder); every root passes `alvs_core::path::check_path` containment; unit tests for installed + portable layouts and long-path/Unicode names
- [X] T007 Create `crates/alvs-store/src/atomic.rs`: the 03 §3.9.1 primitive — write `<target>.tmp` in the same folder → `File::sync_all()` (FlushFileBuffers) → `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)` via the `windows` crate with `// SAFETY:` comment + test (OD-11); keep the original on any failure; map io errors (full-disk kinds → E8001, permission → E8004, others → E1008); `pub(crate) trait Fs` seam (always compiled, never `#[cfg(test)]`-gated) with the std/windows implementation; in-crate `#[cfg(test)]` unit tests: step-wise failure injection at tmp/flush/replace + real permission cases — pre-create `<target>.tmp` as a directory (open fails ERROR_ACCESS_DENIED) and read-only target file at the replace step (research D12/D13; review F-04/F-05)
- [X] T008 [P] Create `crates/alvs-store/src/keys.rs`: the static settings table mirroring all 68 rows of 03 §3.6 verbatim (key, kind, range/enum, default, page) incl. free-shape rows (`alvs.keyboard.overrides` object, `alvs.workspaces.saved` array) + per-kind validation predicates; unit tests: every key has a default, ranges reject their boundary+1 and accept boundary values, enums reject unknown members

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 — Saves that cannot corrupt, opens that find the media (Priority: P1) ⭐ MVP

**Goal**: The 03 §3.9.2/§3.9.6 save/open pipeline with atomic writes, size gate, migration with `.v<old>.bak`, E1004 read-only, and relative-first media fallback (REQ-PRJ-002; REQ-REL-007 save path).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_prj_002 t_rel_007` — round-trip lossless, relative fallback, simulated failures keep the previous file intact.

### Tests for User Story 1 (write first; must fail before implementation)

- [X] T009 [P] [US1] Create `crates/alvs-store/tests/t_prj_002.rs`: save/open round-trip through a `DocumentHooks` impl over `alvs_model::Project` (dev-dep); media resolution when the absolute path is missing but `rel_path` resolves (and both-missing fails that item only); simulated tmp/flush/replace failures leave the previous file byte-identical; raw-size gate rejects an oversized document before parse; `schemaVersion`-newer fixture opens read-only (E1004); test-local hooks with synthetic versions prove `.v<old>.bak` is written and E1005 keeps the original (research D2; review F-03)
- [X] T010 [P] [US1] Create `crates/alvs-store/tests/t_rel_007_save.rs`: injected disk-full and real permission failures (pre-created `.tmp` directory; read-only target at replace) across save/autosave/backup/settings writers keep previous files intact and map to E8001/E8004/E1008

### Implementation for User Story 1

- [X] T011 [US1] Create `crates/alvs-store/src/project.rs`: `trait DocumentHooks` (serialize; parse_and_validate; migrate → (bytes, migrated) with E1005; fixup_media_paths rel-first-when-absolute-missing) + `trait RecentProjects` (record/list/set_pinned) + `ProjectStore<H>`: `save(Arc<H::Doc>)` = background-thread serialize → atomic write → backup into `Backups\` → prune to `alvs.general.backupsKeep` → delete sidecar → `recent.record`; `open(path)` = exist → raw-length size gate (`alvs_model::validate` size hook, 03 §3.5.15) → parse_and_validate → migrate (write `<Name>.alvs.v<old>.bak` first) → newer → read-only E1004 → basic lock acquire (creating `crates/alvs-store/src/lock.rs` now with the exclusive `create_new` acquire writing `{pid, host, user, startedUtc, appVersion}` and the release; the liveness/takeover semantics arrive in T022) → fixup_media_paths; every write SEC-04-contained
- [X] T012 [US1] Create `app/src-tauri/src/project_io.rs`: `ProjectHooks` implementing `DocumentHooks` for `alvs_model::Project` (03 §3.2 serialization; parse + `validate`; `alvs_model::migrate` driver + result validation; media abs+rel resolution) and add the row-legal app-crate deps (`alvs-core`, `alvs-model`, `alvs-store`); `cargo build --workspace --locked` passes
- [X] T013 [US1] Make T009/T010 green; verify the suite filters `t_prj_002` and `t_rel_007` pass with `cargo nextest run -p alvs-store --locked`

**Checkpoint**: US1 independently functional (MVP: saves/opens are crash-safe).

---

## Phase 4: User Story 2 — Autosave and crash recovery (Priority: P2)

**Goal**: Dirty+interval sidecar writes that never touch the main file; session marker + recovery offer with both timestamps (REQ-PRJ-003; REQ-REL-001 services; NFR-REL-02).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_prj_003` — exact firing condition with a fake clock; marker lifecycle; recovery predicate by mtime.

### Tests for User Story 2

- [X] T014 [P] [US2] Create `crates/alvs-store/tests/t_prj_003.rs`: autosave fires exactly when dirty ∧ `ms_since_last ≥ interval` (fake `Clock`; interval from the keys table default 60), sidecar content equals the snapshot, main file untouched; marker created/cleared; `evaluate` returns the offer with both timestamps only when marker present ∧ autosave mtime > main mtime

### Implementation for User Story 2

- [X] T015 [US2] Create `crates/alvs-store/src/autosave.rs`: pure decision core `fn decision(dirty, since_last_ms, interval_sec)` + `Clock` trait + production driver thread writing `<Name>.alvs.autosave` via the atomic primitive; sidecar deleted on save/clean close (03 §3.9.3; research D10)
- [X] T016 [US2] Create `crates/alvs-store/src/session.rs`: marker `{pid, startedUtc}` create-at-startup/clear-on-clean-exit + `evaluate(main, autosave)` by mtime → `RecoveryOffer` (03 §3.9.5; research D14); make T014 green

**Checkpoint**: US1+US2 independently functional.

---

## Phase 5: User Story 3 — Backups the user can restore (Priority: P3)

**Goal**: Timestamped per-save backups, prune to keep-count, list with time+size, restore into a new file (REQ-PRJ-004).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_prj_004`.

### Tests for User Story 3

- [X] T017 [P] [US3] Create `crates/alvs-store/tests/t_prj_004.rs`: backup written per save named `<Name>_<yyyyMMdd-HHmmss>.alvs`, pruned to `backupsKeep` (default 20; boundaries 1 and 100), listing exposes time+size, restore copies into a new file and leaves the backup untouched

### Implementation for User Story 3

- [X] T018 [US3] Create `crates/alvs-store/src/backup.rs`: write via the atomic primitive with utc.rs stamps, prune newest-keep, list, restore-to-new-file; make T017 green

**Checkpoint**: US1–US3 independently functional.

---

## Phase 6: User Story 4 — Settings that never break (Priority: P4)

**Goal**: The settings document store: validation against the keys table, one E8002 notice, unknown-key preservation, `.bak` recovery, 500 ms debounced atomic writes (REQ-SET-003).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_set_003`.

### Tests for User Story 4

- [X] T019 [P] [US4] Create `crates/alvs-store/tests/t_set_003.rs`: out-of-range `alvs.general.autosaveIntervalSec=5` → default 60 + exactly one E8002 notice; unknown keys survive load→write; corrupt file with `.bak` restores it; corrupt without `.bak` → defaults + E8002; rapid changes coalesce to one debounced write (injected clock)

### Implementation for User Story 4

- [X] T020 [US4] Create `crates/alvs-store/src/settings.rs`: `BTreeMap` document + `settingsVersion=1` gate + validation walk against keys.rs (invalid → default, one notice) + writer thread (500 ms debounce, injected clock in tests) + `flush()` + `settings.json.bak` refresh on every successful write + corrupt-recovery ladder (03 §3.6; research D7); typed getters for the consumed keys (autosaveIntervalSec, backupsKeep, recentLimit, logLevel, redactDiagnostics, cacheFolder, defaultProjectFolder); make T019 green

**Checkpoint**: US1–US4 independently functional.

---

## Phase 7: User Story 5 — One writer per project (Priority: P5)

**Goal**: Full lock semantics — liveness probe, read-only E1007, dead/other-host takeover rules (REQ-PRJ-011).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_prj_008`.

### Tests for User Story 5

- [X] T021 [P] [US5] Create `crates/alvs-store/tests/t_prj_008.rs`: exclusive create; live same-host lock → ReadOnly + E1007; dead pid (spawn a real sleeping child via `std::process::Command`, kill and reap it, then probe its pid) → TakeoverAvailable; other host with startedUtc < 24 h → no takeover; ≥ 24 h → takeover; release removes the file

### Implementation for User Story 5

- [X] T022 [US5] Complete `crates/alvs-store/src/lock.rs`: liveness via `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` + `WaitForSingleObject(handle, 0)` + `CloseHandle` with `// SAFETY:` comment + test (OD-11); `LockOutcome` rules from 03 §3.9.4; age from startedUtc epoch-ms; wire full semantics into `ProjectStore::open`; make T021 green

**Checkpoint**: US1–US5 independently functional.

---

## Phase 8: User Story 6 — App folders, cache DB, recent projects (Priority: P6)

**Goal**: SQLite open with §3.7 pragmas, embedded transactional migrations, integrity rebuild (E8003), and the recent-projects service.

**Independent Test**: `cargo nextest run -p alvs-store --locked cache` — pragmas, migration versioning, corrupt-rebuild, recent behavior.

### Tests for User Story 6

- [X] T023 [P] [US6] Create `crates/alvs-store/tests/cache.rs`: `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000` after open; `meta.schema_version` advances through migrations; simulated corrupt file → renamed `cache.sqlite.corrupt-<timestamp>` (with `-wal`/`-shm` handled) → rebuilt + E8003; recent upsert/list newest-first/trim to `recentLimit`/pin toggle

### Implementation for User Story 6

- [X] T024 [US6] Create `crates/alvs-store/src/cache.rs`: `Connection` with the four §3.7 pragmas, `include_str!` migration runner (files sorted by `NNN_`, one transaction, version in `meta`), `PRAGMA quick_check` → rename+rebuild path (research D8)
- [X] T025 [US6] Create `crates/alvs-store/src/recent.rs`: the SQLite `RecentProjects` implementation for `recent_projects` (03 §3.7) with trim to `alvs.general.recentLimit` (research D9); make T023 green

**Checkpoint**: US1–US6 independently functional.

---

## Phase 9: User Story 7 — Logs that respect the user (Priority: P7)

**Goal**: JSON-lines logging with the custom 10 MB × 10 rotating writer, required fields, level from settings, and path redaction (REQ-REL-003).

**Independent Test**: `cargo nextest run -p alvs-store --locked t_rel_003`.

### Tests for User Story 7

- [X] T026 [P] [US7] Create `crates/alvs-store/tests/t_rel_003.rs`: writes past 10 MB rotate to ≤ 10 files (`app.log`, `app.log.1`…`app.log.9`, older deleted); lines carry `ts`/`level`/`target`/`msg` (+ `job_id`/`project_id`/`code` when provided); user paths in debug lines are redacted; `redact_paths` unit coverage lives beside the helper

### Implementation for User Story 7

- [X] T027 [P] [US7] Create `crates/alvs-core/src/redact.rs`: `redact_paths(&str) -> String` replacing drive-letter absolutes, UNC, and env-root paths with `<path>` (CMP-01 "logging setup helpers"; research D15) with unit tests
- [X] T028 [US7] Create `crates/alvs-store/src/logging.rs`: `tracing_subscriber::fmt().json()` with `UtcTime::rfc_3339`, the `SizeRotatingWriter` (10 MB threshold, keep 10) wrapped in `tracing_appender::non_blocking` with a `LogGuard` flushed on drop, level from `alvs.advanced.logLevel` (research D3; naming per spec Assumptions); make T026 green

**Checkpoint**: all seven stories independently functional.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Cross-story wiring, evidence, and gates.

- [X] T029 [P] Wire settings consumption across services (autosave interval, backup keep-count, recent limit, log level, redaction flag) and add the SEC-04 containment test covering every writer path (contracts guarantee G7)
- [X] T030 Run the full `specs/003-storage-services/quickstart.md` validation (all seven filters + the §3 integration run + `pwsh -File tools/verify.ps1` all available gates + `python tools/git/check_commits.py origin/main..HEAD`) and record commands/results into `specs/003-storage-services/evidence.md` (constitution III; G-09)
- [X] T031 [P] Final sweep: `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, no placeholders (`python tools/check_placeholders.py`), crate docs complete, `python tools/docs/check_docs.py` green (docs/00 at 1.1.0 + OD-11 carried)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: immediate; T002 parallel with T001's tail
- **Foundational (Phase 2)**: after Setup; **blocks all stories** (T007 is the critical path — everything writes through it)
- **Stories (Phases 3–9)**: after Phase 2, in priority order for a single implementer (US1 → US2 → … → US7); US3/US4 ride US1's pipeline but are separately testable; US5 extends US1's basic lock; US6 implements the trait US1 defined
- **Polish (Phase 10)**: after all stories

### User Story Dependencies

- **US1 (P1)**: foundation only — MVP
- **US2 (P2)**: foundation + US1's sidecar/atomic usage (testable alone via the pure core)
- **US3 (P3)**: foundation + US1 save pipeline hooks (prune/list/restore testable alone)
- **US4 (P4)**: foundation only (keys.rs + atomic)
- **US5 (P5)**: foundation + US1's basic lock module
- **US6 (P6)**: foundation only (trait from US1 compiles standalone)
- **US7 (P7)**: foundation + paths/keys

### Parallel Opportunities

- T002–T008 are file-disjoint and [P]-marked (largest parallel batch)
- Within each story: the test file and non-test modules are file-disjoint ([P])
- T023/T024 (cache) parallel with T026/T027/T028 (logging) if stories are staffed separately

### Within Each User Story

- Tests written first and failing (template rule; FR-018 suites)
- Seams/traits before services; services before the green-up task
- Story complete (suite green) before the next priority

---

## Parallel Example: User Story 1

```text
Task: "t_prj_002 suite in crates/alvs-store/tests/t_prj_002.rs"   (T009)
Task: "t_rel_007_save suite in crates/alvs-store/tests/t_rel_007_save.rs" (T010)
# then, after both compile-fail as expected:
Task: "project.rs pipeline" (T011)  +  "app/src-tauri/src/project_io.rs" (T012)  # file-disjoint
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 + Phase 2 → foundation ready
2. Phase 3 (US1) → **STOP and VALIDATE**: `t_prj_002` + `t_rel_007` green, round-trip crash-safe
3. Continue P2→P7 in order, validating each story's filter before the next

### Incremental Delivery

Each story lands with its suite green and a commit (Conventional Commits + DCO); `/wp-verify` runs after each stage per the card protocol.

---

## Notes

- Every `unsafe` block carries `// SAFETY:` + a test (docs/02 §2.14, OD-11): exactly `MoveFileExW` (T007) and the liveness probe (T022)
- No `time`/`chrono`/`tempfile`/`dirs` direct deps (G-11; research D1/D5)
- Determinism: fake `Clock`, injectable `Fs`, seeded proptest (constitution III)
- Commit after each task or logical group; stop at any checkpoint to validate a story independently

---

## Phase 11: Convergence

- [X] T032 Gate the media-path probes in `fixup_media_paths` per SEC-04: before any filesystem access on a document-controlled candidate (`item.path`, and the `project_dir.join(rel_path)` resolution — note `join` with an absolute or UNC `rel_path` replaces the base), route the candidate through `alvs_core::path::check_path` with `PathOrigin::ProjectFile` semantics — device paths and reserved names must be denied without a probe (the item stays offline), UNC paths must never be probed and must surface as `NeedsConfirmation` data for the caller (THR-13: an SMB stat leaks NTLM credentials; the confirmation dialog itself is WP-05 per FR-019), while legitimate local absolute media outside the project folder must still resolve (containment is the write rule; the read gate is device/reserved/UNC) — with tests covering a UNC `item.path`, a UNC/absolute `rel_path` join escape, a device path, and a reserved name (FR-017; plan data-model validation rule 3; 05 §5.7 SEC-04; constitution VII) (partial)
