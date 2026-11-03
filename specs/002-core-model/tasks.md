# Tasks: WP-02 — Core types and project model

**Input**: Design documents from `/specs/002-core-model/` (spec.md, plan.md, research.md D1–D17, data-model.md, contracts/crate-apis.md, quickstart.md)

**Prerequisites**: plan.md (approved by the owner, OD-08, 2026-11-03, after independent review), spec.md, research.md, data-model.md, contracts/crate-apis.md, quickstart.md

**Tests**: Test tasks are first-class here — the card's acceptance is the four named suites (T-PRJ-005, T-PRJ-006, T-PRJ-007, T-PLY-005) plus the 09 §9.4 duties; each implementation task names the tests it must land with, and each verification task names the command and required outcome (constitution III).

**Organization**: Tasks are grouped by user story (spec.md US1–US5) to enable independent implementation and verification of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Every task names concrete file paths

## Path Conventions

Monorepo per docs/02 §2.13: `crates/alvs-core` and `crates/alvs-model` (both in the root Cargo workspace). All paths below are repository-relative. Dependency versions are pinned per research D1 (verified 2026-11-02); ts-rs carries `serde-json-impl` (D2/D16).

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Baseline evidence and the two repository-infrastructure touches the plan owns.

- [x] T001 Capture the before-state for the evidence pack: `pwsh -File tools/verify.ps1` exit 0 with the WP-01-final gate table (bindings-drift SKIPPED — `app/src/ipc/bindings` absent); both crates build as empty libs with `#![deny(missing_docs)]`; `Cargo.lock` carries serde/serde_json/thiserror/uuid (app transitives) but not schemars/ts-rs/proptest (specs/002-core-model/evidence.md)
- [x] T002 [P] Add the dependency sets of research D2 to `crates/alvs-core/Cargo.toml` (serde derive, serde_json, thiserror, uuid v7; dev proptest) and `crates/alvs-model/Cargo.toml` (alvs-core, serde derive, serde_json, schemars, ts-rs with `serde-json-impl`, thiserror, uuid v7; dev proptest); `cargo build --workspace --locked` → exit 0; commit the updated `Cargo.lock` (crates/alvs-core/Cargo.toml)
- [x] T003 [P] Add the 02 §2.11 lint posture of research D17: `[workspace.lints]` in the root `Cargo.toml` (`clippy.unwrap_used = "warn"`, `clippy.expect_used = "warn"`, plus `rust.unused = "warn"` hygiene) with `[lints] workspace = true` in both crates; `cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0 (Cargo.toml)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `alvs-core` primitives every story consumes. No user-story label by design.

**⚠ CRITICAL**: US2–US5 cannot begin until this phase is complete (US1 shares it).

- [x] T004 [P] Create `crates/alvs-core/src/error.rs`: `ErrorCode` (E0001, E0002, E1003, E1004, E1005, E1006, E1102, E1104, E2001 — names and severities per 12 §12.13), `AppError` (thiserror; every variant maps to exactly one code via `fn code(&self)`), `ErrorEnvelope` serializable field-by-field per 04 §4.9 (`code`, `severity`, `messageKey = "error.<code>"`, `params`, `detail`, `retryable`, `recovery[{commandId,labelKey}]`); unit tests: variant→code uniqueness, envelope JSON shape (crates/alvs-core/src/error.rs)
- [x] T005 [P] Create `crates/alvs-core/src/id.rs`: UUID v7 mint/parse/format per 03 §3.2.2 (lowercase hyphenated; uniqueness by construction); tests: format validity, monotonic-ish ordering sanity (crates/alvs-core/src/id.rs)
- [x] T006 [P] Create `crates/alvs-core/src/path.rs`: `PathDecision::Allowed | NeedsConfirmation | Denied(AppError)` + `check_path(candidate, roots)` per SEC-04 — canonicalize; reject `\\.\`, `\\?\GLOBALROOT`, reserved names (CON, NUL, AUX, PRN, COM1–9, LPT1–9, case-insensitive, with-extension); containment to caller-supplied roots; UNC ⇒ `NeedsConfirmation`; tests: every rejection class, containment boundaries, UNC case (09 §9.4 alvs-core row) (crates/alvs-core/src/path.rs)

**Checkpoint**: Core primitives compile, lint-clean, documented, tested.

---

## Phase 3: User Story 1 — Exact time: flicks, rationals, timecode (Priority: P1) ⭐ MVP

**Goal**: FR-001–FR-003 — flicks with the literal 03 §3.3.1 table, reduced rationals, drop-frame timecode.

**Independent Test**: `cargo nextest run -p alvs-core --locked` → T-PLY-005 suite green (SC-001).

- [x] T007 [P] [US1] Create `crates/alvs-core/src/rational.rs`: `Rational { num: u32, den: u32 }` always reduced, `den > 0`, normalizing constructor; property test: reduction invariants over random pairs (03 §3.3.2) (crates/alvs-core/src/rational.rs)
- [x] T008 [US1] Create `crates/alvs-core/src/time.rs` part 1 — `Flicks(i64)`, `FLICKS_PER_SECOND = 705_600_000`, `frame_duration(rate)`, `frame_start(n, rate)`, custom-rate validity (`705600000 × den` divisible by `num`, `1 ≤ num/den ≤ 120`), round-half-to-even foreign conversion, `MAX_SEQUENCE_LENGTH` (24 h); **the literal per-rate test asserts the 03 §3.3.1 table exactly** (23.976→29,429,400 … 60→11,760,000; 48 kHz→14,700; 44.1 kHz→16,000) (crates/alvs-core/src/time.rs)
- [x] T009 [US1] Add `Timecode` to `crates/alvs-core/src/time.rs`: format/parse `HH:MM:SS:FF` and `;` drop-frame at 29.97/59.94 (12 §12.9 FPS-04/07) per SMPTE ST 12-1 (drop 2/min at 30000/1001, 4/min at 60000/1001, except every 10th minute; 24 h wrap), frames and seconds forms, typed absolute and `+`/`-` relative entry; **T-PLY-005 suite**: round-trip property at every catalog rate, drop-frame boundary cases (minute 1, minute 10), relative-entry arithmetic (REQ-PLY-005; research D5) (crates/alvs-core/src/time.rs)

**Checkpoint**: SC-001 — the core time foundation is exact and proven.

---

## Phase 4: User Story 2 — The project model, lossless (Priority: P2)

**Goal**: FR-006, FR-007, FR-014 — every 03 §3.4 type with `extra` maps, canonical serialization, JSON Schema.

**Independent Test**: entity round-trip fixtures + schema generation green (SC-006 partial).

- [x] T010 [P] [US2] Create `crates/alvs-model/src/project.rs`, `media.rs`, `bin.rs`, `asset.rs`: top-level Project (format string, schemaVersion, savedBy, settings), MediaItem (fingerprint, probe snapshot enums, imageSequence, label, derived), Bin (tree, parentId), Asset (lutInfo, sha256) per 03 §3.4.1–3.4.4 with `#[serde(flatten)] extra` (D16) and camelCase/enum-ID serde (03 §3.2) (crates/alvs-model/src/media.rs)
- [x] T011 [P] [US2] Create `crates/alvs-model/src/sequence.rs`: Sequence + format (rec709-sdr, autoMatchPending, in/out points, master audio) per 03 §3.4.5; every flicks field `#[ts(type = "number")]` where TS derives apply (crates/alvs-model/src/sequence.rs)
- [x] T012 [P] [US2] Create `crates/alvs-model/src/track.rs`: Track + Clip (types media|title|generator|sequence; sourceIn/speed/freezeAt/linkGroupId/groupId), intrinsic transform/crop/opacity, the audio object (channelMode, fadeCurve) per 03 §3.4.6 (crates/alvs-model/src/track.rs)
- [x] T013 [P] [US2] Create `crates/alvs-model/src/effect.rs`, `mask.rs`, `title.rs`: EffectInstance + animatable values (static/keyframed, all §3.4.7 value types, bezier handles), Mask (usage clip|effect), TitleDocument (layers, anim in/out, template) per 03 §3.4.7–3.4.9 (crates/alvs-model/src/effect.rs)
- [x] T014 [P] [US2] Create `crates/alvs-model/src/transition.rs`, `marker.rs`, `caption.rs`: Transition (alignment coverage), Marker (standard|chapter), CaptionTrack + style + items per 03 §3.4.10 (crates/alvs-model/src/caption.rs)
- [x] T015 [US2] Serialization proof: fixtures covering every entity kind (incl. unknown-field survival through `extra`, enum catalog IDs, keyframe/Bezier shapes); JSON round-trip value-equality tests; 2-space/UTF-8/camelCase conformance (03 §3.2; FR-007) (crates/alvs-model/tests/)
- [x] T016 [US2] Create `crates/alvs-model/src/schema.rs`: `project_json_schema()` via schemars `schema_for!` (ADR-008; research D1 anchor); test: a valid project validates against the generated schema (crates/alvs-model/src/schema.rs)

