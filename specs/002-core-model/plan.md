# Implementation Plan: WP-02 — Core types and project model

**Branch**: `feat/002-core-model` | **Date**: 2026-11-02 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-core-model/spec.md`; constraints from the owner's `/speckit.specify` invocation (card docs/10 §10.7; requirements REQ-PRJ-001 (model), REQ-PRJ-005 (model), REQ-PRJ-007 (undo engine), REQ-PRJ-008; tests T-PRJ-005, T-PRJ-006, T-PRJ-007, T-PLY-005; property tests of 09 §9.4; coverage ≥ 80%; cli-free; nothing beyond the canonical documents).

## Summary

Implement the product's data heart exactly as the WP-02 card specifies: `alvs-core` gains exact time (flicks with the literal 03 §3.3.1 rate table, rationals, drop-frame timecode per REQ-PLY-005), UUID v7 IDs, the SEC-04 path policy as a decision type, and the error model (per-crate `thiserror` enums mapping to 12 §12.13 codes + the 04 §4.9 envelope). `alvs-model` gains every 03 §3.4 type with `extra` maps (included in the TS surface via ts-rs's `serde-json-impl` feature — owner decision) and 03 §3.2 serialization, full 03 §3.5 validation incl. malicious-size limits, the `EditOp` enum with application for the card's categories (project/sequence/track/media/bin/marker/caption-track + batch), ADR-009 undo/redo with Arc roots, a deterministic 512 MB estimated-size budget over the ≥ 200-step guarantee, `EntityPatch` diffing, JSON Schema generation, the forward-only migrations framework with v1 fixtures, and committed ts-rs bindings — **plus the CI bindings-drift step that WP-01's contract assigned to this package** (rust-lint: export test + `git diff --exit-code`) and the §2.11 clippy lint posture (`[workspace.lints]` unwrap_used/expect_used). No persistence (WP-03), no clip-editing semantics (WP-12), no UI.

## Technical Context

**Language/Version**: Rust 1.99.1, edition 2024, pinned by `rust-toolchain.toml`; `#![deny(missing_docs)]` already on both crates (docs/02 §2.4 rule 4) and kept green.

**Primary Dependencies** (all baseline 00 §0.8; versions verified via crates.io 2026-11-02, research D1): `serde` 1.0.229, `serde_json` 1.0.154, `schemars` 1.2.3, `ts-rs` 12.0.1, `uuid` 1.29.0 (feature `v7`), `thiserror` 2.0.21; dev `proptest` 1.12.0. Explicitly not used: `arc-swap` (engine-side), `insta` (JSON fixtures suffice), `tracing` (WP-03).

**Storage**: none in this package — the `.alvs` file *format types* only; reading/writing/backup files are WP-03 (03 §3.9 lifecycle).

**Testing**: 09 §9.4 rows for `alvs-core` and `alvs-model`; the card's named suites (T-PRJ-005/006/007, T-PLY-005); proptest 10,000×50 at CI scale (env-tunable for the future nightly scale); coverage via `cargo llvm-cov` ≥ 80% (measured and recorded; standing CI threshold deferred, research D14).

**Target Platform**: Windows 11 x64 (product); crates are platform-pure Rust — the path policy encodes Windows device-path/reserved-name/UNC rules (SEC-04) and is unit-tested on Windows.

**Project Type**: two library crates in the existing Cargo workspace (CMP-01/CMP-02, docs/02 §2.4).

**Performance Goals**: deterministic budgets, not runtime speed: undo ≥ 200 steps / ≤ 1,000 / 512 MB estimated; flicks arithmetic is exact integer math by construction. NFR-PERF targets attach to playback/render, not this package.

**Constraints**: every type/rule/invariant cites 03 §3.2–3.5; EditOp per 04 §4.4 with the card's category scoping (spec FR-009/FR-010; `batch` included as a canonical composite of those categories — interpretation recorded in research D8); errors per 02 §2.11 + 12 §12.13 incl. the **lint posture (workspace lints `unwrap_used`/`expect_used` warn; `-D warnings` denies — research D17)**; envelope per 04 §4.9; DTOs/bindings per 04 §4.2 + 03 §3.2.6 with `extra` included in the TS surface (`serde-json-impl`, research D2/D16) and the **CI drift step added to `rust-lint` as WP-01's contract assigns (research D15)**; no `unwrap`/`expect` outside tests (02 §2.11); no new dependencies beyond the baseline rows (G-11); `cli`-free (card); AGPL headers and LF per 02 §2.14.

