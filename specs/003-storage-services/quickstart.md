# WP-03 Quickstart — validation guide

Prerequisites: dev workstation per `tools/setup/Test-DevEnvironment.ps1` green; branch `feat/003-storage-services`. Every command runs from the repository root. Expected outcomes cite the acceptance suite (card) they prove; shapes in [data-model.md](data-model.md), APIs in [contracts/crate-apis.md](contracts/crate-apis.md).

## 1. Full crate suite (all seven acceptance suites in one run)

```powershell
cargo nextest run -p alvs-store --locked
```

**Expected**: all tests pass, including the seven named filters below; no test touches the network, the wall clock (injected `Clock`/`Fs` seams), or folders outside its per-test temp dir.

## 2. Per-acceptance validation

| # | Suite | Command | Expected outcome (spec SC) |
|---|---|---|---|
| 1 | T-PRJ-002 save/open | `cargo nextest run -p alvs-store --locked t_prj_002` | round-trip lossless via `ProjectHooks` (dev-dep `alvs-model`); relative media path used when absolute missing; injected tmp/flush/replace failures leave the previous file intact |
| 2 | T-PRJ-003 autosave | `cargo nextest run -p alvs-store --locked t_prj_003` | fires exactly on dirty + interval (fake clock), content = snapshot, main file untouched |
| 3 | T-PRJ-004 backups | `cargo nextest run -p alvs-store --locked t_prj_004` | per-save `Backups\<Name>_<yyyyMMdd-HHmmss>.alvs`, prune to `backupsKeep`, restore copies into a new file |
| 4 | T-PRJ-008 locks | `cargo nextest run -p alvs-store --locked t_prj_008` | exclusive create; live lock → E1007 read-only; dead pid → takeover; other host < 24 h → no takeover |
| 5 | T-SET-003 settings | `cargo nextest run -p alvs-store --locked t_set_003` | invalid → default + one E8002; unknown keys preserved; corrupt → `.bak` restore (no `.bak` → defaults) |
| 6 | T-REL-003 logs | `cargo nextest run -p alvs-store --locked t_rel_003` | rotation at 10 MB, ≤ 10 files, JSON `ts/level/target/msg`, paths redacted |
| 7 | T-REL-007 (save part) | `cargo nextest run -p alvs-store --locked t_rel_007` | disk-full (injected) and permission failures (pre-created `.tmp` directory; read-only target file at the replace step) on save/autosave/backup/settings keep previous files intact with E8001/E8004/E1008 |

Filters are name prefixes on the test functions; the exact test names appear in `tasks.md`.

## 3. Integration with the model (D2 seam proof)

```powershell
cargo nextest run -p alvs-store --locked project   # save/open pipeline against real alvs-model Project
```

**Expected**: the `DocumentHooks` implementation over `alvs_model::Project` (a) round-trips a current `schemaVersion = 1` WP-02 fixture with **no** migration (the registry is empty at v1 — the only registered version), and (b) opens a `schemaVersion = 2` fixture **read-only** with E1004. The physical `.v<old>.bak` write and E1005-on-failure path are exercised through a **test-local** `DocumentHooks` carrying synthetic versions and migration steps (legal: the WP-02 driver takes caller-supplied steps), since no real step exists until the version that first bumps the schema.

## 4. Gates

```powershell
pwsh -File tools/verify.ps1        # all available gates PASS (incl. dependency-policy on the new deps
                                    #   and crate-rules on the new crate)
python tools/git/check_commits.py origin/main..HEAD
```

**Expected**: `docs`, `commits`, `rust-format`, `rust-lint`, `dependency-policy`, `crate-rules`, `placeholders`, `rust-test`, `bindings-drift` all PASS; UI gates remain SKIPPED (WP-04). `cargo deny check` passes with `rusqlite`/`tracing-appender` (+ `symlink` transitive) on the allowlist.

## 5. Manual smoke (optional, local only)

Run the app-crate composition module's doc-test/example: save a `Project` to a temp folder via `project_io::ProjectHooks`, kill the process without `SessionMarker::clear`, re-evaluate → recovery offer present with both timestamps. This is the same path WP-05 will wire to DLG-15.
