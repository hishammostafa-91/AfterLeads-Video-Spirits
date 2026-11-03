# Feature Specification: WP-03 — Storage and application services

**Feature Branch**: `feat/003-storage-services`

**Created**: 2026-11-03

**Status**: Draft

**Input**: User description: "Implement WP-03 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: REQ-PRJ-002, REQ-PRJ-003, REQ-PRJ-004, REQ-PRJ-011, REQ-SET-003, REQ-REL-001 (services), REQ-REL-003, REQ-REL-007 (save path). Acceptance tests: T-PRJ-002, T-PRJ-003, T-PRJ-004, T-PRJ-008, T-SET-003, T-REL-003, T-REL-007 (save part). Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents."

**Canonical basis**: WP-03 card (docs/10 §10.7). Requirements cited from docs/01-product.md (REQ-PRJ-002/003/004/011, REQ-SET-003, REQ-REL-001/003/007 rows quoted where binding; NFR-REL-02 via the T-PRJ-003 acceptance row). Design authorities from the card's Read line: 03 §3.6–3.9; 02 §2.5, §2.11; 05 SEC-03/04; test definitions from 09 §9.10. Nothing is added beyond those documents (G-01, G-02, constitution I).

## User Scenarios & Testing *(mandatory)*

Actors: the application backend that later work packages assemble (the app shell of WP-05 calls these services at startup/shutdown per 02 §2.5) and, through it, the end user of REQ-PRJ-002/003/004/011, REQ-SET-003, REQ-REL-001/003/007. This package is the product's persistence layer: every project file on disk, every setting, every backup, lock, autosave, and log line stands on it.

### User Story 1 - Saves that cannot corrupt, opens that find the media (Priority: P1)

The user saves a project and trusts it: the `.alvs` file (UTF-8 JSON per 03 §3.2) is written atomically — a crash or power loss mid-write never leaves a torn file; the previous file survives any failure (REQ-PRJ-002; REQ-REL-007 save path). On open, the file is size-checked before parsing (03 §3.5.15 via SEC-03), migrated if older with the original copied to a `.bak` first (03 §3.9.6), validated (03 §3.5), and opened read-only with E1004 if newer. Media paths are stored absolute plus relative to the project folder, and when the absolute path is missing the relative path is tried first — a project moved with its folder keeps working (REQ-PRJ-002).

**Why this priority**: G-16 (never lose user data) lives or dies here; every other storage story builds on this pipeline, and the data-loss bound of NFR-REL-02 ("zero corrupted project files") is its direct obligation.

**Independent Test**: T-PRJ-002 suite — save/open round-trip, relative-path fallback, atomic write under simulated failure; T-REL-007 (save part) — disk-full and permission errors keep previous files intact (09 §9.10 rows).

**Acceptance Scenarios**:

1. **Given** a dirty project, **When** it is saved, **Then** serialization happens on a background thread, the file is replaced atomically, a timestamped backup is written and pruned, the autosave sidecar is deleted, and recent projects are updated (03 §3.9.2).
2. **Given** a write that fails on the final replace (disk full, permission denied), **When** save is attempted, **Then** the previous file is intact and an actionable error (E8001/E8004/E1008/E5003 per 12 §12.13) is reported (03 §3.9.1; REQ-REL-007).
3. **Given** a project whose media absolute path no longer exists but whose relative path does, **When** the project opens, **Then** the media resolves via the relative path (REQ-PRJ-002).
4. **Given** a file larger than the 03 §3.5.15 document limit, **When** open is attempted, **Then** it is rejected by the size check before parsing (SEC-03; 03 §3.5 rule 15).

---

### User Story 2 - Autosave and crash recovery (Priority: P2)

While the user edits, a sidecar `<Name>.alvs.autosave` is refreshed whenever the project is dirty and `alvs.general.autosaveIntervalSec` (default 60) has elapsed — autosave never touches the main file (03 §3.9.3). A session marker is created at startup and removed only on a clean exit (03 §3.9.5). After a crash, the recovery offer (the data behind DLG-15) presents the autosave, the last saved version, or discard, with both timestamps; choosing the autosave opens it as an unsaved document whose next save writes the main file (REQ-REL-001 services; REQ-PRJ-003). The user never loses more than one autosave interval of work and never receives a corrupted project file (NFR-REL-02).

