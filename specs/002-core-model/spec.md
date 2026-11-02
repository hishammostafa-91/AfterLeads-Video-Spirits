# Feature Specification: WP-02 — Core types and project model

**Feature Branch**: `feat/002-core-model`

**Created**: 2026-11-02

**Status**: Draft

**Input**: User description: "Implement WP-02 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: REQ-PRJ-001 (model), REQ-PRJ-005 (model), REQ-PRJ-007 (undo engine), REQ-PRJ-008. Acceptance tests: T-PRJ-005, T-PRJ-006, T-PRJ-007, T-PLY-005. Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents."

**Canonical basis**: WP-02 card (docs/10 §10.7). Requirements cited from docs/01-product.md (REQ-PRJ-001/005/007/008 rows quoted where binding; REQ-PLY-005 via the T-PLY-005 acceptance row). Design authorities from the card's Read line: 02 ADR-008/009, §2.4, §2.11; 03 §3.2–3.5, §3.9.6; 04 §4.2, §4.4, §4.9; 12 §12.8–12.9, §12.13; test definitions from 09 §9.4 and §9.10. Nothing is added beyond those documents (G-01, G-02, constitution I).

## User Scenarios & Testing *(mandatory)*

Actors: the editor's future UI (WP-05+), the engine, and persistence services (WP-03) consuming these crates — and through them, the end user of REQ-PRJ-001/005/007/008. This package is the product's data heart: every timeline position, project file, edit, and undo step stands on it.

### User Story 1 - Exact time: flicks, rationals, and timecode (Priority: P1)

Every duration, position, and frame rate in the product is represented exactly, and the user can read and type times as timecode. A frame at any supported rate (12 §12.9, incl. 29.97/59.94 drop-frame) maps to an integer flick count with zero drift, and timecode display/entry (`HH:MM:SS:FF`, `;` for drop-frame; typed absolute or `+/-` relative values) round-trips exactly (REQ-PLY-005 via T-PLY-005).

**Why this priority**: every other story and every later work package consumes these primitives; a time error here is a product-wide error. ADR-008 names flicks the representation for exactly this reason.

**Independent Test**: T-PLY-005 suite — flicks/timecode conversions for every FPS entry including drop-frame, rational reduction, round-trip parse/format (09 §9.4 `alvs-core` row).

**Acceptance Scenarios**:

1. **Given** any catalog frame rate (12 §12.9) and any audio rate (48,000/44,100 Hz), **When** a frame/sample duration is computed in flicks (1 s = 705,600,000), **Then** it is an exact integer per the 03 §3.3.1 table, and frame `n` starts at exactly `n × frameFlicks`.
2. **Given** a time in flicks at 29.97 drop-frame, **When** it is formatted and re-parsed (including a typed `+/-` relative value), **Then** the same flicks value results, with the `;` separator (REQ-PLY-005; 03 §3.3.1).
3. **Given** a custom frame rate `num/den`, **When** checked, **Then** it is accepted only if `705600000 × den` is divisible by `num` and `1 ≤ num/den ≤ 120` (03 §3.3.1); a rational is always reduced with `den > 0` (03 §3.3.2).

---

### User Story 2 - The project model: every `.alvs` type, lossless (Priority: P2)

The complete project-file data model of 03 §3.4 exists as types: top-level project, media items, bins, assets, sequences, tracks and clips, effect instances with animatable values, masks, title documents, transitions, markers, caption tracks, and the non-undoable view state — each carrying unknown fields forward (`extra` maps) so files from newer versions never lose data on re-save (03 §3.2.3). Projects serialize to and from the canonical JSON conventions (UTF-8, camelCase, enum catalog IDs; 03 §3.2) and a JSON Schema is generated from the types (ADR-008).

**Why this priority**: REQ-PRJ-001's model side — the New Project data (name, sequence preset per 12 §12.8 with the SQP-01 auto-match fallback to SQP-02 at 30 fps, frame rate per 12 §12.9, 48/44.1 kHz audio, SDR Rec.709, square pixels, stereo master) — and every later feature reads or writes these structures.

**Independent Test**: serialization round-trip on representative fixtures for every 03 §3.4 entity; JSON Schema generation succeeds (09 §9.4 `alvs-model` row).

**Acceptance Scenarios**:

