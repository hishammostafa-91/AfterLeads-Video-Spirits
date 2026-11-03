# Implementation Plan: WP-03 — Storage and application services

**Branch**: `feat/003-storage-services` | **Date**: 2026-11-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-storage-services/spec.md`; constraints from the owner's `/speckit.specify` invocation (card docs/10 §10.7; requirements REQ-PRJ-002, REQ-PRJ-003, REQ-PRJ-004, REQ-PRJ-011, REQ-SET-003, REQ-REL-001 (services), REQ-REL-003, REQ-REL-007 (save path); tests T-PRJ-002/003/004/008, T-SET-003, T-REL-003, T-REL-007 (save part); read only 03 §3.6–3.9, 02 §2.5/§2.11, 05 SEC-03/04; nothing beyond the canonical documents) **plus the owner's Option-A decision (2026-11-03) resolving the card-vs-CMP-08 conflict via the `DocumentHooks` seam (research D2)**.

## Summary

Deliver the persistence layer exactly as the WP-03 card specifies: the new crate `alvs-store` (CMP-08, internal deps `alvs-core` only) provides application path resolution incl. portable mode (03 §3.8), the table-driven settings store with validation/backup/recovery (03 §3.6), the SQLite cache with the full §3.7 schema, embedded transactional migrations, and corrupt-rebuild (E8003), the recent-projects service, the exact §3.9.1 atomic-write primitive (`FlushFileBuffers` + `MoveFileExW(REPLACE|WRITE_THROUGH)`), the project save/open pipeline generic over a `DocumentHooks` seam with backups, autosave scheduler, lock files, session marker and crash-recovery offer (03 §3.9), and the logging setup (JSON lines, custom 10 MB × 10 size-rotating writer — `tracing-appender` has no size rotation, D3 — plus the redaction helper in `alvs-core`). `alvs-core` gains the seven missing catalog codes (E1007/E1008/E5003/E8001–E8004, D4). The production `DocumentHooks` implementation for `alvs_model::Project` lands as a thin composition module in the app backend (CMP-12), and the whole pipeline is proven inside `alvs-store`'s own tests via a dev-dependency on `alvs-model` (dev-deps are exempt from CMP row checks — owner-approved Option A). No UI, no export path (WP-22), no cache-content writers (WP-07/08/21).

## Technical Context

**Language/Version**: Rust 1.99.1, edition 2024, pinned by `rust-toolchain.toml`; `[lints] workspace = true` picks up the §2.11 clippy posture (unwrap_used/expect_used warn + `-D warnings`).

**Primary Dependencies** (all baseline 00 §0.8 rows; versions verified 2026-11-03, research D1): `rusqlite` 0.41.1 (`bundled`), `tracing` 0.1.44 (locked), `tracing-subscriber` 0.3.23 (`json`, `time` features), `tracing-appender` 0.2.5 (non-blocking path; its new `symlink` transitive is MIT/Apache-2.0, allowlisted), `windows` 0.62.2 (locked; `Win32_Storage_FileSystem`, `Win32_System_Threading`, `Win32_Foundation`), `serde`/`serde_json`/`thiserror`/`uuid` at the locked sibling versions; internal: `alvs-core`; dev: `alvs-model`, `proptest`. Explicitly not used: `tokio` (std threads, D17), `tauri` (CMP-12-exclusive, D6), `time`/`chrono`/`tempfile`/`dirs` (not baseline rows, D5/D6).

**Storage**: this package *is* the storage layer — `.alvs` + sidecar + `Backups\` + `.v<old>.bak` in the project folder; `settings.json`(+`.bak`), `cache.sqlite`, `session.marker`, logs under the app folders (03 §3.8).

**Testing**: the card's seven suites (T-PRJ-002/003/004/008, T-SET-003, T-REL-003, T-REL-007 save part) as named filters + per-module unit tests; deterministic and offline via `Clock`/`Fs` seams (D10/D13) and std temp dirs with uuid names; proptest only for the civil-date math (D5), run with a **fixed seed** (`ProptestConfig` seed pinned, `PROPTEST_SEED` honored in CI) or exhaustive bounded strategies — constitution III's seeded-determinism rule. No coverage threshold is in the card — none invented (D18).

**Target Platform**: Windows 11 x64 — `MoveFileExW`/`FlushFileBuffers`, `OpenProcess` liveness, `CREATE_NEW` exclusivity, `%APPDATA%`/`%LOCALAPPDATA%` roots, long paths via `alvs-core::path`.

**Project Type**: one new library crate (`alvs-store`, CMP-08) in the existing workspace + a thin app-backend module (`app/src-tauri/src/project_io.rs`, CMP-12) + additive `alvs-core` extensions (error codes, redact).

**Performance Goals**: none in the card; the obligations are durability bounds — zero torn files, ≤ one autosave interval of lost work (NFR-REL-02), settings writes debounced 500 ms, DB busy_timeout 5 s.

**Constraints**: every behavior cites 03 §3.6–3.9 / 02 §2.5/§2.11 / 05 SEC-03/04; CMP row compliance enforced by `tools/check_deps.py` (the Option-A seam keeps `alvs-store` at `alvs-core`-only regular deps); licenses via `cargo deny` (deny.toml allowlist verbatim from 05 §5.6); no `unwrap`/`expect` outside tests; AGPL headers (02 §2.14); **one canonical doc change rides this PR (G-19): docs/02 §2.14's `unsafe` allowlist gained `alvs-store` narrowed to the two Win32 groups of D11/D12 (OD-11, owner decision 2026-11-03; docs/00 Set version 1.0.4 → 1.1.0 per §0.11 rule 2)**.

**Scale/Scope**: 14 modules in one new crate + one SQL migration + two touched crates (`alvs-core` additive, app backend thin); ~19 spec FRs; the largest surfaces are the 68-key settings table (§3.6) and the §3.7 DDL.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| I. Canonical documentation first | PASS | Every FR/decision cites 01/02/03/05/09/12 sections; both canonical conflicts were escalated to the owner (G-17) — the card-vs-CMP-08 one resolved as Option A (no doc change), the §2.14 `unsafe` one resolved as OD-11 (scoped doc change in this PR, G-19); versions verified (D1). |
| II. Scope discipline | PASS | FR-019/D16 fence out dialogs (WP-05+), export errors (WP-22), cache-content writers/eviction (WP-07/08/21), job re-queue, models, single-instance, startup orchestration; no 1.1/1.2 items. |
| III. Test-first evidence | PASS | The seven named suites are the acceptance (SC-001..007); determinism via Clock/Fs injection (constitution III's no-wall-clock rule). |
| IV. Protected main and clean history | PASS | Work on `feat/003-storage-services` from `main` `6355f6d`; gates + review before the owner's merge. |
| V. Quality gates non-negotiable | PASS | verify.ps1/CI green with the new crate; dependency-policy and crate-rules pass under the Option-A seam; no gate edits. |
| VI. Global and accessible by default | PASS* | No UI; `error.<code>` keys already cataloged (12 §12.13) for later localization. |
| VII. Private, secure, local-first | PASS | SEC-04 containment on every write (FR-017); SEC-03 size gates before parse; no network, no secrets; paths redacted (REQ-REL-003). |
| VIII. Simplicity and performance budgets | PASS | Table-driven settings instead of a 68-field struct (D7); pure autosave decision core (D10); no tokio/crossbeam (D17); no new direct deps (D1). |
| IX. Open-source integrity | PASS | SPDX headers on all new files; nothing copied (civil-date math implemented here); transitives allowlisted (D1). |
| Technology constraints | PASS | Baseline rows only; toolchain unchanged. |

`*` = passes with the documented reading. No Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/003-storage-services/
├── plan.md              # This file
├── research.md          # Phase 0 — D1..D18 (versions verified 2026-11-03; owner Option-A decision)
├── data-model.md        # Phase 1 — entities→module map, rules, state transitions
├── quickstart.md        # Phase 1 — per-acceptance validation guide
├── contracts/
│   └── crate-apis.md    # Phase 1 — public API of alvs-store + alvs-core additions + guarantees
└── tasks.md             # Phase 2 (/speckit.tasks — not created by /speckit.plan)
```

