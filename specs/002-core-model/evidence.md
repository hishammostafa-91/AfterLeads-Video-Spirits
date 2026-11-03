# Evidence — WP-02 Core types and project model

Assembled per G-09 and task T031 on 2026-11-03, branch `feat/002-core-model`; **updated 2026-11-03 after the Phase 9 convergence pass (T032–T042)** — all numbers below are from the final branch state.

## Before-state (T001, captured before any WP-02 change)

- `pwsh -File tools/verify.ps1` exited 0 with the WP-01-final gate table; `bindings-drift` was **SKIPPED** because `app/src/ipc/bindings/` did not exist yet.
- Both crates built as empty libraries carrying only `#![deny(missing_docs)]` and SPDX headers.
- `Cargo.lock` carried serde, serde_json, thiserror, and uuid (app transitives) but **not** schemars, ts-rs, or proptest — all three were added by this package at the pinned, crates.io-verified versions recorded in `specs/002-core-model/research.md` (D2–D4).

## Quickstart §1–§8 (final run, `specs/002-core-model/quickstart.md`)

| § | Command | Result |
|---|---|---|
| 1 | `cargo nextest run -p alvs-core --locked` | **27/27 pass** — flicks table, custom-rate validity, rational reduction (incl. deserialization invariants), path policy (SEC-04, E9005), and the T-PLY-005 timecode suite incl. skipped-label canonicalization |
| 2–4 | `cargo nextest run -p alvs-model --locked` | **168/168 pass** — entity round-trips incl. unknown-field survival at nested levels, all fifteen 03 §3.5 rule groups with positive+negative fixtures (rules 1/7/8/9/12/15 at full strength incl. the 256 MB first pass), size-limit rejection, JSON Schema, edits/undo/patches with per-op `EntityPatch`, migrations with driver-side result validation |
| 5 | `cargo nextest run -p alvs-model --locked random_edit_sequences_hold_invariants` | **1/1 pass** — see below |
| 6 | `cargo test -p alvs-model export_bindings --locked` + `git status --porcelain -- app/src/ipc/bindings` | **60/60 pass; working tree clean** (no drift) |
| 7 | `cargo llvm-cov nextest -p alvs-core/-p alvs-model --locked --no-tests=pass --summary-only` | both ≥ 80% — see below |
| 8 | `pwsh -File tools/verify.ps1` · `python tools/docs/check_docs.py` · `cargo metadata --no-deps` | **all available gates PASS**; docs checks pass; `alvs-*` crates declare no `[[bin]]` targets (the only workspace binary is the Tauri GUI app itself) — `cli`-free per FR-018 |

Workspace total: `cargo nextest run --workspace --locked` → **195/195 pass**.

## Suites by T-ID

- **T-PLY-005** (drop-frame timecode, 09 §9.x): `alvs-core` time suite — format∘parse round-trip at every 12 §12.9 rate (drop-frame `;` at 29.97/59.94), typed `+/-` relative entry, frames/seconds forms. Green in §1's 24 tests.
- **T-PRJ-005** (project rules): sequence create/duplicate/rename/delete incl. last-sequence rejection and default/active reference switching; frame-rate change snapping with adjusted-item reporting (E1104 data); E1102 media-in-use protection; bin delete re-parenting; batch atomicity as one undo step — `tests/apply.rs` + `edit.rs` tests. Green.
- **T-PRJ-006** (undo/redo): ADR-009 Arc-root engine — undo/redo exact inverses over 50, 200, and 1,000 steps (SC-003 at the named scale), deterministic-budget eviction (≥ 200 floor, ≤ 1,000 cap, 512 MB estimate), History-panel step jump with multi-step redo ordering — `history.rs` tests. Green.
- **T-PRJ-007** (migrations, 03 §3.9.6): forward-only driver with v1 fixture pair round-trip (`tests/fixtures/migrations/v1_in.json`/`v1_out.json` via `tests/migrations.rs`), newer-`schemaVersion` read-only outcome with unknown fields kept, missing-step E1005 and failing-step paths via cfg(test) synthetic steps, and driver-side validation of the migrated result (parse + every 03 §3.5 rule; E1005 on invalid). Green.

## Property suite (SC-005)