1. **Given** a project containing every entity kind of 03 §3.4, **When** it is serialized and deserialized, **Then** the result is value-equal, including unknown/forwarded fields and animatable values (static and keyframed).
2. **Given** the generated JSON Schema, **When** a valid project is validated against it, **Then** it passes (ADR-008).
3. **Given** a new project created from preset SQP-01 with no clip yet, **When** its format is inspected, **Then** it carries `autoMatchPending` with the SQP-02/30 fps fallback (03 §3.4.5; REQ-PRJ-001).

---

### User Story 3 - A model that refuses to be corrupt (Priority: P3)

Every invariant of 03 §3.5 is checked on load and after every applied edit in debug builds and tests: reference integrity, no nesting cycles (depth ≤ 8, E1006), track ordering without overlap, frame-grid alignment, media-duration bounds, transition adjacency and handle limits, keyframe rules, link-group shape, caption/marker rules, sequence format ranges, asset hashes, and the malicious-file size limits (≤ 99 tracks/kind, ≤ 20,000 clips/sequence, ≤ 100 sequences, ≤ 50,000 media, ≤ 10,000 keyframes/parameter, ≤ 20,000 captions/track, file ≤ 256 MB). An edit that would violate an invariant is rolled back and reported as E0002 (03 §3.5; THR-02 defense).

**Why this priority**: untrusted project files are a named threat (05 THR-02); validation is what makes every later edit surface safe, and its violations are contract breaches (E0002), not crashes.

**Independent Test**: every 03 §3.5 rule covered by a positive and a negative fixture; random edit sequences (09 §9.4 proptest) leave a valid model or produce a documented rejection.

**Acceptance Scenarios**:

1. **Given** a project file exceeding any 03 §3.5.15 size limit, **When** opened, **Then** loading fails with a validation issue rather than exhausting memory (T-PRJ-007 "malicious limits").
2. **Given** an edit whose result would break any 03 §3.5 rule, **When** applied, **Then** the previous root is kept, the edit is rejected, and E0002 semantics are reported (03 §3.5).

---

### User Story 4 - Edits and the undo engine (Priority: P4)

The `EditOp` enum exists per the 04 §4.4 contract (`#[serde(tag = "op")]`, localization key `history.<op>` per variant), and this package applies the **project / sequence / track / media / bin / marker / caption-track** categories: media add/remove/update (with E1102 protection), bin create/update/delete (delete moves contents to the parent), sequence create/duplicate/delete (never the last one)/update (frame-rate change snaps clip times to the new grid and reports adjusted items, E1104), track add/remove/move/update, markers, caption tracks, `renameProject`, and `batch` (all-or-nothing, one undo step). Undo/redo uses ADR-009: an immutable state tree with `Arc` sharing; every edit produces a new root; undo/redo move between roots — at least 200 steps guaranteed and up to 1,000 within a 512 MB budget (REQ-PRJ-007) — and each application returns an `EntityPatch` (revision, upsert/remove by entity kind, dirtyRanges) computed by structural diffing (04 §4.4).

**Why this priority**: REQ-PRJ-005 and REQ-PRJ-007 are the user-facing core of editing; everything timeline-shaped builds on this engine (the History panel itself is WP-13 — this package supplies the steps it lists and jumps through).

**Independent Test**: T-PRJ-005 (sequence create/duplicate/delete; frame-rate snapping + E1104) and T-PRJ-006 (undo/redo equality, 200-step guarantee, memory budget) suites; every implemented op has a happy path and a rejection test (09 §9.4).

**Acceptance Scenarios**:

1. **Given** a project with one sequence, **When** `deleteSequence` targets it, **Then** the edit is rejected (REQ-PRJ-005 "cannot delete the last one").
2. **Given** a sequence whose frame rate changes from 30 to 25, **When** `updateSequence` applies, **Then** clip times snap to the new frame grid, any adjusted items are reported (E1104 semantics), and the result validates (REQ-PRJ-005).
3. **Given** 1,000 applied edits within the memory budget, **When** undo is called 1,000 times, **Then** every intermediate root is reproduced exactly (undo = redo inverse, no drift; T-PRJ-006).
4. **Given** any applied edit, **When** its `EntityPatch` is applied to a normalized copy of the prior state, **Then** the copy equals the new root (patch correctness; 04 §4.4).

---

### User Story 5 - Versioned files that never lose data (Priority: P5)

