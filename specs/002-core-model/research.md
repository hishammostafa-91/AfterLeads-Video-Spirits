# Research: WP-02 — Core types and project model

**Date**: 2026-11-02 · **Inputs**: WP-02 card (docs/10 §10.7); docs/01 REQ-PRJ-001/005/007/008 (+REQ-PLY-005 via T-PLY-005); docs/02 ADR-008/009, §2.4, §2.11; docs/03 §3.2–3.5, §3.9.6; docs/04 §4.2, §4.4, §4.9; docs/12 §12.8–12.9, §12.13; docs/09 §9.4, §9.10; docs/00 §0.8; docs/05 §5.7 SEC-04/THR-02; repository state (G-10); crates.io and docs.rs (queried 2026-11-02).

No NEEDS CLARIFICATION markers remain. Every externally sourced fact below was verified on 2026-11-02.

## D1 — Dependency versions (verified via crates.io, 2026-11-02) and pinning strategy

- **Decision**: manifest constraints pinned to these exact stable versions (Cargo.lock commits the resolution):

| Crate | Baseline (00 §0.8) | Verified max stable |
|---|---|---|
| serde / serde_json | 1.x / 1.x | 1.0.229 / 1.0.154 |
| schemars | 1.x | 1.2.3 |
| ts-rs | 12.x | 12.0.1 |
| uuid | 1.x | 1.29.0 |
| thiserror | 2.x | 2.0.21 |
| proptest (dev) | 1.x | 1.12.0 |