**Why this priority**: the crash-recovery promise (UC-08) is a P0 requirement, and its service half — marker detection plus "autosave newer than main" — must exist before any UI can show the dialog.

**Independent Test**: T-PRJ-003 suite — autosave timing and content (REQ-PRJ-003, NFR-REL-02; 09 §9.10 row), with an injected clock so timing is deterministic (constitution III).

**Acceptance Scenarios**:

1. **Given** a project dirty for longer than the interval, **When** the autosave tick fires, **Then** the snapshot is atomically written to the sidecar and the main file is untouched (03 §3.9.3).
2. **Given** a clean exit, **When** shutdown completes, **Then** the session marker is removed; given a crash, the marker remains (03 §3.9.5).
3. **Given** a leftover session marker at startup and a last-open project whose autosave is newer than its main file, **When** recovery is evaluated, **Then** the three options with both timestamps are offered (REQ-REL-001; DLG-15 data).

---

### User Story 3 - Backups the user can restore (Priority: P3)

Every manual save also writes a timestamped backup `Backups\<Name>_<yyyyMMdd-HHmmss>.alvs`, keeping the newest `alvs.general.backupsKeep` (default 20). The user can list backups with time and size and restore one into a new file (the data behind DLG-16) (REQ-PRJ-004).

**Why this priority**: backups are the user's undo for mistakes the undo engine cannot reach (overwritten saves); they ride the save pipeline of Story 1 and are cheap to verify (T-PRJ-004).

**Independent Test**: T-PRJ-004 suite — backup creation, pruning, restore (09 §9.10 row).

**Acceptance Scenarios**:

1. **Given** 25 manual saves with `backupsKeep` at its default, **When** the 26th backup would appear, **Then** only the newest 20 remain (REQ-PRJ-004).
2. **Given** existing backups, **When** a restore is requested, **Then** the chosen backup is copied into a new project file and the original backup is untouched (REQ-PRJ-004; DLG-16 data).

---

### User Story 4 - Settings that never break the app (Priority: P4)

Settings persist in `settings.json` as a flat object with `settingsVersion`, every key of the 03 §3.6 table carrying its documented type, range, and default; keys absent from the file take defaults, unknown keys are preserved on write, each value is validated and an invalid value is replaced by its default and reported once (E8002). Writes are debounced 500 ms and atomic, and a `settings.json.bak` (last good file) enables recovery when the file is corrupt (REQ-SET-003; 03 §3.6).

**Why this priority**: every preference the product honors (autosave interval, backup count, recent limit, log level, redaction flag, cache folder) is read through this store; a broken settings file must degrade to defaults, never to a broken app.

**Independent Test**: T-SET-003 suite — settings validation, unknown keys, corruption recovery (09 §9.10 row).

**Acceptance Scenarios**:

1. **Given** a settings file with an out-of-range `alvs.general.autosaveIntervalSec` (say 5), **When** settings load, **Then** the default 60 applies, the invalid value is reported once (E8002), and the unknown keys survive the next write (03 §3.6; REQ-SET-003).
2. **Given** a corrupt `settings.json` with an existing `.bak`, **When** settings load, **Then** the last good file is restored (REQ-SET-003).
3. **Given** rapid consecutive changes, **When** writes are requested, **Then** the store debounces 500 ms and writes atomically (03 §3.6 write policy).

---

### User Story 5 - One writer per project (Priority: P5)

Opening a project creates `<Name>.alvs.lock` exclusively with `{ pid, host, user, startedUtc, appVersion }`. A second open attempt on the same host with a live process yields read-only (E1007); a dead process or a lock older than 24 h on another host can be taken over after confirmation; the lock is removed on close (REQ-PRJ-011; 03 §3.9.4).

**Why this priority**: silent double-writing is data loss without a crash; the lock protocol is small, self-contained, and a P0 requirement with its own acceptance test.

