# Evidence — WP-02 Core types and project model

Assembled per G-09 and task T031 on 2026-11-03, branch `feat/002-core-model` (20 commits over `main`). All commands were run on the final branch state unless noted.

## Before-state (T001, captured before any WP-02 change)

- `pwsh -File tools/verify.ps1` exited 0 with the WP-01-final gate table; `bindings-drift` was **SKIPPED** because `app/src/ipc/bindings/` did not exist yet.
- Both crates built as empty libraries carrying only `#![deny(missing_docs)]` and SPDX headers.
- `Cargo.lock` carried serde, serde_json, thiserror, and uuid (app transitives) but **not** schemars, ts-rs, or proptest — all three were added by this package at the pinned, crates.io-verified versions recorded in `specs/002-core-model/research.md` (D2–D4).

## Quickstart §1–§8 (final run, `specs/002-core-model/quickstart.md`)

| § | Command | Result |
|---|---|---|
| 1 | `cargo nextest run -p alvs-core --locked` | **24/24 pass** — flicks table, custom-rate validity, rational reduction, path policy (SEC-04), and the T-PLY-005 timecode suite |
| 2–4 | `cargo nextest run -p alvs-model --locked` | **143/143 pass** — entity round-trips incl. unknown-field survival, all fifteen 03 §3.5 rule groups with positive+negative fixtures, size-limit rejection, JSON Schema, edits/undo/patches, migrations |
| 5 | `cargo nextest run -p alvs-model --locked random_edit_sequences_hold_invariants` | **1/1 pass** — see below |
| 6 | `cargo test -p alvs-model export_bindings --locked` + `git status --porcelain -- app/src/ipc/bindings` | **53/53 pass; working tree clean** (no drift) |
| 7 | `cargo llvm-cov nextest -p alvs-core/-p alvs-model --locked --no-tests=pass --summary-only` | both ≥ 80% — see below |
| 8 | `pwsh -File tools/verify.ps1` · `python tools/docs/check_docs.py` · `cargo metadata --no-deps` | **all available gates PASS**; docs checks pass; `alvs-*` crates declare no `[[bin]]` targets (the only workspace binary is the Tauri GUI app itself) — `cli`-free per FR-018 |

Workspace total: `cargo nextest run --workspace --locked` → **167/167 pass**.

## Suites by T-ID

- **T-PLY-005** (drop-frame timecode, 09 §9.x): `alvs-core` time suite — format∘parse round-trip at every 12 §12.9 rate (drop-frame `;` at 29.97/59.94), typed `+/-` relative entry, frames/seconds forms. Green in §1's 24 tests.
- **T-PRJ-005** (project rules): sequence create/duplicate/rename/delete incl. last-sequence rejection and default/active reference switching; frame-rate change snapping with adjusted-item reporting (E1104 data); E1102 media-in-use protection; bin delete re-parenting; batch atomicity as one undo step — `tests/apply.rs` + `edit.rs` tests. Green.
- **T-PRJ-006** (undo/redo): ADR-009 Arc-root engine — undo/redo exact inverses over 50 steps, deterministic-budget eviction (≥ 200 floor, ≤ 1,000 cap, 512 MB estimate), History-panel step jump — `history.rs` tests. Green.
- **T-PRJ-007** (migrations, 03 §3.9.6): forward-only driver with v1 fixture pair round-trip (`tests/fixtures/migrations/v1_in.json`/`v1_out.json` via `tests/migrations.rs`), newer-`schemaVersion` read-only outcome with unknown fields kept, missing-step E1005 and failing-step paths via cfg(test) synthetic steps. Green.

## Property suite (SC-005)

- CI scale **10,000 random sequences × 50 ops**: **1 test, pass, 13.6 s wall** (run `Summary [13.141s] 1 test run: 1 passed`), far inside the ≤ 25-minute CI budget (research D13/F10).
- Invariant checked at every step: the step either applies (model still valid — `Model::apply` validates after each op in tests) or rejects with a documented code (E1102/E0002/E1003/E1006); the end state validates clean.
- `ALVS_PROPTEST_CASES` raises the case count only (default 10,000; parsed value below the default is ignored) — G-08 compliant; the 1,000,000-case nightly scale was **not** exercised (no nightly full-matrix job yet; deferred per research D13).

## Coverage (SC-006, NFR-MNT-01 — gate ≥ 80% line coverage)

| Crate | Regions | Functions | Lines |
|---|---|---|---|
| `alvs-core` | 968 / 45 missed = **95.35%** | 92.19% | 572 / 35 missed = **93.88%** |
| `alvs-model` | 4,385 / 158 missed = **96.40%** | 95.42% | 3,031 / 84 missed = **97.23%** |

Commands exactly as in quickstart §7 (`--no-tests=pass --summary-only`). Standing CI enforcement is deferred per research D14.

## TypeScript bindings and drift (SC-006, FR-015)

- **54 generated `.ts` files** committed under `app/src/ipc/bindings/`; every flicks field typed `number` (03 §3.2.6); `extra` maps present as `JsonValue` index signatures (owner decision F2 / research D16).
- **53 `export_bindings` tests** regenerate byte-identical output: `cargo test -p alvs-model export_bindings --locked` → 53/53, then `git diff --exit-code -- app/src/ipc/bindings` → clean.
- CI gate added to `rust-lint` (`ci.yml`) per WP-01's contract assignment (F1): the same export test + `git diff --exit-code`, reported independently (`if: always()`). Local `verify.ps1` now shows **bindings-drift PASS** (target-presence activation confirmed).

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
- Rule 14's file-existence/SHA-256 half and the §3.5.15 on-disk file-size half are persistence-layer checks (WP-03); the in-memory size limits are enforced and tested here.
- The 512 MB undo budget is enforced on a deterministic structural size estimate (ADR-009), not allocator measurements — exact-boundary eviction is tested at 200/300-step synthetic roots.
- Quickstart §5's filter was corrected to the real test name (`random_edit_sequences_hold_invariants`) — nextest matches test names, not binary names; fixed at the source in `quickstart.md`.