Project schema versioning per REQ-PRJ-008: `schemaVersion` with **forward-only** migrations `v(n) → v(n+1)` over the JSON value; the original file is copied to `<Name>.alvs.v<old>.bak` before migration (backup performed by the persistence layer of WP-03; the migration framework, backup contract, and validation-on-migrated-result land here); files from a newer application version open **read-only** with E1004 and keep unknown fields in memory; every migration ships with input/output fixtures (03 §3.9.6). The crate's DTOs export TypeScript bindings with every flicks field typed `number` (03 §3.2.6), regenerated by an export test that the CI bindings-drift check (WP-01) enforces.

**Why this priority**: durability rules (G-16) and the compatibility contract; it closes the package by making the model consumable by the UI (04 §4.2) and safe across versions.

**Independent Test**: T-PRJ-007 (migrations, newer-version read-only, malicious limits) with v1 fixtures; `cargo test -p alvs-model export_bindings` produces the committed bindings (09 §9.4; 04 §4.2).

**Acceptance Scenarios**:

1. **Given** a v1 project file, **When** the current version opens it, **Then** migration runs forward-only, the result validates, and the backup contract is honored (03 §3.9.6; T-PRJ-007).
2. **Given** a file with `schemaVersion` greater than current, **When** opened, **Then** it is read-only with E1004 and unknown fields survive in memory (03 §3.9.6; REQ-PRJ-008).
3. **Given** the exported TypeScript bindings, **When** regenerated, **Then** they match the committed ones byte-for-byte, with all flicks fields as `number` (03 §3.2.6; 04 §4.2).

---

### Edge Cases

- A malicious or corrupt file (huge structures, bad references, oversized values) must fail validation with named issues — never panic, never exhaust memory (03 §3.5.15; 05 THR-02).
- Unknown fields from newer versions: preserved through load → edit → save even while the file is read-only-restricted (03 §3.2.3 with 3.9.6).
- Frame-rate change with items that cannot snap losslessly: the snapped result must still validate; adjusted items are reported (E1104), never silently dropped (REQ-PRJ-005).
- `removeMedia` when clips reference the media: rejected with E1102 unless the caller explicitly opts to also remove the clips (04 §4.4).
- Undo budget: reaching the 512 MB cap must drop the oldest steps while preserving at least 200 (REQ-PRJ-007); memory measurement is deterministic in tests.
- Timestamps from media at non-catalog rates convert with round-half-to-even (error < 1.5 ns; 03 §3.3.1).
- All persisted times fit i64 and stay below 2^53 for TS (`number`); the 24-hour maximum sequence length bounds this (03 §3.3.1).

## Requirements *(mandatory)*

All requirements are scoped to the WP-02 card. IDs are feature-local (FR-xxx) and cite their canonical source.

### Functional Requirements

