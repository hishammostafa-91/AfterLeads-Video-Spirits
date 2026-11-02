# Quickstart: validating WP-02 — core types and project model

End-to-end validation guide mapping the card's acceptance to commands and expected outcomes. Prerequisites: verified workstation (tools incl. `cargo-nextest`, `cargo-llvm-cov`), branch `feat/002-core-model`.

## 1. Core primitives incl. drop-frame timecode (T-PLY-005, SC-001)

```powershell
cargo nextest run -p alvs-core --locked
```

Expected: all tests pass — the literal flicks table of 03 §3.3.1 asserted per rate, custom-rate validity, rational reduction, path-policy cases (SEC-04), and the timecode suite: format∘parse round-trip at every 12 §12.9 rate (drop-frame `;` at 29.97/59.94), typed `+/-` relative entry, frames/seconds forms.

## 2. Model types, validation, serialization, schema (SC-006 partial)

```powershell
cargo nextest run -p alvs-model --locked
```

Expected: entity round-trip fixtures (incl. unknown-field survival via `extra`), every 03 §3.5 rule with positive+negative fixtures, size-limit rejection (malicious-file defense), JSON Schema generation.

## 3. Edits, undo/redo, patches (T-PRJ-005, T-PRJ-006, SC-002/SC-003)

Included in §2's run (same crate). Expected specifically: sequence create/duplicate/rename/delete rules (last-sequence rejection), frame-rate change snapping with adjusted-item reporting (E1104 data), E1102 media protection, bin delete re-parenting, batch atomicity; undo/redo equality over 200–1,000 steps, budget eviction order with the ≥ 200 floor; `EntityPatch` application-equivalence property.

## 4. Migrations (T-PRJ-007, SC-004)

Included in §2. Expected: fixture pairs round-trip through the migration driver; newer-`schemaVersion` file ⇒ read-only outcome (E1004) with unknown fields kept.

## 5. Property suite at CI scale (SC-005)

```powershell
cargo nextest run -p alvs-model --locked proptest
```

Expected: 10,000 random sequences × 50 ops green (invariants hold after every applied step; rejections are documented codes). **Record the measured duration in the PR evidence** against the ≤ 25-minute CI budget (research D13/F10); the `ALVS_PROPTEST_CASES` knob exists to raise the scale for the future nightly run, never to lower the CI scale (G-08). The nightly 1,000,000-scale runs when the nightly full-matrix job exists.

## 6. TypeScript bindings + drift gates (SC-006, FR-015)

```powershell
cargo test -p alvs-model export_bindings --locked
git status --porcelain   # committed bindings unchanged
pwsh -File tools/verify.ps1   # bindings-drift now PASS (activates on target presence)
```

Expected: bindings regenerate byte-identical into `app/src/ipc/bindings/`; every flicks field typed `number` (03 §3.2.6); `extra` maps present as index signatures (research D16). The **CI drift step** (added to `rust-lint` by this package, per WP-01's contract — research D15) runs the same export test + `git diff --exit-code` and must be green on the PR.

## 7. Coverage ≥ 80% (SC-006, NFR-MNT-01)

```powershell
cargo llvm-cov nextest -p alvs-core --locked --no-tests=pass --summary-only
cargo llvm-cov nextest -p alvs-model --locked --no-tests=pass --summary-only
```

Expected: line coverage ≥ 80.0% for both crates; numbers recorded in `specs/002-core-model/evidence.md` (enforcement threshold is a quality-gates follow-up, research D14).

## 8. Gates and `cli`-free (FR-018)

```powershell
pwsh -File tools/verify.ps1
python tools/docs/check_docs.py
cargo metadata --no-deps --format-version 1  # then confirm: no [[bin]] targets in alvs-core / alvs-model
```

Expected: all available gates PASS (`bindings-drift` newly active per §6; UI gates still SKIPPED until WP-04/05); neither crate declares a binary target or CLI dependency — `cli`-free per the card.

## References

- API surface: [contracts/crate-apis.md](contracts/crate-apis.md) · entity map: [data-model.md](data-model.md) · decisions: [research.md](research.md)
