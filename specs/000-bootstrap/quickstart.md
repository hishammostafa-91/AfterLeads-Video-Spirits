# Quickstart: WP-00 — Workstation and repository bootstrap

**Date**: 2026-11-02 · **Spec**: [spec.md](spec.md) · **Plan**: [plan.md](plan.md)

End-to-end validation of the skeleton. Every scenario maps to a success criterion in the spec and produces evidence for the pull request (G-09). Prerequisite for all scenarios: a Windows 11 x64 workstation prepared per docs/10 §10.2.

## Scenario A — Workstation green (SC-001)

**Prerequisites**: tools installed per docs/10 §10.2 (Git, PowerShell 7, Rustup, Node 26 LTS, pnpm 12.x, Python + uv, GitHub CLI, vcpkg, OpenCode, Spec Kit).

```powershell
pwsh -File tools/setup/Test-DevEnvironment.ps1
```

**Expected**: every row prints PASS (no FAIL, no WARN) and the final line is `Workstation ready.`
**Evidence**: paste the full table into the PR.

## Scenario B — Fresh clone builds with the pinned toolchain (SC-002, SC-006)

**Prerequisites**: git configured; rustup present (the exact toolchain installs itself from `rust-toolchain.toml` on first cargo command).

```powershell
git clone https://github.com/hishammostafa-91/AfterLeads-Video-Spirits.git <folder>
Set-Location <folder>
pwsh -File tools/git/Install-Hooks.ps1
cargo build --workspace --locked
```

**Expected**: `Install-Hooks.ps1` confirms the hooks; the build exits 0 having compiled the eleven empty crates and the app crate.
**Evidence**: the exit-0 build tail (crate list) in the PR.

## Scenario C — Application scaffold builds and opens with the exact identity (SC-003)

**Prerequisites**: Scenario B clone; pnpm install already done in the scaffold step of implementation.

```powershell
pnpm -C app build          # exits 0
pnpm -C app tauri dev      # opens a window
```

**Expected**: the production build exits 0; the dev run opens an empty window titled exactly `AfterLeads-Video-Spirits` (docs/00 §0.5).
**Evidence**: build exit status plus a screenshot of the window title in the PR.

## Scenario D — Local gates pass on the skeleton (SC-004)

```powershell
pwsh -File tools/verify.ps1
python tools/docs/check_docs.py
```

**Expected**: `verify.ps1` prints its gate table — `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test` PASS; the remaining gates SKIPPED with the script's own reasons (see plan.md D7) — and exits 0 with `All available gates passed.`; `check_docs.py` exits 0.
**Evidence**: the gate table in the PR.

## Scenario E — Main is protected (SC-005, manual)

**Prerequisites**: a clone with hooks installed (Scenario B). Performed once, on any clone, without changing anything: the attempt must fail, and the failed attempt leaves nothing behind.

```powershell
git switch main
git commit --allow-empty -m "test: hook rejection evidence"   # NOT signed, must be rejected
# expected: pre-commit rejects with: committing on 'main' is not allowed (G-03)
git switch chore/000-bootstrap
```

**Expected**: `.githooks/pre-commit` rejects the commit (exit 1) with the G-03 message; no commit is created.
**Evidence**: the hook's rejection output in the PR. If the commit succeeds, stop and report — that is a gate failure (G-03, G-08).

## Scenario F — STATUS and constitution hygiene (supporting)

```powershell
git diff .specify/memory/constitution.md    # empty: constitution preserved (FR-003)
```

**Expected**: empty output.
**Evidence**: noted in the PR alongside the Spec Kit initialization commit (`311a773`).

## Out of scope (verified absent, not tested)

CI workflows (WP-01), FFmpeg/media I/O (WP-06), real UI panels and localization keys (WP-04/05), dependency notices (script-generated with packaging, WP-28). Their absence in this package is required by the card's "Not included" line — nothing here validates them.