- CI scale **10,000 random sequences × 50 ops**: **1 test, pass, 27.3 s wall** (run `Summary [26.677s] 1 test run: 1 passed`), far inside the ≤ 25-minute CI budget (research D13/F10). The strategy now draws from the **full WP-02 op set** with generated payloads (names, times, presets incl. invalid ones, formats, markers) and mixed live/dangling references (review F-08); `ALVS_PROPTEST_CASES` clamps any parsed value up to at least 10,000 (raise-only, review F-07).
- Invariant checked at every step: the step either applies (model still valid — `Model::apply` validates after each op in tests) or rejects with a documented code (E1102/E0002/E1003/E1006); the end state validates clean.
- `ALVS_PROPTEST_CASES` raises the case count only (default 10,000; parsed value below the default is ignored) — G-08 compliant; the 1,000,000-case nightly scale was **not** exercised (no nightly full-matrix job yet; deferred per research D13).

## Coverage (SC-006, NFR-MNT-01 — gate ≥ 80% line coverage)

| Crate | Regions | Functions | Lines |
|---|---|---|---|
| `alvs-core` | 1,134 / 101 missed = **91.09%** | 85.14% | 678 / 84 missed = **87.61%** |
| `alvs-model` | 5,793 / 192 missed = **96.69%** | 93.65% | 3,995 / 94 missed = **97.65%** |

Commands exactly as in quickstart §7 (`--no-tests=pass --summary-only`). Standing CI enforcement is deferred per research D14.

## TypeScript bindings and drift (SC-006, FR-015)

- **61 generated `.ts` files** committed under `app/src/ipc/bindings/` (incl. `EditOp.ts` + its input-struct types from the review pass, and `ParamTarget.ts`/`ParamIntrinsic.ts` from the convergence pass); every flicks field typed `number` (03 §3.2.6); `extra` maps present as `JsonValue` index signatures on every persisted struct incl. the nested ones (owner decision F2 / research D16+D19).
- **60 `export_bindings` tests** regenerate byte-identical output: `cargo test -p alvs-model export_bindings --locked` → 60/60, then `git diff --exit-code -- app/src/ipc/bindings` → clean.
- CI gate added to `rust-lint` (`ci.yml`) per WP-01's contract assignment (F1): the same export test + `git diff --exit-code`, reported independently (`if: always()`). Local `verify.ps1` now shows **bindings-drift PASS** (target-presence activation confirmed).

## Convergence phase (Phase 9, T032–T042, 2026-11-03)

`/speckit.converge` found 11 gaps (5 HIGH, 5 MEDIUM, 1 LOW; none CRITICAL, none unrequested); `/speckit.implement` closed all of them:

