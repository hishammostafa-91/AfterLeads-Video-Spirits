# Implementation Plan: WP-01 — CI and quality gates

**Branch**: `ci/001-ci-quality-gates` | **Date**: 2026-11-02 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-ci-quality-gates/spec.md`; constraints from the owner's `/speckit.specify` invocation (card docs/10 §10.7; docs/05 §5.6; docs/08 §8.2–8.4; docs/09 §9.1; requirements NFR-MNT-01, NFR-MNT-02, NFR-SEC-01 (gates); tests T-SEC-001, T-QA-002; nothing beyond the canonical documents).

## Summary

Stand up the automated verification backbone exactly as the WP-01 card specifies: pull-request CI (`.github/workflows/ci.yml`) publishing the eight OPS-03 required-check job names with the steps that apply to the current skeleton; nightly CI (`.github/workflows/nightly.yml`) scoped to dependency audit and docs links; `.github/dependabot.yml` (weekly cargo/npm/actions); the `deny.toml` dependency-and-license policy of 05 §5.6 with the SEC-10 HTTP-client ban; three first-party check tools (`tools/check_deps.py`, `tools/check_placeholders.py`, `tools/check_npm_licenses.py`) and the `--only` wiring of `tools/git/check_commits.py`; all actions pinned to commit SHAs; acceptance demonstrated by deliberately failing commits, and the T-SEC-001/T-QA-002 baselines plus the OPS-02 owner checklist recorded in the PR. No release workflow, no later-WP CI jobs (card "Not included").

## Technical Context

**Language/Version**: YAML (GitHub Actions, Dependabot), Python 3 standard library only for the check tools (contracts/tools.md), TOML for `deny.toml`. Rust 1.99.1 stays pinned via `rust-toolchain.toml`; pnpm 12.16.1 in CI matches the workstation.

**Primary Dependencies**: GitHub Actions, all pinned to full commit SHAs (research.md D1, verified 2026-11-02): `actions/checkout` v7.0.1, `actions/cache` v6.1.0, `actions/upload-artifact` v7.0.2, `taiki-e/install-action` v2.89.0, `lycheeverse/lychee-action` v2.9.0, `pnpm/action-setup` v6.1.0. CI-environment Rust tools (installed, not workspace dependencies — G-11 untouched): `cargo-deny` (0.20.2 verified locally), `cargo-nextest` (0.9.150 verified locally), `cargo-llvm-cov`.

**Storage**: N/A — configuration and tooling only (docs/03 not in the card's Read line).

**Testing**: acceptance is gate-based (card): CI green on the PR; four deliberately failing commits fail the right jobs; T-SEC-001 and T-QA-002 baselines recorded (09 §9.10). See [quickstart.md](quickstart.md).

**Target Platform**: GitHub-hosted runners — `ubuntu-latest` (docs/commits/dco) and `windows-2025` (Rust/UI/build/e2e jobs; image verified current, research.md D1). Product platform unchanged: Windows 11 x64.

**Project Type**: repository CI configuration + first-party gate tooling (tools/ is CMP-14, docs/02 §2.4).

**Performance Goals**: pull-request CI ≤ 25 minutes with warm caches (NFR-MNT-02; T-QA-002) — served by per-branch concurrency with cancel-in-progress, `actions/cache` for cargo and pnpm stores keyed by the lockfiles (research.md D2, D7), and prebuilt-binary tool installation (D5).

**Constraints**: everything the workflows run must be a canonical gate command (08 §8.3–8.4); licenses/bans/sources exactly per 05 §5.6; actions SHA-pinned (05 §5.6 rule 5); read-only workflow permissions (08 §8.2); no gate weakened or faked (G-08) — the single sanctioned no-op is `e2e-smoke` until WP-05 (card); no new workspace/npm dependencies (G-11); no repository-settings changes (G-18); LF endings and SPDX headers on the Python tools (02 §2.14).

**Scale/Scope**: 2 workflow files, 1 Dependabot config, 1 `deny.toml`, 3 new Python tools (~100–200 lines each), 1 extended Python tool (`--only` flag), 0 product code changes, 0 dependency-manifest changes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| I. Canonical documentation first | PASS | Every FR cites card/01/05/08/09; every plan decision cites a canonical section (research.md D1–D18); the one spec correction (FR-018) aligned the spec *to* the canonical 08 §8.4 table. |
| II. Scope discipline | PASS | FR-022 bounds scope: no release workflow, no later-WP jobs/steps; job steps limited to existing targets (D3 matrix); nothing from 1.1/1.2. |
| III. Test-first evidence | PASS | Card acceptance is gate-demonstration based: failure demos map each gate to its failing job (SC-002); T-SEC-001/T-QA-002 baselines recorded with command outputs (D18, G-09). |
| IV. Protected main and clean history | PASS | Work on `ci/001-ci-quality-gates` from `main`; failure demos ride a separate draft PR closed unmerged, never touching `main` (D17); commits conventional + DCO. |
| V. Quality gates non-negotiable | PASS | Workflows run the canonical gate commands verbatim (08 §8.3–8.4); `verify.ps1` unedited (D16); no `continue-on-error` except the informational nightly docs-links job (D10). |
| VI. Global and accessible by default | PASS* | No user-facing product strings exist in this package; workflow/job names are infrastructure identifiers, not UI. |
| VII. Private, secure, local-first | PASS | Workflows request `contents: read` only; actions SHA-pinned (supply-chain, 05 §5.6 rule 5); Dependabot is the canonical update path; no secrets, no network at runtime beyond what CI needs. |
| VIII. Simplicity and performance budgets | PASS | Minimal job set with skeleton-applicable steps only; first-party tools reuse the `check_commits.py` pattern; no speculative jobs/abstractions (D3–D16). |
| IX. Open-source integrity | PASS | SPDX + copyright headers on the new Python tools (02 §2.14); no code copied from other products; actions are services consumed by pinned reference, recorded in D1. |
| Technology constraints | PASS | No workspace/npm dependency changes; Rust/pnpm versions untouched (G-11); runner images verified (D1). |

`*` = passes with the documented reading; no violation exists. No Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/001-ci-quality-gates/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output — D1..D18, all external facts verified 2026-11-02
├── data-model.md        # Phase 1 output — configuration entities (no product data)
├── quickstart.md        # Phase 1 output — validation guide per acceptance criterion
├── contracts/
│   ├── ci-workflows.md  # workflow/job/check-name contract (OPS-03/OPS-05/Dependabot)
│   └── tools.md         # CLI contracts of the four check tools
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

What WP-01 creates or touches (existing baseline omitted):

```text
.github/
├── workflows/
│   ├── ci.yml               # OPS-03 pull-request CI — 8 jobs, skeleton-scope steps (D3)
│   └── nightly.yml          # OPS-05 nightly — dependency-audit + docs-links (D10)
└── dependabot.yml           # weekly: cargo "/", npm "/app", github-actions "/" (D11)
deny.toml                    # cargo-deny: 05 §5.6 licenses, advisories, bans, sources (D12)
tools/
├── check_deps.py            # 02 §2.4 rules 1–4 (VER-09) — new (D13)
├── check_placeholders.py    # 06 §6.13 rule 4 markers — new (D14)
├── check_npm_licenses.py    # npm-side 05 §5.6 allowlist — new (D15)
└── git/check_commits.py     # gains optional --only conventional|dco (D4)
```

Not touched: `tools/verify.ps1` (gates activate by target presence, D16), all manifests/lockfiles, all product code.

**Structure Decision**: flat tooling layout in `tools/` and `.github/` per docs/02 §2.13 (CMP-14 "Repository tooling — tools/"; CI files at `.github/workflows/` per 08 §8.3). Not a choice — the canonical layout names every path this package writes.

## Complexity Tracking

No constitution violations; no entries.

---

## Phase 0 — Research

See [research.md](research.md). All technical-context unknowns resolved with citations; **zero NEEDS CLARIFICATION remain**. Every externally sourced fact (action SHAs, runner image availability, tool support, schema fields, local tool versions) was verified against GitHub API/raw sources or the local workstation on 2026-11-02 — G-10.

## Phase 1 — Design

- [data-model.md](data-model.md): no product data (docs/03 not in scope); documents the configuration entities — CI/nightly jobs, action pins, the license policy set, the npm policy, and the shared check-tool result contract.
- [contracts/ci-workflows.md](contracts/ci-workflows.md): the observable workflow contract — triggers, published check names, per-job guarantees, deferred-step ownership, Dependabot sets, pinning rule.
- [contracts/tools.md](contracts/tools.md): CLI contracts for `check_commits.py --only` and the three new tools (exit codes, output forms, scan scope, standard-library-only constraint).
- [quickstart.md](quickstart.md): validation guide mapping every acceptance criterion (green CI, failure demos, T-SEC-001/T-QA-002 baselines, OPS-02 handoff) to commands, expected outputs, and evidence to record.

## Constitution Check — re-evaluation after Phase 1 design

Re-checked after the design artifacts: no change. The design adds configuration and tooling only — no product entities, interfaces, dependencies, or behavior beyond the card; the only sanctioned green-without-work job is the card's `e2e-smoke` no-op; deferred steps are explicitly owned by later work packages (G-02). Principles I–IX and the technology constraints remain PASS.