**Checkpoint**: SC-006 partial — the type set round-trips losslessly and schematizes.

---

## Phase 5: User Story 3 — A model that refuses to be corrupt (Priority: P3)

**Goal**: FR-008 — all 03 §3.5 rules as `Issue` findings + malicious-size limits.

**Independent Test**: positive+negative fixture per rule; oversized file rejected (SC-004 part).

- [ ] T017 [US3] Create `crates/alvs-model/src/validate.rs`: `validate(&Project) -> Vec<Issue>` implementing 03 §3.5 rules 1–14 (IDs, references incl. the §3.5.2 list, nesting depth ≤ 8 → E1006, track sort/overlap, frame-grid multiples incl. audio quarter-frame, duration>0 + speed bounds, source bounds + freezeAt, transition adjacency/handles/one-per-edit, keyframe rules, link-group shape, caption rules, marker rules incl. chapter uniqueness, format ranges incl. 44.1/48 kHz, asset sha256); each rule covered by a passing and a failing fixture (crates/alvs-model/src/validate.rs)
- [ ] T018 [US3] Add the 03 §3.5.15 size limits as first-pass checks (≤ 99 tracks/kind, ≤ 20,000 clips/sequence, ≤ 100 sequences, ≤ 50,000 media, ≤ 10,000 keyframes/parameter, ≤ 20,000 captions/track, file ≤ 256 MB over the JSON value); malicious-fixture tests reject without traversal (05 THR-02; T-PRJ-007 part) (crates/alvs-model/src/validate.rs)