- **T032** `EditResult` now carries `patch: EntityPatch` filled by `Model::apply` via `diff(before, after)`; `undo()`/`redo()` return `Option<EditResult>` with the transition patch (contract `edit`/`history` rows).
- **T033** rule 7 complete: `sourceIn + duration × speed ≤ media duration` (exact `i128` cross-multiplication; images unlimited) and `freezeAt` within the media duration, from the in-model probe.
- **T034** rule 8 complete: at most one transition per edit point, and `0 < d ≤` available source handles per alignment (A `d/2` center / `d` start / `0` end, B mirrored, ceil for odd `d`; only media clips with resolving media are bounded).
- **T035** rule 15's `project file ≤ 256 MB` limit as the O(1) `validate::check_file_size` first-pass hook for the WP-03 loader.
- **T036** `Rational` invariants enforced on deserialization (rejects `den = 0`, normalizes to reduced form) — untrusted JSON cannot inject non-canonical values.
- **T037** view state is never part of undo history (03 §3.4.11): `undo`/`redo`/`step` carry the current root's `ui` into restored roots.
- **T038** `ParamTarget` + `ParamIntrinsic` per 04 §4.4, exported to TS bindings.
- **T039** the migration driver validates its result (parses as `Project` + every 03 §3.5 rule) and fails E1005 naming the first problem.
- **T040** undo/redo exact-inverse equality proven at 200 and 1,000 steps, incl. multi-step redo ordering after a `step(i)` jump.
- **T041** rich patch-application equivalence: a multi-kind batch patch applied to a normalized copy reproduces the new root per entity kind. **This proof caught a real `diff` defect** (a removed track's clips were pushed to `remove.clips` twice) — fixed.
- **T042** rule 1's id walk covers clip markers and track effects.

## Review fixes (F-01..F-20, 2026-11-03, owner: "fix all findings including low")

The reviewer subagent (REQUEST CHANGES, 3 HIGH / 6 MEDIUM / 11 LOW) re-reviewed `origin/main...HEAD`; all 20 findings were fixed with doc-exact remedies:

- **F-01 (HIGH)** `duplicateSequence` deep re-ids every nested entity (tracks, clips, effects, masks, clip markers, transitions incl. endpoint remapping, caption tracks/items) and remaps link/group ids per distinct original group — populated-sequence duplication test asserts id disjointness and clean validation.
- **F-02 (HIGH)** `check_nesting` rewritten: white/grey/black cycle DFS (E1006 only on real cycles; depth undefined on cyclic graphs) + memoized longest-chain depth ≤ 8; diamond-reuse fixture proves legitimate sequence reuse is accepted.
- **F-03 (HIGH)** `createSequence` resolves the full 12 §12.8 preset table (SQP-02..SQP-12 dimensions at 30 fps/48 kHz/stereo/Rec.709), rejects SQP-13-without-format and unknown preset ids (E1003); per-preset dimension tests.
- **F-04 (M)** E1102 → severity `info` per 12 §12.13. **F-05 (M)** E9005 added (`error`) with `AppError::PathRejected`; all SEC-04 denials use it. **F-06 (M)** nested `extra` maps on 15 persisted structs (research D19) + nested-level unknown-field round-trip tests. **F-07 (M)** `ALVS_PROPTEST_CASES` is now truly raise-only (clamped ≥ 10,000). **F-08 (M)** proptest strategies cover the full WP-02 op set with generated payloads. **F-09 (M)** rule 9 + the 10,000-keyframe limit walk every `AnimatableValue` surface (intrinsic, audio, masks, generator, titles, track/master/transition params). 
- **F-10..F-20 (LOW)** skipped drop-frame labels canonicalize forward (ST 12-1) with 29.97/59.94 tests; the dirty-range dedup actually fires now (and marker-only changes no longer emit a spurious full extent — markers are metadata per D10); dead code removed and E1104 emitted once as an envelope; `updateMarker.note` optional (never erases); `new_project_with` takes the caller's `SavedBy`; rule 12 covers clip markers; frame-rate changes snap transition durations; `renameProject` enforces the 1–200 character name rule; research D18 records `alvs-core`'s `schemars`/`ts-rs`; `EditOp` is exported to the TS surface (61 bindings / 60 export tests); STATUS row is `in-review`.

Final state after fixes: **195/195 workspace tests** (27 core + 168 model), clippy/fmt clean, verify.ps1 all available gates PASS (bindings-drift re-verified after committing the regenerated bindings), coverage above.

## Final gate table (`pwsh -File tools/verify.ps1`)

```
Gate              Result
----              ------
docs              PASS
commits           PASS
rust-format       PASS
rust-lint         PASS
dependency-policy PASS
crate-rules       PASS
placeholders      PASS
rust-test         PASS
bindings-drift    PASS
ui-typecheck      SKIPPED (script not defined yet — WP-04/05)
ui-lint           SKIPPED (script not defined yet — WP-04/05)
ui-test           SKIPPED (script not defined yet — WP-04/05)
i18n              SKIPPED (script not defined yet — WP-04/05)
```

## Honesty ledger (not verified / deferred)

- Nightly 1,000,000-case proptest scale: not run (no nightly job yet; knob is raise-only).
- Migration registry is intentionally empty at `schemaVersion = 1`; real steps first ship with the version that bumps the schema (WP-03 owns the `.bak` backup write around the driver).
- Rule 14's file-existence/SHA-256 half is a persistence-layer check (WP-03); the in-memory size limits — including the O(1) `check_file_size` hook that WP-03's loader calls on the raw document length — are enforced and tested here.
- The 512 MB undo budget is enforced on a deterministic structural size estimate (ADR-009), not allocator measurements — exact-boundary eviction is tested at 200/300-step synthetic roots.
- Quickstart §5's filter was corrected to the real test name (`random_edit_sequences_hold_invariants`) — nextest matches test names, not binary names; fixed at the source in `quickstart.md`.
- **Generated-file SPDX policy (review F-19)**: the bindings under `app/src/ipc/bindings/` are machine-written ts-rs artifacts whose provenance is the ts-rs banner; 02 §2.14's source-file rule is applied to hand-written sources. Post-processing generated files to add headers would break the byte-stable drift gate (the generator would produce different bytes than committed), so they are exempt. `EditResult` is not exported to TS yet: its wire shape (`revision, patch, label, warnings: ErrorEnvelope[]`) is built by the app layer (WP-03/05) from these types — `EditOp`, the wire *input*, is exported.