**Independent Test**: T-PRJ-008 suite — lock file creation, read-only, stale takeover (09 §9.10 row).

**Acceptance Scenarios**:

1. **Given** a project open in a live process, **When** another open is attempted, **Then** the caller is offered read-only (E1007) and save is disabled for that session (03 §3.9.4).
2. **Given** a lock whose recorded process is dead, **When** open is attempted, **Then** take-over is offered (REQ-PRJ-011).
3. **Given** a lock from another host younger than 24 h, **When** open is attempted, **Then** take-over is not offered (03 §3.9.4).

---

### User Story 6 - App folders, the cache database, and recent projects (Priority: P6)

Every application folder of 03 §3.8 resolves from the product identifier — settings, user library, cache, models, logs, crashes, session marker, updates, per-process temporary — with portable mode (`portable.flag` next to the executable) relocating everything under `<exe folder>\data\…`, Unicode and long paths handled through `alvs-core::path`. The cache database `cache.sqlite` opens with `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`; migrations run from `crates/alvs-store/migrations/NNN_<name>.sql` in a transaction with the version in `meta`; a corrupt database is renamed `cache.sqlite.corrupt-<timestamp>` and rebuilt (E8003). Recent projects are tracked in their table, updated on save/open, and trimmed to `alvs.general.recentLimit` (03 §3.7, §3.8; 02 §2.5 steps 3 and 7).

**Why this priority**: these are the foundations Stories 1–5 and every later service stand on; they carry no dedicated acceptance test of their own but are exercised by all of them and by the startup contract of 02 §2.5.

**Independent Test**: unit and integration suites in this package's CI run — folder resolution (installed and portable), database open + migration + integrity-rebuild, recent-project upsert/trim — plus their use by T-PRJ-002's save/open round-trip.

**Acceptance Scenarios**:

1. **Given** portable mode with `portable.flag` next to the executable, **When** app paths resolve, **Then** every folder of 03 §3.8 lives under `<exe folder>\data\…` (03 §3.8).
2. **Given** a cache database missing the current schema version, **When** it opens, **Then** pending migrations run in a transaction and the version in `meta` advances (03 §3.7).
3. **Given** a corrupt `cache.sqlite`, **When** open is attempted, **Then** it is renamed with a timestamp, rebuilt, and E8003 is reported (03 §3.7; 02 §2.5 step 7).
4. **Given** saves of distinct projects, **When** recent projects are read, **Then** entries are newest-first, deduplicated by path, and trimmed to `alvs.general.recentLimit` (03 §3.7).

---

### User Story 7 - Logs that respect the user (Priority: P7)

Logs roll at 10 MB keeping 10 files in the logs folder, as structured JSON lines with `ts`, `level`, `target`, `msg`, plus `job_id`, `project_id`, `code` when known; media content, caption text, and full user paths never appear at info level (paths only at debug, redacted in diagnostics bundles), and a redaction helper serves diagnostics export (REQ-REL-003; 02 §2.11; 03 §3.9.8).

**Why this priority**: diagnosis depends on it, but it serves every other story rather than the user directly; the level honors `alvs.advanced.logLevel` and redaction honors `alvs.advanced.redactDiagnostics` (both 03 §3.6).

**Independent Test**: T-REL-003 suite — log rotation and redaction (09 §9.10 row).

**Acceptance Scenarios**:

1. **Given** log output past 10 MB, **When** the file rolls, **Then** at most 10 files are kept (03 §3.9.8; REQ-REL-003).
2. **Given** a log line carrying a user file path, **When** diagnostics are exported with redaction on (the default), **Then** the path is redacted and the structured fields are intact (REQ-REL-003; 03 §3.6).

---

### Edge Cases