- **FR-001**: `alvs-core` MUST define flicks with 1 second = 705,600,000, the exact per-rate durations of 03 §3.3.1 for every 12 §12.9 frame rate and both audio rates, frame-start arithmetic (`n × frameFlicks`), the custom-rate validity rule (`705600000 × den` divisible by `num`, `1 ≤ num/den ≤ 120`), round-half-to-even conversion for foreign rates, and the 24-hour sequence bound (03 §3.3.1).
- **FR-002**: `alvs-core` MUST define reduced rationals (`num`/`den`, `den > 0`) for frame rates and speeds (03 §3.3.2), normalized space/angles and dB/pan conventions as value documentation for parameter types (03 §3.3.3–3.3.4).
- **FR-003**: `alvs-core::time` MUST implement timecode formatting and parsing — `HH:MM:SS:FF`, `;` separator for drop-frame at 29.97/59.94, frames and seconds forms, typed absolute and `+/-` relative entry — and be the only display-conversion code (REQ-PLY-005; 03 §3.3.1; T-PLY-005).
- **FR-004**: `alvs-core` MUST provide UUID v7 identifiers per 03 §3.2.2 (unique per project, never reused) and the path policy of SEC-04: canonicalization, rejection of Windows device paths and reserved names (`CON`, `NUL`, `AUX`, `PRN`, `COM1–9`, `LPT1–9`), write containment to the project folder/cache/user library/user-picked folders, and UNC-from-project-file confirmation gating (05 §5.7 SEC-04).
- **FR-005**: `alvs-core` MUST define the error model: `thiserror` enums with every variant mapping to exactly one code of 12 §12.13 via `fn code(&self)`, including the error envelope type of 04 §4.9 (`code`, `severity`, `messageKey = error.<code>`, `params`, `detail`, `retryable`, `recovery`), covering at minimum the codes this package can raise: E0001, E0002, E1003, E1004, E1005, E1006, E1102, E1104, E2001 (02 §2.11; 12 §12.13; 04 §4.9).
- **FR-006**: `alvs-model` MUST define every 03 §3.4 type — top-level project (format string, schemaVersion, savedBy, settings incl. `defaultSequenceId`/`proxyMode`), media items (fingerprint, probe snapshot, image-sequence, label, derived), bins (tree), assets (LUT info), sequences (format with `rec709-sdr`, `autoMatchPending`, in/out points, master audio), tracks and clips (all clip types `media|title|generator|sequence`, intrinsic transform/crop/opacity, the audio object with `channelMode` and `fadeCurve`), effect instances and animatable values (static/keyframed, all value types of 03 §3.4.7), masks, title documents, transitions with alignment coverage, markers (`standard|chapter`), caption tracks with style, and the `ui` view state — each persisted object carrying `#[serde(flatten)] extra` for unknown fields (03 §3.4; 03 §3.2.3).
- **FR-007**: Serialization MUST follow 03 §3.2: UTF-8 without BOM, LF, 2-space indent, camelCase keys, enum catalog string IDs, integers for times/counts/sizes and floats for parameters; every flicks field annotated `#[ts(type = "number")]` (03 §3.2).
- **FR-008**: `alvs-model` MUST validate every rule of 03 §3.5 (references, nesting depth ≤ 8 with E1006, track sort/overlap, frame-grid multiples incl. audio quarter-frame, `duration > 0`, speed 1/10..16, source bounds incl. `freezeAt`, transition adjacency/handles, keyframe rules, link-group shape, caption and marker rules, format ranges even 16–8,192 and 44.1/48 kHz, asset sha256, and the size limits of 3.5.15), on load and after every applied edit in debug builds and tests; a violating edit rolls back with E0002 (03 §3.5).
- **FR-009**: The `EditOp` enum MUST follow 04 §4.4: `#[serde(tag = "op")]`, the documented field shapes, `ParamTarget`, and `history.<op>` keys; this package MUST implement application for the project, sequence, track, media, bin, marker, and caption-track categories (addMedia/removeMedia with E1102 and `alsoRemoveClips`, updateMedia, createBin/updateBin/deleteBin with content re-parenting, createSequence with `fromPreset` per 12 §12.8, duplicateSequence/deleteSequence with last-one protection/updateSequence with frame-rate snapping and E1104, addTrack/removeTrack/moveTrack/updateTrack, addMarkers/updateMarker/removeMarkers, addCaptionTrack/updateCaptionTrack/removeCaptionTrack, renameProject, and batch as one undo step) (04 §4.4; card Deliverables).
- **FR-010**: Clip-editing semantics — insert/move/trim/slip/slide/split/delete/lift/extract, links/groups, clip timing and attribute ops, effect/mask/transition/title/generator ops, nesting, and caption-item ops — MUST NOT be implemented in this package (card "Not included"; 04 §4.4 defines their future contract).
- **FR-011**: Undo/redo MUST use the ADR-009 design — immutable state tree, `Arc` sharing, every edit producing a new root, undo/redo moving between roots — guaranteeing ≥ 200 steps and up to 1,000 within a 512 MB budget, evicting oldest steps beyond the budget while never dropping below 200 (ADR-009; REQ-PRJ-007).
- **FR-012**: Every applied op MUST return an `EditResult` containing an `EntityPatch` (revision, upsert/remove maps for the ten entity kinds, dirtyRanges with sequence time extents) computed by structural diffing of consecutive roots (04 §4.4; ADR-009).
- **FR-013**: The migrations framework MUST implement forward-only `v(n) → v(n+1)` steps over the JSON value, validation of the migrated result, read-only opening with E1004 when `schemaVersion` > current (unknown fields kept), and the v1 fixture set proving each step (03 §3.9.6; REQ-PRJ-008; T-PRJ-007).
- **FR-014**: A JSON Schema for the project file MUST be generated from the types (ADR-008).
- **FR-015**: DTOs MUST export TypeScript bindings via an export test to `app/src/ipc/bindings/`, regenerable and byte-stable, with flicks fields as `number` and `extra` maps included in the TS surface; the CI bindings-drift check (export test + diff) runs in the `rust-lint` job, delivered by this package per the WP-01 contract's assignment (04 §4.2; 03 §3.2.6; ADR-010).
- **FR-016**: Tests MUST cover, per 09 §9.4: every FPS entry's flicks/timecode conversions incl. drop-frame and rational reduction and path policy (alvs-core); every implemented `EditOp` happy path + rejection, 03 §3.5 invariants after random edit sequences (proptest), undo/redo round-trip equality, entity-patch correctness, serialization round-trip, migrations, and schema generation (alvs-model); plus the card's named suites T-PRJ-005, T-PRJ-006, T-PRJ-007, T-PLY-005 (card Acceptance; 09 §9.10 rows).
- **FR-017**: Line coverage MUST be ≥ 80% for `alvs-core` and `alvs-model` (card Acceptance; NFR-MNT-01).
- **FR-018**: The crates MUST be `cli`-free — no CLI dependency or entry point (card Acceptance; 02 §2.4 dependency discipline).
- **FR-019**: Scope boundary — persistence services (WP-03: paths, save/open, autosave, backups, locks, settings, cache), the History panel UI (WP-13), and all clip-editing semantics (WP-12) are excluded (card "Not included").