- **Rationale**: all rows are the sanctioned Data/Core/Tests baseline (G-11; no ADR needed); full-version constraints plus the committed lockfile give reproducible builds. API anchors verified on docs.rs: schemars 1.2 exposes `schema_for!` + `JsonSchema` derive; ts-rs 12's export API is `TS::export`/`TS::export_all`/`TS::export_to_string` + the `#[ts(export_to = "…")]` attribute and `TS_RS_EXPORT_DIR` (details in D15; exact calls verified against the vendored source per the anti-hallucination rule).
- **Not used** (with reasons): `arc-swap` — the undo history needs plain `Arc<Project>` roots; arc-swap is the engine-side swap container (ADR-009's "thread-safe sharing with the engine") and enters with the engine work packages; `insta` — JSON fixtures under version control cover the migration/schema snapshot needs; `tracing` — logging setup is a WP-03 deliverable, not in the WP-02 card. `uuid` carries the `v7` feature (03 §3.2.2).

## D2 — Crate dependency graphs (check_deps.py-compliant, 02 §2.4)

- **Decision**: `alvs-core`: `serde` (derive), `serde_json`, `thiserror`, `uuid` (v7); dev `proptest`. `alvs-model`: `alvs-core` plus `serde` (derive), `serde_json`, `schemars`, `ts-rs` **with the `serde-json-impl` feature**, `thiserror`, `uuid` (v7); dev `proptest`. The ts-rs feature is required by the review F2 decision: the persisted structs carry `#[serde(flatten)] extra: BTreeMap<String, serde_json::Value>` unconditionally (03 §3.2.3), and `serde_json::Value` implements `TS` only under that feature — without it the DTO derives do not compile, and with it the flattened maps surface in TypeScript as `T & { [key: string]: JsonValue }` (unknown-field forward-compatibility visible to the UI, matching the plan's intent that `extra` is included, not skipped; owner decision 2026-11-03).
- **Rationale**: CMP-01/CMP-02 rows (model depends only on core + its own baseline externals); every external is a baseline row; exclusive crates untouched; `#![deny(missing_docs)]` already present (kept green by documenting the new public APIs).

## D3 — Module layout

- **Decision**:
  - `crates/alvs-core/src`: `time.rs` (flicks, frame math, timecode), `rational.rs`, `id.rs` (UUID v7 mint/parse), `path.rs` (SEC-04 policy), `error.rs` (`ErrorCode`, `AppError`, `ErrorEnvelope`).
  - `crates/alvs-model/src`: `project.rs` (top level, settings, view state), `media.rs`, `bin.rs`, `asset.rs`, `sequence.rs`, `track.rs` (track, clip, intrinsic, audio object), `effect.rs` (instance, animatable values, keyframes), `mask.rs`, `title.rs`, `transition.rs`, `marker.rs`, `caption.rs`, `edit.rs` (`EditOp`, `ParamTarget`, application), `history.rs` (undo/redo engine), `patch.rs` (`EntityPatch`, diffing), `validate.rs` (03 §3.5), `migrate.rs` (03 §3.9.6 framework + registry), `dto.rs` (TS-exporting DTO surface, 04 §4.2), `schema.rs` (JSON Schema generation).
- **Rationale**: entity-per-module mirrors 03 §3.4's sectioning; behavior modules mirror the card's deliverable list; `dto.rs` exists because 04 §4.2 pins DTOs to `alvs-model::dto`.

## D4 — Time: flicks and frame math (03 §3.3.1)

- **Decision**: `Flicks(pub i64)` newtype with `FLICKS_PER_SECOND = 705_600_000`; frame duration computed from the rational rate; a test asserts the doc's literal per-rate table (23.976→29,429,400 … 60→11,760,000; 48 kHz→14,700; 44.1 kHz→16,000) so drift from the canonical numbers is impossible; custom-rate validity per FR-001; round-half-to-even for foreign-rate conversion; 24-hour bound constant.
- **Rationale**: 03 §3.3.1 verbatim; literal-table tests turn the doc into an executable contract.
- **Alternatives**: computing the table at runtime only (no literal test) — rejected: a rounding regression would be invisible.

## D5 — Timecode incl. drop-frame (REQ-PLY-005, T-PLY-005)

- **Decision**: `alvs-core::time::Timecode` formats/parses `HH:MM:SS:FF` and `HH;MM;SS;FF` (drop-frame `;` at 29.97/59.94 per 12 §12.9), frames (`FF` count) and seconds forms; parsing accepts absolute and `+`/`-` relative values and whitespace-trimmed input; drop-frame follows SMPTE ST 12-1 (drop the first 2 frame numbers per minute at 30000/1001, 4 at 60000/1001, except every 10th minute; 24-hour wrap). Format∘parse round-trips every representable value at every catalog rate (property test).
- **Rationale**: REQ-PLY-005 names the forms and the `;` convention at the drop-frame rates of 12 §12.9 (FPS-04, FPS-07); SMPTE ST 12-1 is the industry definition of drop-frame.
- **Alternatives**: storing timecodes as strings — rejected (03 §3.3.1: flicks is the representation; timecode is display-only).

## D6 — Path policy (05 §5.7 SEC-04)

- **Decision**: `alvs-core::path` exposes a pure check API: `check_path(candidate, roots) -> PathDecision` with `PathDecision::Allowed | NeedsConfirmation(Unc) | Denied(AppError)`; checks = canonicalization, Windows device-path rejection (`\\.\`, `\\?\GLOBALROOT`), reserved-name rejection (CON, NUL, AUX, PRN, COM1–9, LPT1–9, case-insensitive, with-extension), write containment (project folder, cache, user library, user-picked roots — supplied by the caller as validated roots), and UNC (`\\server\share`) from project files → `NeedsConfirmation` (the UI dialog is WP-05+; this package returns the decision).
- **Rationale**: SEC-04 verbatim; a decision-type keeps the policy pure and testable (09 §9.4 lists it under alvs-core tests) without pulling UI or WP-03 path management in.
- **Alternatives**: boolean API — rejected: loses the confirmation contract SEC-04 explicitly requires for UNC.

## D7 — Error model (02 §2.11; 12 §12.13; 04 §4.9)

- **Decision**: `alvs-core::error` defines `ErrorCode` (enum with the codes this package can raise — at minimum E0001, E0002, E1003, E1004, E1005, E1006, E1102, E1104, E2001 — each with its catalog message string as documentation), `AppError` (`thiserror` enum; every variant maps to exactly one code via `fn code(&self)`), and `ErrorEnvelope` (`code`, `severity`, `messageKey = "error.<code>"`, `params: BTreeMap`, `detail`, `retryable`, `recovery: Vec<Recovery>` with `commandId` + `labelKey`) exactly per 04 §4.9. New variants for later crates extend their own enums mapping into codes.
- **Rationale**: 02 §2.11 ("each crate defines thiserror enums; every variant maps to exactly one code") + the envelope shape is normative JSON in 04 §4.9.
- **Alternatives**: one giant error enum per crate — rejected: 02 §2.11 prescribes per-crate enums mapped to shared codes.

## D8 — `EditOp` application architecture (04 §4.4)

- **Decision**: `Model::new(root)` holds `Arc<Project>` + `History`; `apply(&mut self, op: &EditOp) -> EditResult`; application = clone-on-write through `Arc::make_mut`, then `validate`; a violated invariant restores the prior root and returns E0002 semantics; every variant carries `fn history_key(&self) -> &'static str` (`"history.<op>"`); `batch` applies inner ops atomically and records one history step — *interpretation record (review F9)*: `batch` is not one of the card's seven category names, but it is a canonical op (04 §4.4) whose inner ops are exactly those categories; including it is a composite of sanctioned categories, not new semantics; `removeMedia` enforces E1102 unless `alsoRemoveClips`; `updateSequence` frame-rate change snaps clip starts/durations to the new frame grid and returns the adjusted-item list (E1104 data).
- **Rationale**: ADR-009 (new root per edit) + 04 §4.4 semantics + 03 §3.5 rollback rule; ops applied in the card's categories only (spec FR-009/FR-010).
- **Alternatives**: inverse-command undo — rejected (ADR-009's own rejected alternative).

## D9 — Undo/redo engine and the memory budget (ADR-009; REQ-PRJ-007)

- **Decision**: `History` keeps `undo: Vec<Arc<Project>>`, `redo: Vec<Arc<Project>>`, and a deterministic **estimated-size account**: `estimate_size(&Project) -> usize` (per-entity structural estimate: fixed per-entity base + payload lengths such as name/notes/text) summed per root; the budget = 512 MB of *estimated* size with oldest-first eviction above it, but never evicting below **200** entries (the guarantee is unconditional; 1,000 is the cap on retained entries). `undo()/redo()/step(i)/steps()` expose the History-panel needs (jump = truncate+apply chain, WP-13 consumes). Tests assert eviction order, the ≥200 floor, and the 1,000 cap with synthetic roots of computed size (deterministic — no allocator sampling).
- **Rationale**: REQ-PRJ-007's "at least 200 guaranteed, up to 1,000 within 512 MB" needs a measurable budget; deterministic estimation makes the threshold testable and platform-independent, which real allocation measurement is not. ADR-009's roots-between-edits design makes eviction trivially safe (roots are immutable snapshots).
- **Alternatives**: measuring live heap (e.g. allocation counters) — rejected: non-deterministic across platforms/allocators, untestable at exact boundaries.

## D10 — `EntityPatch` diffing (04 §4.4; ADR-009)

- **Decision**: `diff(old: &Arc<Project>, new: &Arc<Project>) -> EntityPatch` walks the ten entity kinds with `Arc::ptr_eq` short-circuit per entity (ADR-009's "patches computed by Arc::ptr_eq diffing"); `revision` increments per applied op; `dirtyRanges` by change kind for this package's ops: sequence/track structural changes → the full extent of that sequence; marker add/remove → a zero-length range at the marker time; media/bin/project-level changes → no ranges. (`remove.media` etc. keep full-extent for their sequence when tracks/clips are removed via `alsoRemoveClips`.)
- **Rationale**: 04 §4.4 defines the patch shape and ADR-009 the mechanism; the *range policy for non-clip ops is this package's own conservative engineering decision* (review F4 — no canonical document assigns granular ranges to these ops): structural changes invalidate a whole sequence; marker changes are point events; media/bin/project changes alter no timeline pixels. Clip-extent precision arrives with clip ops (WP-12), which is also the scale at which the engine's dirty-range consumer (09 §9.4, alvs-engine row) begins to matter.
- **Alternatives**: empty dirtyRanges for all WP-02 ops — rejected: track add/remove visibly changes the whole timeline; canonical citation for a finer policy — none exists (verified by review).

## D11 — Validation (03 §3.5)

- **Decision**: `validate(&Project) -> Vec<Issue>` with `Issue { code: ErrorCode, path: String, detail: String }`, implementing all fifteen rule groups of 03 §3.5, including the size limits (3.5.15) checked first (cheap, pre-traversal); `Model::apply` runs it after every op under `debug_assertions` and in tests (03 §3.5); load-time use is exposed for WP-03.
- **Rationale**: 03 §3.5 verbatim ("checked on load and after every EditOp in debug builds and tests"); issue-per-rule keeps T-EDT-011 (WP-12) and this package's fixtures sharp.
- **Alternatives**: boolean validate — rejected: E0003/E1003 reporting and tests need per-rule detail.

## D12 — Migrations (03 §3.9.6; REQ-PRJ-008; T-PRJ-007)

- **Decision**: `migrate.rs` provides a step registry `Vec<MigrationStep>` where `MigrationStep { from: u32, to: u32, run: fn(&mut serde_json::Value) -> Result<(), AppError> }`, a driver that walks `schemaVersion → current` forward-only, validates the migrated value, and a `MigrationOutcome` that tells the caller (WP-03) to write `<Name>.alvs.v<old>.bak` first (the framework exposes the backup contract; file writing is WP-03); `schemaVersion > current` → read-only outcome with E1004 (unknown fields kept by the `extra` maps). Release 1.0 ships `schemaVersion = 1`; the *shipped* registry is empty (no synthetic version bump — `schemaVersion 2` is not a canonical fact, G-02/G-10), and the driver's step-running, backup-signal, and E1005-failure paths are exercised by **test-registered synthetic `MigrationStep`s under `cfg(test)`** (fixtures supply the input/output JSON; the tests register throwaway steps that mutate the value, so every driver path executes without shipping a fake version — review F5). Fixtures live in `crates/alvs-model/tests/fixtures/migrations/` as input/output JSON pairs.
- **Rationale**: 03 §3.9.6 verbatim ("every migration has fixtures (input/output pairs) and a test"); test-registered steps keep T-PRJ-007's "migrations" coverage and the ≥ 80% coverage acceptance real while refusing an invented shipped version.
- **Alternatives**: shipping a demo v2 — rejected: invents a version that no canonical document defines.

## D13 — Property testing (09 §9.4)

- **Decision**: `proptest` strategies generate small random projects (bounded entity counts) and random op sequences drawn from the implemented WP-02 op set; each op either applies (then invariants must hold) or rejects (documented codes only); CI-scale = 10,000 sequences × 50 ops; the case count is env-tunable (`ALVS_PROPTEST_CASES`) so the nightly 1,000,000-scale run activates when the nightly full-matrix job exists (WP-01 scoped nightly to audit + links; the proptest job is not in WP-02's scope to add). **The measured duration of the CI-scale suite is recorded in the PR evidence** (review F10) against the ≤ 25-minute PR-CI budget it shares (NFR-MNT-02); the env knob exists for the nightly *increase* only — reducing the CI scale with it would weaken a gate (G-08) and is not a sanctioned use.
- **Rationale**: 09 §9.4 alvs-model row; env-tunability is a mechanism, not a scope change; the runtime estimate (bounded projects, CoW apply, ~500k ops) is plausible but measured, not assumed.
- **Alternatives**: fixed-seed fuzz in CI only — rejected: the doc names both scales.

## D14 — Coverage (NFR-MNT-01, card acceptance)

- **Decision**: measure with `cargo llvm-cov nextest -p alvs-core … / -p alvs-model …` at implementation time; record the ≥ 80% numbers in the PR evidence (SC-006). No CI threshold step is added in this package — standing enforcement of NFR-MNT-01 across all crates is the coverage-gate mechanism that belongs to the quality-gates scope, not to the first code package (WP-01's rust-test job uploads the artifact; the threshold check can be added by the work package the canonical docs assign it to — flagged in the plan as a follow-up question for the owner, not decided here).
- **Rationale**: card acceptance says "coverage ≥ 80%" (measured outcome); G-02 forbids adding unassigned CI gates.

## D15 — ts-rs bindings (04 §4.2; 03 §3.2.6)

- **Decision**: DTO types live in `alvs-model::dto`, derive `Serialize, Deserialize, TS`, and export to `app/src/ipc/bindings/` via an `export_bindings` test; every flicks field carries `#[ts(type = "number")]` (annotated on the struct fields — `#[ts(type)]` is not allowed on flattened fields); the generated bindings are committed. **Mechanism (corrected, review F3)**: ts-rs 12's API is `TS::export(cfg)` / `TS::export_all(cfg)` / `TS::export_to_string(cfg)` plus the `#[ts(export_to = "…")]` attribute and the `TS_RS_EXPORT_DIR` environment/config variable — the export test steers output to `app/src/ipc/bindings/` with it; there is no `TS::export_to` method (exact calls verified against the vendored source at implementation, per the anti-hallucination rule).
- **CI drift step is this package's deliverable (corrected, review F1)**: WP-01's contract assigned the bindings-drift CI step to WP-02 ("Deferred steps and their owners: bindings drift (WP-02)"; docs/08 §8.3.1 lists it in the `rust-lint` row). WP-02 therefore adds to `rust-lint`, mirroring `tools/verify.ps1` lines 58–61: `cargo test -p alvs-model export_bindings --locked` followed by `git diff --exit-code -- app/src/ipc/bindings` (gate steps report independently, `if: always()`). The local gate activates by target presence; the CI step activates with this package's workflow edit. ADR-010's "CI fails if regenerated bindings differ" is enforced from WP-02 on.
- **Rationale**: 04 §4.2 (DTOs in `alvs-model::dto`, `cargo test -p <crate> export_bindings`, CI fails on drift — ADR-010) + 03 §3.2.6 + the WP-01 contract's explicit ownership assignment.

## D16 — `extra` maps, enum IDs, and the DTO surface (03 §3.2.3–3.2.4; owner decision 2026-11-03)

- **Decision**: every persisted struct carries `#[serde(flatten)] pub extra: BTreeMap<String, serde_json::Value>` with default+skip-when-empty handling that preserves unknown fields verbatim on round-trip (test: unknown-field survival); enums serialize as their catalog string IDs (e.g. `"fx.color.basic"`, `"normal"`, `"CLR-03"`) via string tags/rename per type. On the DTO surface the `extra` maps are **included, not skipped** (owner decision, review F2): with ts-rs's `serde-json-impl` feature (D2) the flattened maps compile and appear in TypeScript as `T & { [key: string]: JsonValue }`, keeping 03 §3.2.3's forward-compatibility visible in the typed surface the UI consumes.
- **Rationale**: 03 §3.2.3 mandates the maps on every persisted object unconditionally; the plan's DTO contract re-exports those same types; skipping them would have closed the TS types but silently narrowed §3.2.3's guarantee for the UI side — the owner chose inclusion.

## D17 — Clippy lint posture (02 §2.11; owner decision 2026-11-03)

- **Decision**: WP-02 adds the §2.11 lint posture to the workspace: `[workspace.lints]` in the root `Cargo.toml` with `clippy.unwrap_used = "warn"` and `clippy.expect_used = "warn"`, wired into the member crates via `[lints] workspace = true` (and into every later crate as it gains code). CI and `tools/verify.ps1` already run clippy with `-D warnings`, which escalates these warnings to failures — exactly §2.11's "set to warn and denies new warnings". Test code remains free to use `unwrap` (lints scope or `#[cfg(test)]` allowlist at the statement level only, never file-wide).
- **Rationale**: 02 §2.11 is canonical and in the card's Read line; the reviewer verified no lint config exists in the repo today (a gap WP-02 owns as the first real-code package; owner chose to enforce now). `#![deny(missing_docs)]` (02 §2.4 rule 4) already rides the same crates.
- **Alternatives**: leaving §2.11 prose-only — rejected by the owner; statement-level `expect("invariant: …")` documentation exceptions remain per §2.11.
