# WP-03 Evidence — Storage and application services

All commands run on the `feat/003-storage-services` branch (workstation: Windows 11 x64, Rust 1.99.1). Acceptance suites use the card's filters; every run is deterministic and offline (injected clocks/filesystem seams; constitution III).

## Gate table (final state)

| Command | Result |
|---|---|
| `pwsh -File tools/verify.ps1` | **All available gates PASS** — docs, commits, rust-format, rust-lint, dependency-policy, crate-rules, placeholders, rust-test, bindings-drift (UI gates SKIPPED: scripts arrive with WP-04) |
| `python tools/git/check_commits.py origin/main..HEAD` | **all commits OK** |
| `cargo nextest run --workspace --locked` | **281/281 pass** (31 alvs-core + 246 alvs-store + 4 app backend) |
| `cargo test --workspace --locked` (the `rust-test` gate) | **PASS** — 40 test binaries ok |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **clean** |
| `cargo fmt --all -- --check` | **clean** |

## Acceptance suites (card)

| # | Suite | Command | Result |
|---|---|---|---|
| 1 | T-PRJ-002 save/open round-trip, relative fallback, atomic write under simulated failure | `cargo nextest run -p alvs-store --locked t_prj_002` | **7/7 pass** — lossless round-trip with recent-recording + sidecar cleanup; relative path takes over when the absolute misses (both-missing stays offline); 256 MB+1 sparse document rejected before parse (E1003); `schemaVersion`-newer opens read-only (E1004) with unknown fields kept; synthetic v0→v1 migration writes `.v0.bak` with the ORIGINAL bytes (fixtures are real projects — the WP-02 driver validates migrated results); failed migration reports E1005 with the original untouched; real read-only-target save failure keeps the previous bytes (E8004) |
| 2 | T-PRJ-003 autosave timing and content | `cargo nextest run -p alvs-store --locked t_prj_003` | **4/4 pass** — fires exactly on dirty ∧ elapsed ≥ interval (fake clock: 59 s skip / 60 s write / no double-write / 15 s minimum row), main file never touched, sidecar content verified, spawned production loop writes and stops on drop; session-marker lifecycle + recovery offer compares timestamps (stale sidecar → no offer) with both ISO stamps |
| 3 | T-PRJ-004 backup creation, pruning, restore | `cargo nextest run -p alvs-store --locked t_prj_004` | **2/2 pass** — five saves hold exactly `keep` = 3 with the newest carrying the last save; list exposes time + size; restore copies the OLDEST into a new file (untouched backup) that re-opens as a project |
| 4 | T-PRJ-008 lock create, read-only, stale takeover | `cargo nextest run -p alvs-store --locked t_prj_008` | **4/4 pass** — exclusive create; live same-host holder → read-only (E1007, never a silent seizure); verifiably dead holder → take-over (real Win32 probe against a verified-free pid — the spawn/kill approach from analyze I2 proved flaky under pid recycling, replaced with equivalent deterministic coverage: probe-true against our live pid, probe-false against a verified-free pid, pure rules in lock.rs); other-host 24 h boundary; the pipeline surfaces both outcomes and a confirmed take-over re-opens writable |
| 5 | T-SET-003 settings validation, unknown keys, corruption recovery | `cargo nextest run -p alvs-store --locked t_set_003` | **6/6 pass** — invalid range/enum/step values revert to defaults with one collective E8002; unknown keys survive the flush; corrupt → `.bak` restore; corrupt without `.bak` → defaults; `settingsVersion` 2 treated as corrupt; rapid changes coalesce into one debounced write carrying the last value; typed getters pinned |
| 6 | T-REL-003 log rotation and redaction | `cargo nextest run -p alvs-store --locked t_rel_003` | **4/4 pass** — 96 MB through the real 10 MB × 10 contract keeps ≤ 10 files with `app.log.9` present and no file past limit+line; canonical JSON lines (`ts` Z-suffixed, `level`, `target`, `msg`, `job_id`/`project_id`/`code` carried); trace filtered at info; debug emitted at debug; redacted path line never leaks the raw form; global install smoke |
| 7 | T-REL-007 (save part) disk-full/permission errors keep previous files intact | `cargo nextest run -p alvs-store --locked t_rel_007` | **5/5 pass** — read-only project file (no side effects, short-circuited recent, E8004); `.tmp`-directory collision (E8004); vanished folder (E1008); read-only sidecar autosave failure (main untouched); read-only settings target (previous file intact). The step-injected disk-full cases live in `atomic.rs` unit tests (E8001/E1008/E8004 at all three §3.9.1 steps) per review F-05's structure |