**Checkpoint**: SC-004 part — validation is complete and size-hardened.

---

## Phase 6: User Story 4 — Edits and the undo engine (Priority: P4)

**Goal**: FR-009, FR-011, FR-012 — EditOp for the card's categories, ADR-009 undo/redo with budget, EntityPatch.

**Independent Test**: T-PRJ-005 + T-PRJ-006 suites green (SC-002, SC-003).

- [ ] T019 [US4] Create `crates/alvs-model/src/edit.rs` part 1: the `EditOp` enum (`#[serde(tag = "op")]`) with the WP-02 category variants and their 04 §4.4 field shapes, `ParamTarget`, `history_key() -> "history.<op>"` per variant, `EditResult`; serde tests for every variant's JSON shape (crates/alvs-model/src/edit.rs)
- [ ] T020 [US4] Apply the project/media/bin categories: `addMedia`, `removeMedia` (E1102 unless `alsoRemoveClips`, then full-extent dirty range), `updateMedia`, `createBin`/`updateBin`/`deleteBin` (contents re-parent to the parent bin), `renameProject`; happy-path + rejection tests each (04 §4.4) (crates/alvs-model/src/edit.rs)
- [ ] T021 [US4] Apply the sequence/track/marker/caption-track categories: `createSequence` (with `fromPreset` per 12 §12.8, `autoMatchPending` per 03 §3.4.5), `duplicateSequence`, `deleteSequence` (reject deleting the last), `updateSequence` (frame-rate change snaps clip times to the new grid, collects adjusted items — E1104 data; in/out points), `addTrack`/`removeTrack`/`moveTrack`/`updateTrack`, `addMarkers`/`updateMarker`/`removeMarkers`, `addCaptionTrack`/`updateCaptionTrack`/`removeCaptionTrack`, and `batch` (all-or-nothing, one undo step; interpretation record in research D8); **T-PRJ-005 suite** (REQ-PRJ-005) (crates/alvs-model/src/edit.rs)
- [ ] T022 [US4] Create `crates/alvs-model/src/history.rs`: `Model { root: Arc<Project>, history }`, `apply(op) -> EditResult` with clone-on-write via `Arc::make_mut`, post-op validation under debug/tests, rollback + E0002 on violation; `undo`/`redo` exact inverses; `steps()`/`step(i)` for the History panel (WP-13); **deterministic estimated-size budget** (D9): 512 MB oldest-first eviction, unconditional ≥ 200 floor, ≤ 1,000 cap; **T-PRJ-006 suite**: 200/1000-step equality, eviction order, floor-under-oversized-roots (REQ-PRJ-007) (crates/alvs-model/src/history.rs)
- [ ] T023 [US4] Create `crates/alvs-model/src/patch.rs`: `EntityPatch` (revision; upsert/remove for the ten kinds; dirtyRanges) per 04 §4.4; `diff(old, new)` with `Arc::ptr_eq` short-circuit; the D10 range policy (structural ⇒ full sequence extent; marker ⇒ point; media/bin/project ⇒ none); property: patch-application equivalence (spec US4/AC4) (crates/alvs-model/src/patch.rs)
- [ ] T024 [US4] Rejection semantics suite: every implemented op's documented failure (E1102, E1104 paths, last-sequence, validation rollback E0002) with fixtures; random-edit proptest strategy (bounded projects; ops drawn from the WP-02 set; applied ⇒ invariants hold, rejected ⇒ documented code) at **CI scale 10,000×50** via `ALVS_PROPTEST_CASES`; record the measured duration for the PR evidence (09 §9.4; research D13; raise-only knob, G-08) (crates/alvs-model/tests/)