**Scale/Scope**: 24 modules across two crates (5 core + 19 model, research D3); the largest surface is the 03 §3.4 type set (~13 entity groups) and the validation rule set (15 groups); ~9 EditOp categories applied plus two repository-infrastructure touches assigned to this package: the CI bindings-drift step (`.github/workflows/ci.yml` `rust-lint`) and the workspace lint table (root `Cargo.toml`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| I. Canonical documentation first | PASS | Every FR/decision cites 01/02/03/04/05/09/12 sections; the plan adds no requirements; versions verified against the baseline (D1). |
| II. Scope discipline | PASS | FR-010/FR-019 fence out WP-03/WP-12/WP-13 (+ caption-item ops, effects/masks/transitions application); only the card's EditOp categories are applied. |
| III. Test-first evidence | PASS | The four named suites + 09 §9.4 duties are the acceptance (SC-001..006); literal-table and property tests turn the canonical numbers into executable contracts. |
| IV. Protected main and clean history | PASS | Work on `feat/002-core-model` from `main`; gates + review before merge. |
| V. Quality gates non-negotiable | PASS | verify.ps1/CI green incl. the newly activated bindings-drift gate; coverage measured ≥ 80%; no gate edits. |
| VI. Global and accessible by default | PASS* | No UI in this package; `history.<op>`/`error.<code>` keys are named for the UI packages to localize. |
| VII. Private, secure, local-first | PASS | SEC-04 path policy implemented and tested; THR-02 malicious-size limits enforced; no network, no secrets. |
| VIII. Simplicity and performance budgets | PASS | Plain `Arc` history (no speculative arc-swap), deterministic size estimation, entity-per-module mirroring the canonical structure; no unneeded deps (D1/D2/D9). |
| IX. Open-source integrity | PASS | SPDX headers already on the crates; new files carry them; nothing copied. |
| Technology constraints | PASS | Baseline dependency rows only (D1); toolchain unchanged. |

`*` = passes with the documented reading. No Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/002-core-model/
├── plan.md              # This file
├── research.md          # Phase 0 — D1..D16 (versions verified 2026-11-02)
├── data-model.md        # Phase 1 — entity→module map, validation, state transitions
├── quickstart.md        # Phase 1 — validation guide per acceptance item
├── contracts/
│   └── crate-apis.md    # Phase 1 — public API of alvs-core / alvs-model
└── tasks.md             # Phase 2 (/speckit.tasks — not created by /speckit.plan)
```

### Source Code (repository root — what WP-02 writes)

```text
crates/alvs-core/src/
├── time.rs        # Flicks, frame math (03 §3.3.1), Timecode (REQ-PLY-005)
├── rational.rs    # reduced rationals (03 §3.3.2)
├── id.rs          # UUID v7 (03 §3.2.2)
├── path.rs        # SEC-04 path policy (05 §5.7)
└── error.rs       # ErrorCode, AppError, ErrorEnvelope (02 §2.11; 04 §4.9)
crates/alvs-model/src/
├── project.rs media.rs bin.rs asset.rs sequence.rs track.rs
├── effect.rs mask.rs title.rs transition.rs marker.rs caption.rs   # 03 §3.4 types
├── edit.rs        # EditOp + application (04 §4.4, card categories)
├── history.rs     # ADR-009 undo/redo + budget
├── patch.rs       # EntityPatch + Arc::ptr_eq diffing
├── validate.rs    # 03 §3.5
├── migrate.rs     # 03 §3.9.6 framework + registry
├── dto.rs         # TS DTO surface (04 §4.2)
└── schema.rs      # JSON Schema generation (ADR-008)
crates/alvs-model/tests/fixtures/migrations/   # v1 input/output pairs
app/src/ipc/bindings/                          # committed generated TS types
.github/workflows/ci.yml                       # + bindings-drift steps in rust-lint (WP-01 contract assigns to WP-02)
Cargo.toml Cargo.lock                          # dependency additions (baseline rows) + [workspace.lints] (02 §2.11)
```

**Structure Decision**: entity-per-module mirroring 03 §3.4's sections with behavior modules for the card's deliverables (research D3); fixtures under the crate's `tests/` per Cargo convention; bindings land in the canonical UI path (04 §4.2) so the WP-01 drift gate activates by target presence.

## Complexity Tracking

No constitution violations; no entries.

---

## Phase 0 — Research

See [research.md](research.md) — D1..D16 with citations; **zero NEEDS CLARIFICATION**; dependency versions and the schemars/ts-rs API anchors verified against crates.io and docs.rs on 2026-11-02 (G-10); drop-frame per SMPTE ST 12-1 (D5); the deterministic undo-budget design (D9) and the patch dirty-range policy (D10) are the two design decisions the reviewer should scrutinize hardest.

## Phase 1 — Design

- [data-model.md](data-model.md): entity→module map; the 03 §3.5 validation set; state transitions for `Model`, `History`, and file lifecycle; relationships.
- [contracts/crate-apis.md](contracts/crate-apis.md): the public surface of both crates — the time/rational/id/path/error primitives, the EditOp/History/Patch/validate/migrate/schema APIs — plus the four test-enforced behavioral guarantees.
- [quickstart.md](quickstart.md): validation commands for each acceptance item (§1–§8), incl. the bindings-drift activation and the ≥ 80% coverage measurement.

## Constitution Check — re-evaluation after Phase 1 design

Re-checked after the design artifacts: no change. The design introduces no entities beyond 03 §3.4, no interfaces beyond 04 §4.2/4.4/4.9 and the library APIs the card's deliverables name; the deferred items (WP-03 persistence, WP-12 clip semantics, WP-13 panel, nightly-scale proptest, CI coverage threshold) are explicitly owned. Principles I–IX and the technology constraints remain PASS.