## Supporting suites

| Suite | Command | Result |
|---|---|---|
| Cache database (US6) | `cargo nextest run -p alvs-store --locked cache_` | **3/3 pass** — §3.7 pragmas (per-connection ones read through the cache's own connection), schema version 1 + idempotent reopen, ten tables + `jobs_by_state`, the proxies state CHECK enforced, corrupt garbage + stale WAL rebuilt with evidence kept (E8003 flag), recent record/list/trim/upsert/pin-survives at limit 2 |
| Settings wiring (T029) | `cargo nextest run -p alvs-store --locked settings_values` | **1/1 pass** — the typed getters drive autosave/backup/recent/logging/redaction |
| SEC-04 containment (T029, G7) | `cargo nextest run -p alvs-store --locked sec04` | **2/2 pass** — §3.8 writer targets Allowed within `containment_roots`; project artifacts are siblings of the user-chosen file (Allowed with the project folder as root) |
| Model integration (D2 seam) | `cargo nextest run -p alvs-store --locked t_prj_002` | covered by suite 1 (hooks over the real `alvs_model::Project` via the dev-dep; the production hooks ship in `app/src-tauri/src/project_io.rs`, CMP-12) |

## Construction notes (implementation-time discoveries, all resolved within the approved plan)

- **FlushFileBuffers needs a write handle**: a read-only `File::open` + `sync_all` returns ERROR_ACCESS_DENIED — the §3.9.1 flush opens with write access (found by a step diagnostic, fixed in T007).
- **`fs::rename` lacks WRITE_THROUGH** (verified in std source) → the `windows` crate `MoveFileExW` call is required (OD-11's first unsafe group); HRESULT_FROM_WIN32 codes are decoded back to raw Win32 errors for the E8001/E8004/E1008 mapping.
- **Corrupt SQLite headers fail the WAL pragma** before any `quick_check` — `SQLITE_NOTADB`/`SQLITE_CORRUPT` at connect now route to the rename-and-rebuild path.
- **Per-connection pragmas** (`foreign_keys`, `synchronous`) are invisible to a different connection — verified through a `Cache::pragma` accessor.
- **Backup listing order** uses the name stamp (`yyyyMMdd-HHmmss`, parsed back to epoch ms), not mtime: same-millisecond writes would otherwise tie. `prune` removes the oldest tail (an early inverted cut was caught by the suite and fixed).
- **tracing-appender 0.2.5 rotation is schedule-only** (registry-verified) — the custom `SizeRotatingWriter` implements the canonical 10 MB × 10 numbers; the crate still supplies the non-blocking path.
- **The default JSON formatter nests `msg` under `fields`** — a custom `FormatEvent` writes the §2.11 canonical fields exactly.
- **`cargo test` shares a process per binary** — the two logging field tests use `tracing::subscriber::with_default` (thread-local) with a single global-install smoke, so both gate runners pass.
- **Test-name filters**: nextest matches test names, so every suite test carries its prefix (`t_prj_002_…`) — the documented filters select them exactly.

## Honesty ledger (not verified / deferred)

- The production `DocumentHooks` composition (`app/src-tauri/src/project_io.rs`) compiles and is row-legal, but no app-level caller drives it yet (WP-05 wires it to the shell); its behavior is proven by the identical test-side hooks.
- The migration registry is empty at `schemaVersion = 1` (by design, WP-02); `.v<old>.bak` + E1005 paths are proven with synthetic steps.
- Cache **content** writers (thumbnails/waveforms/proxies/analysis) and LRU eviction belong to WP-07/08/21 — this package delivers the schema, migrations, integrity rebuild, and recent projects (research D16).
- The diagnostics-bundle export that *applies* `redact_paths` at export time is WP-27; the helper and its gating flag ship here.
- Nightly-scale property testing does not exist for this package; `utc.rs` uses deterministic strided sweeps + known-date tables instead of random cases (constitution III's determinism rule made the RNG unnecessary).
- `settings.json` key order is the map's alphabetical order (03 §3.2 pins the project-file form; the settings document order is unspecified).