- Crash between `.tmp` write and replace: the original file must remain and the `.tmp` be left behind or cleaned; no torn main file ever (03 §3.9.1; NFR-REL-02).
- Disk full or permission denied at any write (project, autosave, backup, settings, cache): previous files stay intact, actionable codes E8001/E8004/E1008/E5003 (REQ-REL-007; 12 §12.13).
- Corrupt `settings.json` with **no** backup yet (first run): fall back to defaults and report E8002 (03 §3.6; assumption below).
- Corrupt `cache.sqlite`: rename + rebuild, E8003 — cache loss is acceptable, user data is not (03 §3.7).
- Lock without a live process but a *different host* and younger than 24 h: no take-over (03 §3.9.4).
- Autosave sidecar present while the main file is newer (crash after save, before sidecar delete): recovery must compare timestamps, not mere existence (03 §3.9.5).
- Project file at `schemaVersion` > current: read-only with E1004, unknown fields kept in memory (03 §3.9.6; migration driver from WP-02).
- Media whose absolute and relative paths are both missing: resolution fails for that item only; the rest of the project still opens (REQ-PRJ-002; the offline-media UI is later packages).
- Paths with Unicode, spaces, or beyond MAX_PATH: handled via `alvs-core::path` (`\\?\` when needed); device paths and reserved names rejected, UNC paths from a project file gated behind confirmation (SEC-04; 03 §3.8).
- Settings file missing `settingsVersion` or carrying an unknown one: treated by the validation/backup rules above, never a crash (03 §3.6).
- Rapid setting changes: one debounced write; process exit flushes pending writes (03 §3.6; 02 §2.5 shutdown step 4).

## Requirements *(mandatory)*

All requirements are scoped to the WP-03 card. IDs are feature-local (FR-xxx) and cite their canonical source.

### Functional Requirements

- **FR-001**: The crate MUST provide application path resolution for every location of 03 §3.8 (settings, user library, cache database and files, speech models, logs, crash dumps, session marker, updates rollback, per-PID temporary, default project folder) from the identifier `com.afterleads.videospirits`, with portable mode (`portable.flag` beside the executable → `<exe folder>\data\…`) replacing every folder, Unicode and long-path support via `alvs-core::path` (03 §3.8).
- **FR-002**: The settings store MUST implement the full 03 §3.6 table — `settingsVersion`, every documented key with type/range/default (general, appearance, playback, timeline, captions, media, export, keyboard, AI, advanced, workspaces, UI-internal) — with absent keys defaulting and unknown keys preserved through load→write (03 §3.6; REQ-SET-003).
- **FR-003**: Each settings value MUST be validated on load; an invalid value is replaced by its default and reported once with E8002 (03 §3.6; 12 §12.13).
- **FR-004**: Settings persistence MUST debounce writes by 500 ms, write atomically (FR-005), keep `settings.json.bak` as the last good file, and recover from a corrupt file by restoring that backup (03 §3.6; REQ-SET-003; T-SET-003).
- **FR-005**: The atomic-write primitive MUST follow 03 §3.9.1 exactly for all user and app files: write `<target>.tmp` in the same folder → flush (`FlushFileBuffers`) → replace via `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`; on error keep the original and report (E1008/E8001/E8004).
- **FR-006**: Saving a project MUST run the 03 §3.9.2 pipeline: serialize the snapshot on a background thread → atomic write of the `.alvs` file → write a timestamped backup into `Backups\` → prune to `alvs.general.backupsKeep` → delete the autosave sidecar → update `recent_projects` (03 §3.9.2; REQ-PRJ-002).
- **FR-007**: Opening a project MUST: check file existence; apply the document size limit to the raw length before parsing (`alvs_model::validate` size hook; 03 §3.5 rule 15; 05 SEC-03); parse; when `schemaVersion` is older, copy the original to `<Name>.alvs.v<old>.bak` then run the WP-02 migration driver and validate the result (E1005 on failure, original unchanged); when newer, open read-only with E1004 and keep unknown fields (03 §3.9.6); validate the model (03 §3.5); then create the lock (FR-010). Media items MUST store absolute and project-relative paths and resolve relative-first when the absolute path is missing (REQ-PRJ-002; T-PRJ-002).
- **FR-008**: The autosave scheduler MUST, when the project is dirty and `alvs.general.autosaveIntervalSec` has elapsed since the last autosave, serialize the snapshot and atomically write `<Name>.alvs.autosave`, never touching the main file (03 §3.9.3; REQ-PRJ-003; NFR-REL-02; T-PRJ-003).
- **FR-009**: Backups MUST be written as `Backups\<Name>_<yyyyMMdd-HHmmss>.alvs` on every manual save, pruned to the newest `alvs.general.backupsKeep`, listable with time and size, and restorable into a new file without altering the backup (REQ-PRJ-004; T-PRJ-004).
- **FR-010**: The lock service MUST create `<Name>.alvs.lock` exclusively on open with `{ pid, host, user, startedUtc, appVersion }`; an existing lock with a live same-host process yields read-only (E1007); a dead process, or another host with the lock older than 24 h, permits take-over after confirmation; the lock is removed on close (03 §3.9.4; REQ-PRJ-011; T-PRJ-008).
- **FR-011**: The session marker MUST be created at startup and removed on clean exit; when present at startup together with a last-open project whose autosave is newer than its main file, the recovery service MUST expose the three options (open autosave as an unsaved document, open last saved, discard) with both timestamps (03 §3.9.5; REQ-REL-001 services; 02 §2.5 step 9).
- **FR-012**: The cache database MUST open `cache.sqlite` with `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`; migrations MUST live in `crates/alvs-store/migrations/NNN_<name>.sql`, run in a transaction, and record the version in `meta`; the initial migration MUST create the full 03 §3.7 schema (`meta`, `media_cache`, `thumbnails`, `waveforms`, `proxies`, `jobs`, `recent_projects`, `models`, `encoder_probe`, `analysis`, plus `jobs_by_state`); a corrupt database MUST be renamed `cache.sqlite.corrupt-<timestamp>` and rebuilt with E8003 (03 §3.7; 02 §2.5 step 7).
- **FR-013**: The recent-projects service MUST upsert path, name, thumbnail, `opened_utc`, and pinned state on open/save, and trim to `alvs.general.recentLimit` (03 §3.7; 03 §3.9.2).
- **FR-014**: Logging MUST use `tracing` with `tracing-subscriber` (JSON lines) and `tracing-appender` (rolling), at the level of `alvs.advanced.logLevel`, with required fields `ts`, `level`, `target`, `msg` plus `job_id`, `project_id`, `code` when known; logs roll at 10 MB keeping 10 files in the logs folder; media content, caption text, and full user paths never appear at info level (paths at debug only, redacted in diagnostics) (02 §2.11; 03 §3.6, §3.9.8; REQ-REL-003; T-REL-003).
- **FR-015**: A redaction helper MUST redact user file paths in diagnostics export, honoring `alvs.advanced.redactDiagnostics` (default true) (REQ-REL-003; 03 §3.6; T-REL-003).
- **FR-016**: Store errors MUST map to documented codes with their 12 §12.13 severities — E1004 (read-only, newer), E1005 (migration failed), E1007 (locked), E1008/E8001/E8004 (save failures), E8002 (settings restored), E8003 (cache rebuilt), E5003 (write failed), E9005 (path rejected by SEC-04) — surfacing as the 04 §4.9 envelope; every variant maps to exactly one code via `fn code(&self)` (02 §2.11; 12 §12.13).
- **FR-017**: Every file write MUST be contained by the SEC-04 path policy — never outside the project folder, the cache, the user library, or a user-picked folder; device paths and reserved names rejected; UNC paths from a project file require explicit confirmation before access (05 §5.7 SEC-04; `alvs-core::path`).
- **FR-018**: Tests MUST cover the card's acceptance suites — T-PRJ-002 (save/open round-trip, relative fallback, atomic write under simulated failure), T-PRJ-003 (autosave timing and content, injected clock), T-PRJ-004 (backup creation/pruning/restore), T-PRJ-008 (lock create/read-only/stale takeover), T-SET-003 (validation, unknown keys, corruption recovery), T-REL-003 (rotation and redaction), T-REL-007 save-part (disk-full/permission keeps previous files intact) — deterministic, seeded, offline, no wall-clock dependence (09 §9.10 rows; constitution III).
- **FR-019**: Scope boundary — the recovery/backup/settings **dialogs** and banners (DLG-15/16, read-only banner) are UI (WP-05+); export-path disk errors are WP-22; cache **content** writers (thumbnails, waveforms, proxies, analysis) and their LRU eviction, job re-queue, and model downloads belong to their packages (WP-07/08/21); single-instance forwarding and startup orchestration are the app shell (WP-05); the `.alvs` serialization itself and the migration step registry come from `alvs-model` (WP-02) (card Deliverables/Not-included boundaries).

### Key Entities *(this package owns persistence, not the domain model)*

- **Settings document** (03 §3.6): flat `{ settingsVersion, <key>: value }` object; key table, defaults, validation, `.bak`.
- **Cache database** (03 §3.7): `cache.sqlite` with the ten tables + index; SQL migrations versioned in `meta`.
- **Recent project** (03 §3.7): path-keyed entry with name, thumbnail, opened timestamp, pinned flag.
- **Project folder artifacts** (03 §3.8): `<Name>.alvs`, `.alvs.autosave` sidecar, `.alvs.lock`, `Backups\<Name>_<timestamp>.alvs`, `.v<old>.bak` migration backup.
- **App folders** (03 §3.8): settings/library/cache/models/logs/crashes/marker/updates/temp locations, installed or portable.
- **Session marker & log files** (03 §3.8–3.9.8): abnormal-exit detector; rolling JSON-lines logs with redaction.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T-PRJ-002 passes — save/open round-trip is lossless, the relative path is used when the absolute path is missing, and a simulated mid-write failure leaves the previous file intact.
- **SC-002**: T-PRJ-003 passes — autosave fires exactly on the dirty+interval condition with correct content and never writes the main file (NFR-REL-02: at most one autosave interval of work lost, zero corrupted files).
- **SC-003**: T-PRJ-004 passes — backups are created per save, pruned to the keep-count, and restorable into a new file.
- **SC-004**: T-PRJ-008 passes — lock creation, read-only refusal against a live lock, and stale take-over follow 03 §3.9.4.
- **SC-005**: T-SET-003 passes — invalid values revert to defaults with one notice, unknown keys survive, and corruption recovers from the last good backup.
- **SC-006**: T-REL-003 passes — logs rotate at 10 MB × 10 files with the required structured fields and redact user paths.
- **SC-007**: T-REL-007 (save part) passes — disk-full and permission errors during save/autosave/backup/settings writes leave every previous file intact with actionable codes.

## Assumptions

- Dependencies are the sanctioned 00 §0.8 baseline rows: `rusqlite` 0.41.x (feature `bundled`, Data row 202) for the cache database; `thiserror`/`tracing`/`tracing-subscriber`/`tracing-appender` (Core row 204) for errors and logs; plus the existing `alvs-core`/`alvs-model`, serde stack, and `uuid`. Versions resolve at planning from the baseline and lockfile (G-11; no ADR needed for baseline usage).
- A corrupt `settings.json` on first run (no `.bak` yet) falls back to defaults with E8002 — the only possible reading of "restore the last good backup" when none exists (03 §3.6).
- Autosave timing tests use an injected clock/timer trait (deterministic, no wall-clock; constitution III); the production scheduler uses real time.
- The `.alvs` bytes come from `alvs_model` serialization (WP-02); this package writes them. Migration **steps** and the driver come from `alvs-model`; this package owns the physical `.bak` copy, validation of the migrated result, and the size gate — the exact halves WP-02's honesty ledger deferred here.
- DLG-15/DLG-16 and banners render in later packages; this package exposes the data and codes they need (`error.<code>` message keys already cataloged in 12 §12.13).
- Recent projects live in SQLite per 03 §3.7 (not a settings key), trimmed by `alvs.general.recentLimit`; thumbnails in that table are stored as file references written by later packages (nullable until then).
- The workspace gates that already exist (crate rules, clippy `-D warnings`, tests) keep applying to the new crate; the card adds no coverage threshold for this package.
- Portable mode detection is the presence of `portable.flag` next to the executable (03 §3.8); no other mode exists in 1.0.