### Source Code (repository root — what WP-03 writes)

```text
crates/alvs-store/
├── Cargo.toml                    # rusqlite/tracing stack/windows + [lints] workspace; dev alvs-model, proptest
├── migrations/001_init.sql       # 03 §3.7 DDL verbatim (embedded via include_str!)
└── src/
    ├── lib.rs        paths.rs   error.rs      # 03 §3.8; 02 §2.11 store errors
    ├── atomic.rs     settings.rs keys.rs      # 03 §3.9.1; 03 §3.6 table
    ├── cache.rs      recent.rs                # 03 §3.7 + migrations runner + rebuild
    ├── project.rs    autosave.rs backup.rs    # 03 §3.9.2/§3.9.6 pipeline; §3.9.3; REQ-PRJ-004
    ├── lock.rs       session.rs  logging.rs   # §3.9.4; §3.9.5; 02 §2.11 + size rotation
    └── utc.rs                                 # civil-date math (D5)
crates/alvs-core/src/
├── error.rs                      # + E1007/E1008/E5003/E8001–E8004 (D4)
└── redact.rs                     # redact_paths (D15, CMP-01 logging helpers)
app/src-tauri/src/project_io.rs   # DocumentHooks for alvs_model::Project (D2, CMP-12)
crates/alvs-store/tests/          # t_prj_002/003/004/008, t_set_003, t_rel_003/007 + project integration
```

**Structure Decision**: module-per-artifact mirroring 03 §3.8–3.9 with the settings key table as data (D7) and SQL migrations as embedded files (D8); the model seam (D2) puts the only model-aware code in the app backend where CMP-12 allows it, keeping `alvs-store` row-compliant.

## Complexity Tracking

No constitution violations; no entries.

---

## Phase 0 — Research

See [research.md](research.md) — D1..D18 with citations and evidence; **zero unresolved clarifications** (the card-vs-CMP-08 conflict was owner-resolved as Option A, D2). Versions verified against the lockfile, crates.io, docs.rs, and the local registry on 2026-11-03 (G-10). The decisions the reviewer should scrutinize hardest: **D2** (the DocumentHooks seam and its gate-exempt dev-dependency proof), **D3** (custom size-rotating writer — required because tracing-appender rotates by schedule only), and **D12/D13** (the exact §3.9.1 Win32 sequence and its failure-injection seam).

## Phase 1 — Design

- [data-model.md](data-model.md): entity→module map over 03 §3.6–3.9; validation rules 1–6 incl. the lock-semantics table and recovery predicate; state transitions for document, settings, cache, lock, and log rotation.
- [contracts/crate-apis.md](contracts/crate-apis.md): the public surface of `alvs-store` (paths, error, atomic, settings, cache, recent, project+`DocumentHooks`, autosave, lock, session, logging), the additive `alvs-core` surface, the app-backend composition module, and the seven test-enforced guarantees G1–G7.
- [quickstart.md](quickstart.md): validation commands for each acceptance suite plus the gates.

## Constitution Check — re-evaluation after Phase 1 design

Re-checked after the design artifacts: no change. The design introduces no entities beyond 03 §3.6–3.9, no interfaces beyond the canonical services and the one owner-approved seam (D2); deferred items are explicitly owned by later packages (D16). Principles I–IX and the technology constraints remain PASS.
