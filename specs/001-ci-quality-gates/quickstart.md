# Quickstart: validating WP-01 — CI and quality gates

End-to-end validation guide. Each item maps a card acceptance criterion to concrete commands, expected outcomes, and the evidence to record (G-09). Prerequisites: a workstation green per `tools/setup/Test-DevEnvironment.ps1` (which verifies `cargo-deny`, `cargo-nextest`, `cargo-llvm-cov` — line 73) on the `ci/001-ci-quality-gates` branch.

## 1. Local gates activate and pass (FR-018, SC-006)

```powershell
pwsh -File tools/verify.ps1
```

Expected: `dependency-policy`, `crate-rules`, and `placeholders` now report **PASS** (previously SKIPPED — they activate automatically when `deny.toml`, `tools/check_deps.py`, `tools/check_placeholders.py` exist; verify.ps1 itself is unedited, research.md D16). All other available gates stay PASS; exit code 0.

Evidence: the gate table output.

## 2. Dependency and license policy is green (FR-013, FR-016, SC-004 — the T-SEC-001 baseline)

```powershell
cargo deny check
pnpm install --frozen-lockfile   # if the store is cold
python tools/check_npm_licenses.py
```

Expected: `cargo deny check` reports advisories/sources/licenses/bans checks passing (0 advisories against the committed `Cargo.lock`; every license on the 05 §5.6 allowlist; no banned HTTP-client crate present); `check_npm_licenses.py` reports all npm licenses allowlisted.

Evidence: both outputs — this is the T-SEC-001 baseline (09 §9.10; NFR-SEC-01).

## 3. Workflow hygiene by inspection (FR-017, SC-005)

```powershell
Select-String -Path .github/workflows/*.yml -Pattern 'uses:\s*\S+@' |
  ForEach-Object { if ($_.Line -notmatch '@[0-9a-f]{40}\s*$') { "UNPINNED: $($_.Line)" } }
```

Expected: no output (every `uses:` pinned to a full commit SHA — the six actions of research.md D1); workflow files declare `permissions: contents: read`.

## 4. CI runs green on the PR (FR-019 first half, SC-001)

Push the branch, open the PR (`/wp-finish` does this), then watch the checks. Expected: all eight checks green — `docs`, `commits`, `dco`, `rust-lint`, `rust-test`, `ui`, `build`, `e2e-smoke` (the no-op explains itself in its log).

Evidence: the PR checks section / run URL.

## 5. Deliberately failing commits fail the right jobs (FR-019 second half, SC-002)

Procedure (research.md D17): from `main`, branch `ci/001-demo-failures`; add four commits — (a) a rustfmt violation, (b) code that trips clippy `-D warnings`, (c) an unfinished marker caught by `tools/check_placeholders.py`, (d) a commit made without `-s`; push, open a **draft** PR, let one CI run finish.

Expected: `rust-lint` red with exactly the three failing steps; `dco` red; every other check reports its own result.

Evidence: failing-step links recorded in `evidence.md` and the PR body; close the draft PR; leave the branch for the owner (G-18).

## 6. T-QA-002 baseline (FR-020, SC-003)

Re-run the WP-01 PR checks (close+reopen the PR, or push an empty-state retrigger per D17) after caches are warm from the first run, and read the total duration from the run summary.

Expected: ≤ 25 minutes wall clock (NFR-MNT-02). Evidence: the run URL and duration recorded in `evidence.md` (state that caches were warm).

## 7. Nightly jobs run on demand (FR-011)

Actions tab → *Nightly CI* → *Run workflow* (manual dispatch exercises both jobs regardless of the weekly link cadence).

Expected: `dependency-audit` green; `docs-links` runs informational (`continue-on-error`). Evidence: run URL (optional — dispatch proves the wiring; the schedule itself proves out over time).

## 8. Owner handoff: OPS-02 stage two (FR-021)

The PR body contains the checklist: after merging, enable required status checks `docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke` on `main` (08 §8.2). This is the owner's action; agents never change repository settings (G-18).

## References

- Workflow/job contract: [contracts/ci-workflows.md](contracts/ci-workflows.md)
- Tool CLI contracts: [contracts/tools.md](contracts/tools.md)
- Configuration entities and policy sets: [data-model.md](data-model.md)
- Decision record with verified pins/versions: [research.md](research.md)