**Checkpoint**: SC-002, SC-003, SC-005 — the edit engine and its proofs are complete.

---

## Phase 7: User Story 5 — Versioned files and typed surface (Priority: P5)

**Goal**: FR-013, FR-015 — migrations framework + fixtures; DTOs, committed bindings, CI drift step.

**Independent Test**: T-PRJ-007 green; bindings regenerate byte-identical; drift gates active.

- [ ] T025 [P] [US5] Create `crates/alvs-model/src/migrate.rs`: `MigrationStep` registry, forward-only driver `v(n) → v(n+1)` over `serde_json::Value` with validation of the result, `MigrationOutcome` (Migrated / ReadOnlyNewer → E1004 with unknown fields kept / backup contract for WP-03), E1005 on step failure; shipped registry empty (schemaVersion = 1; research D12) (crates/alvs-model/src/migrate.rs)
- [ ] T026 [US5] Migration proofs under `crates/alvs-model/tests/fixtures/migrations/`: v1 input/output pairs round-trip through the driver; **cfg(test)-registered synthetic steps** exercise step-running, backup-signal, and E1005 paths (research D12/F5); **T-PRJ-007 suite**: migrations, newer-version read-only, malicious limits (03 §3.9.6) (crates/alvs-model/tests/)
- [ ] T027 [P] [US5] Create `crates/alvs-model/src/dto.rs`: the TS-exporting DTO surface (types deriving `Serialize, Deserialize, TS`; flicks fields `#[ts(type = "number")]` on struct fields; `extra` included via `serde-json-impl` → `T & { [key: string]: JsonValue }`, research D16) with the `export_bindings` test writing `app/src/ipc/bindings/` via `TS::export_all`/`TS_RS_EXPORT_DIR` (research D15; exact calls verified against the vendored ts-rs source); commit the generated bindings (04 §4.2; 03 §3.2.6) (crates/alvs-model/src/dto.rs)
- [ ] T028 [US5] Add the CI bindings-drift step to `.github/workflows/ci.yml` `rust-lint` (per WP-01's contract assignment — research D15): `cargo test -p alvs-model export_bindings --locked` then `git diff --exit-code -- app/src/ipc/bindings`, reporting independently (`if: always()`); locally confirm `pwsh -File tools/verify.ps1` now shows `bindings-drift` **PASS** (target presence activation) (tools/verify.ps1)

**Checkpoint**: SC-004 complete; drift gates live locally and in CI.

---

## Phase 8: Polish & Cross-Cutting Concerns

- [ ] T029 Measure coverage per quickstart §7: `cargo llvm-cov nextest -p alvs-core --locked --no-tests=pass --summary-only` and `-p alvs-model` → both ≥ 80.0% line coverage (card acceptance; NFR-MNT-01); record the numbers (research D14 defers standing CI enforcement)
- [ ] T030 Verify `cli`-free per quickstart §8: `cargo metadata --no-deps` shows no `[[bin]]` targets in either crate and no CLI dependency (FR-018); run the full quickstart §1–§8 and fix anything it catches at the source (specs/002-core-model/quickstart.md)
- [ ] T031 Assemble the pull-request evidence per G-09 in `specs/002-core-model/evidence.md`: T001 before-state, test-suite results by T-ID (T-PLY-005, T-PRJ-005/006/007), proptest duration, coverage numbers, bindings drift proof; update STATUS.md WP-02 next step → `finish` (state stays `in-progress` until merge) (specs/002-core-model/evidence.md)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 first; T002 and T003 parallel after (or with) it.
- **Foundational (Phase 2)**: depends on T002 (deps) and T003 (lints govern all new code).
- **US1 (Phase 3)**: T007 parallel with Phase 2; T008→T009 sequential (same file).
- **US2 (Phase 4)**: depends on Phase 2 (Id/errors) + T007/T008 (Rational/Flicks used by the type set); T010–T014 are five parallel module groups; T015/T016 after them.
- **US3 (Phase 5)**: depends on US2's types.
- **US4 (Phase 6)**: depends on US2 + US3 (+ Phase 2 errors); T019→T020→T021 sequential (one file); T022/T023 after T019 (need ops + roots); T024 last in the story.
- **US5 (Phase 7)**: T025 parallel-izable after US2 (migrations walk the JSON value); T026 after T025; T027 after US2's types (and benefits from T009's conventions); T028 after T027.
- **Polish (Phase 8)**: after everything.