### Key Entities *(this package defines the data model)*

The entities of 03 §3.4, in brief (fields, rules, and JSON shapes are normative there, not restated here): **Project** (§3.4.1) · **MediaItem** (§3.4.2) · **Bin** (§3.4.3) · **Asset** (§3.4.4) · **Sequence** (§3.4.5) · **Track / Clip** (§3.4.6, clip types media|title|generator|sequence) · **EffectInstance + AnimatableValue/Keyframe** (§3.4.7) · **Mask** (§3.4.8) · **TitleDocument** (§3.4.9) · **Transition / Marker / CaptionTrack** (§3.4.10) · **ViewState** (§3.4.11, non-undoable). Behavioral entities from 04 §4.4: **EditOp** (tagged enum, `history.<op>`), **EntityPatch** (revision + upsert/remove + dirtyRanges). Value primitives from 03 §3.3: flicks, rational, normalized space, dB/pan.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T-PLY-005 passes — every 12 §12.9 rate (incl. drop-frame round-trips and typed `+/-` entry) converts exactly (property + unit suite).
- **SC-002**: T-PRJ-005 passes — sequence create/duplicate/rename/delete rules and frame-rate snapping with adjusted-item reporting (E1104) verified.
- **SC-003**: T-PRJ-006 passes — undo/redo equality over ≥ 200 (and up to 1,000) steps within 512 MB, oldest-first eviction below 200 never occurring.
- **SC-004**: T-PRJ-007 passes — every migration fixture pair round-trips, newer-version files open read-only (E1004), and every 03 §3.5.15 malicious limit rejects.
- **SC-005**: Property suite of 09 §9.4 green in CI scope (10,000 sequences × 50 ops on implemented ops) with invariants held after every step.
- **SC-006**: Coverage ≥ 80% measured for both crates; serialization round-trip and schema generation green; regenerated TS bindings byte-identical.

## Assumptions

- Dependencies are the sanctioned 00 §0.8 baseline rows: serde/serde_json/schemars/ts-rs/uuid (Data), thiserror (Core), arc-swap (Core), proptest/insta (Tests); versions resolved at planning from the baseline and lockfile (G-11; no ADR needed for baseline usage).
- The proptest CI/nightly split (10,000×50 vs 1,000,000×50, 09 §9.4) is honored as: the CI-scale suite lands here; the nightly-scale run activates with the nightly full-matrix job that later work packages add (WP-01 scoped nightly to audit + links).
- `AppError`-to-envelope mapping and logging field conventions follow 02 §2.11; the UI's localization of `error.<code>`/`history.<op>` keys is the UI packages' work (G-13 keys are named here, not translated).
- The persistence-layer half of REQ-PRJ-008 (physically writing the `.bak` before migration, atomic save) is WP-03; this package provides the migration steps, the backup contract, and validation.
- The History panel (listing/jumping, PNL-13) is WP-13; this package exposes the step list and jump as model operations.
- `schemaVersion` = 1 for release 1.0 (03 §3.4.1); the v1 fixtures are the baseline fixture set (03 §3.9.6).
- The bindings-drift **local** gate activates automatically by target presence (verify.ps1 checks `app/src/ipc/bindings`); the **CI** step does not exist until this package adds it to `rust-lint` — WP-01's contract explicitly assigns that step to WP-02 (specs/001 contracts, deferred-steps list), and FR-015 owns it here.