### Parallel Opportunities

- Phase 1: T002 ∥ T003 · Phase 2: T004 ∥ T005 ∥ T006 (and ∥ T007).
- Phase 4: T010 ∥ T011 ∥ T012 ∥ T013 ∥ T014 (five disjoint module groups).
- Phase 7: T025 ∥ T027 (disjoint files), then T026 and T028.

---

## Parallel Example: User Story 2

```text
Task: "project/media/bin/asset types"        # T010
Task: "sequence type"                        # T011
Task: "track + clip types"                   # T012
Task: "effect/mask/title types"              # T013
Task: "transition/marker/caption types"      # T014
# then sequentially: T015 (round-trip proof), T016 (schema)
```

---

## Implementation Strategy

### MVP First (Phases 1–3)

1. Baseline + deps + lints (Phase 1), core primitives (Phase 2), exact time (Phase 3).
2. **STOP and VALIDATE**: `cargo nextest run -p alvs-core --locked` green — SC-001.

### Incremental Delivery

1. US2 → lossless types + schema (SC-006 partial). 2. US3 → validation hardened. 3. US4 → edit engine + suites (SC-002/003/005). 4. US5 → migrations + bindings + CI drift (SC-004, full SC-006). 5. Phase 8 → coverage, cli-free, evidence, `/wp-finish`.

---

## Notes

- [P] tasks = different files, no dependencies; same-file chains are sequential (T008→T009, T019→T020→T021).
- [Story] labels map to spec.md user stories for traceability.
- Commits: `feat(core)` / `feat(model)` for modules, `test(core|model)` for suites, `build(deps)` for manifests+lock, `ci(ci)` for the drift step, `chore(docs)`/`docs` for ledger/evidence — types and scopes from docs/10 §10.4.5, DCO always.
- The §2.11 lint posture (T003) governs every line: no `unwrap`/`expect` outside tests (statement-level test exceptions only); `#![deny(missing_docs)]` stays green — document every public item as it lands.
- `ALVS_PROPTEST_CASES` raises the property scale (nightly future); lowering CI scale with it is a gate weakening (G-08) and is forbidden.
- No persistence, no clip-editing semantics, no UI (card "Not included"; FR-010/FR-019).
- Stop and ask (G-17) on any discrepancy with the canonical docs; verify every API against the vendored crate sources before use (anti-hallucination rule 1).
