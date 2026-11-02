# Tasks: WP-01 — CI and quality gates

**Input**: Design documents from `/specs/001-ci-quality-gates/` (spec.md, plan.md, research.md, data-model.md, contracts/ci-workflows.md, contracts/tools.md, quickstart.md)

**Prerequisites**: plan.md (approved by the owner, OD-08, 2026-11-02, after independent review), spec.md, research.md (D1–D18, all facts verified), data-model.md, contracts/, quickstart.md

**Tests**: No test-writing tasks — the WP-01 card defines acceptance as gate outcomes on CI and recorded baselines (T-SEC-001, T-QA-002; docs/10 §10.7). Each verification task names the command and its required outcome. The Python tools carry no unit tests of their own: they are themselves checkers, exercised by running them (positive path locally, failing paths in the on-GitHub demo of T023).

**Organization**: Tasks are grouped by user story (spec.md US1–US5) to enable independent implementation and verification of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Every task names concrete file paths

## Path Conventions

Repository root (docs/02 §2.13): `.github/workflows/` for CI, `deny.toml` at the root (card deliverable), `tools/` for first-party checkers (CMP-14). All paths below are repository-relative. All action references use the commit SHAs of research.md D1 — never version tags.

## Phase 1: Setup (Verified baseline — evidence before change)

**Purpose**: Capture the pre-implementation state so every later activation is provable (constitution III, G-09).

- [x] T001 Run `pwsh -File tools/verify.ps1` and save the gate table (expected today: `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test` PASS; `dependency-policy`, `crate-rules`, `placeholders`, `bindings-drift`, `ui-typecheck`, `ui-lint`, `ui-test`, `i18n` SKIPPED with the script's own reasons) as the before-state for the evidence pack (tools/verify.ps1)
- [x] T002 [P] Confirm the CI tool baseline on the workstation: `cargo deny --version` (0.20.2), `cargo nextest --version` (0.9.150), `cargo llvm-cov --version` all succeed, and `tools/setup/Test-DevEnvironment.ps1` line 73 checks exactly these three (research D1) (tools/setup/Test-DevEnvironment.ps1)
- [x] T003 [P] Confirm the clean slate: `.github/workflows/` absent, `deny.toml` absent, `tools/check_deps.py`, `tools/check_placeholders.py`, `tools/check_npm_licenses.py` absent — every card deliverable is new (card; plan Project Structure)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The workflow container every job lives in. No user-story label by design.

**⚠ CRITICAL**: No user-story job can be added until this phase is complete.

- [x] T004 Create `.github/workflows/ci.yml`: workflow `name: CI`; `on.pull_request` (branches: `[main]`) and `on.push` (branches: `[main]`); `concurrency` group `${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true`; top-level `permissions: contents: read`; and the `docs` job — `runs-on: ubuntu-latest`, one step using `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` then `python3 tools/docs/check_docs.py` (research D2, D3; contracts/ci-workflows.md) — the job makes the file a valid workflow from the first commit (.github/workflows/ci.yml)

**Checkpoint**: A valid, trigger-complete workflow with the first of the eight OPS-02 check names.

---

## Phase 3: User Story 1 — Every change verified automatically by PR CI (Priority: P1) ⭐ MVP

**Goal**: The core verification jobs — `rust-test`, `ui`, `build`, `e2e-smoke` — complete with skeleton-applicable steps (spec FR-001, FR-002, FR-007, FR-008, FR-009, FR-010; the `rust-lint` job lands with its policy steps in Phase 4, `commits`/`dco` in Phase 5 — FR-002's full eight-name set completes with those phases).

**Independent Test**: quickstart.md §4 partially — locally, the zero-test coverage pipeline is proven end to end (T009); the on-GitHub green run requires the push and closes with the final tree.

### Implementation for User Story 1

- [x] T005 [US1] Add the `rust-test` job to `.github/workflows/ci.yml` — `runs-on: windows-2025`; `rustup show` (installs the pinned 1.99.1 toolchain from `rust-toolchain.toml` — explicit in every cargo job, uniform with T007/T013/T020, research D5); install `cargo-nextest` and `cargo-llvm-cov` via `taiki-e/install-action@1919c4a95903d2c33e4a221102edca87309fc14b # v2.89.0` (`tool: cargo-nextest,cargo-llvm-cov`); `rustup component add llvm-tools-preview`; step with `env: ALVS_FORCE_WARP: 1` running `cargo llvm-cov nextest --workspace --locked --no-tests=pass --lcov --output-path lcov.info` (the `--no-tests=pass` adjustment is verified in T009 — plain invocation exits 4 on the zero-test workspace, research D6); upload `lcov.info` via `actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2` (research D3, D6; contracts/ci-workflows.md) (.github/workflows/ci.yml)
- [x] T006 [US1] Add the `ui` job — `runs-on: windows-2025`; `pnpm/action-setup@ea17c68df8912ef543352723c149a84f56e3d413 # v6.1.0` with `version: 12.16.1`; `pnpm install --frozen-lockfile`; `pnpm -C app build` (the skeleton's type-correctness proof; npm-licenses step is appended in T013) (research D3, D8) (.github/workflows/ci.yml)
- [x] T007 [US1] Add the `build` job — `runs-on: windows-2025`; `rustup show` (installs the pinned 1.99.1 toolchain from `rust-toolchain.toml`); `cargo build -p afterleads-video-spirits --locked` (debug profile; the executable-start check is a later WP's step) (research D3, D5) (.github/workflows/ci.yml)
- [x] T008 [US1] Add the `e2e-smoke` job — `runs-on: windows-2025`; a single step that prints that it is the card-sanctioned no-op reporting success until WP-05 and exits 0 (research D9; spec FR-010) (.github/workflows/ci.yml)
- [x] T009 [US1] Verify the zero-test coverage pipeline locally (research D6): `cargo llvm-cov nextest --workspace --locked --lcov --output-path $env:TEMP\lcov.info` → exit 0, `0 passed`, lcov present (empty/zero-coverage — the documented baseline, not a false green); if llvm-cov misbehaves with no test binaries, adjust the invocation — never drop the gate (G-08); record the output for the evidence pack (Cargo.toml)

**Checkpoint**: Five of the eight check names exist; the coverage artifact pipeline is proven on the workstation.

---

## Phase 4: User Story 2 — Dependency, license, and advisory policy enforced (Priority: P2)

**Goal**: `deny.toml` + `tools/check_npm_licenses.py` + `.github/dependabot.yml` + the policy steps wired into CI; `dependency-policy` gate activates locally; T-SEC-001 baseline recorded (spec FR-011, FR-012, FR-013, FR-016; SC-004).

**Independent Test**: quickstart.md §2 — `cargo deny check` green (wrapper allowances satisfied) and `check_npm_licenses.py` green on the committed lockfiles.

### Implementation for User Story 2

- [x] T010 [P] [US2] Create `deny.toml` per research D12 exactly: `[licenses] allow` = the 24 SPDX ids of 05 §5.6; `[advisories]` defaults; `[bans] multiple-versions = "warn"` and the HTTP-client deny set — `reqwest` `wrappers = ["tauri"]`, `hyper` `wrappers = ["reqwest", "hyper-util"]`, `ureq`/`isahc`/`surf`/`attohttpc`/`minreq`/`curl`/`curl-sys` unconditional (per-direct-parent-edge semantics verified against the lockfile: `tauri`→`reqwest` line 3099, `reqwest`→`hyper` 2499, `hyper-util`→`hyper` 1332); `[sources] unknown-registry = "deny"`, `unknown-git = "deny"`; SPDX header comment. Then run `cargo deny check` → all four checks green; if any wrapper edge fails, fix the config against the lockfile — never widen the allow list (G-08) (deny.toml)
- [x] T011 [P] [US2] Create `tools/check_npm_licenses.py` per contracts/tools.md: SPDX header, standard library only, runs `pnpm -C app licenses list --json --prod`, verifies every license against the 05 §5.6 allowlist, exit 0/1/2 convention, findings one per line; verify locally after `pnpm install --frozen-lockfile` → exit 0 "all npm licenses allowlisted" (tools/check_npm_licenses.py)
- [x] T012 [P] [US2] Create `.github/dependabot.yml`: version 2; three weekly update sets — `cargo` at `directory: "/"`, `npm` at `directory: "/app"`, `github-actions` at `directory: "/"` (research D11; 05 §5.6 rule 5) (.github/dependabot.yml)
- [x] T013 [US2] Wire the policy steps into CI in one edit of `.github/workflows/ci.yml` (execute after T014 and T015 have created the tools these steps invoke — see Dependencies): add the `rust-lint` job (`runs-on: windows-2025`; `rustup show`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; install `cargo-deny` via the pinned install-action; `cargo deny check`; `python tools/check_deps.py` — tool created in T014; `python tools/check_placeholders.py` — tool created in T015) and append `python tools/check_npm_licenses.py` to the `ui` job after the frozen install (research D3) (.github/workflows/ci.yml)
- [x] T014 [P] [US2] Create `tools/check_deps.py` per contracts/tools.md and research D13: enforces docs/02 §2.4 rules 1–4 (CMP-row dependencies, no cycles; exclusive crates — `ffmpeg-next` only `alvs-media`, `wgpu` only `alvs-render`, `cpal` only `alvs-audio`, `rusqlite` only `alvs-store`, `whisper-rs` only `alvs-stt`, `tauri`/`webview2-com` only `app/src-tauri`; UI imports Rust only via `app/src/ipc/bindings/`; `#![deny(missing_docs)]` in `alvs-core`/`alvs-model`/`alvs-formats`); SPDX header, standard library only; verify → exit 0 "all dependency rules OK" (tools/check_deps.py)
- [x] T015 [P] [US2] Create `tools/check_placeholders.py` per contracts/tools.md and research D14: scans first-party trees (`crates/`, `app/src`, `app/src-tauri/`, `tools/`, `.githooks/`, `tests/` when present; excludes `app/src/ipc/bindings/`) for the 06 §6.13 rule-4 markers — placeholder macros, `TODO`/`FIXME` comments, mocked production paths, commented-out code — with the rule's exemption for test code and the mock backend (`tests/` trees, `*_test.rs`, `*.test.ts(x)`, `#[cfg(test)]` regions, mock-backend module); marker strings assembled by concatenation so the tool never matches itself; verify → exit 0 "no unfinished-work markers" (tools/check_placeholders.py)
- [x] T016 [US2] Run `pwsh -File tools/verify.ps1` → `dependency-policy`, `crate-rules`, `placeholders` now PASS (verify.ps1 unedited — activation is by target presence, research D16; FR-018, SC-006) and capture the T-SEC-001 baseline outputs (`cargo deny check` and `python tools/check_npm_licenses.py`) into `specs/001-ci-quality-gates/evidence.md` (SC-004; 09 §9.10 T-SEC-001) (specs/001-ci-quality-gates/evidence.md)

**Checkpoint**: Six of eight check names; policy green locally on the committed lockfiles; T-SEC-001 baseline recorded.

---

## Phase 5: User Story 3 — Commit discipline enforced remotely (Priority: P3)

**Goal**: The `commits` and `dco` required checks, sharing `tools/git/check_commits.py` via `--only` (spec FR-004, FR-005; FR-002's set now complete at eight).

**Independent Test**: quickstart.md §4 partially — locally the three invocation modes return the documented results (T018); the failing side is demonstrated on GitHub in T023.

### Implementation for User Story 3

- [x] T017 [US3] Extend `tools/git/check_commits.py` with the optional `--only conventional|dco` argument per contracts/tools.md and research D4: no flag = both rules, byte-identical behavior to today (verify.ps1's caller unchanged); invalid `--only` value → exit 2 with usage; SPDX header and module docstring updated (tools/git/check_commits.py)
- [x] T018 [US3] Add the `commits` and `dco` jobs to `.github/workflows/ci.yml`: both `runs-on: ubuntu-latest`, both check out with `fetch-depth: 0` (the range needs full history — research D3/F5), then run `python3 tools/git/check_commits.py origin/main..HEAD --only conventional` and `--only dco` respectively (.github/workflows/ci.yml)
- [x] T019 [US3] Verify locally: `python tools/git/check_commits.py origin/main..HEAD` → "all commits OK" (default unchanged); `--only conventional` and `--only dco` on the same range → OK; `--only bogus` → exit 2 (tools/git/check_commits.py)

**Checkpoint**: All eight OPS-02 check names exist in `ci.yml`; the commit contract is locally verified in all modes.

---

## Phase 6: User Story 4 — Nightly watch (Priority: P4)

**Goal**: `nightly.yml` with the two card-scoped jobs, fully self-contained (spec FR-011; research D10).

**Independent Test**: quickstart.md §7 — manual dispatch runs both jobs (`dependency-audit` green, `docs-links` informational).

### Implementation for User Story 4

- [x] T020 [US4] Create `.github/workflows/nightly.yml` per research D10 and contracts/ci-workflows.md: `on.schedule` `17 2 * * *` and `27 2 * * 0` plus `workflow_dispatch`; `permissions: contents: read`; `dependency-audit` (`runs-on: windows-2025`; `if: github.event_name == 'workflow_dispatch' || github.event.schedule == '17 2 * * *'`; pinned checkout; `rustup show`; install `cargo-deny` via the pinned install-action; `cargo deny check advisories`; `pnpm/action-setup` pinned with `version: 12.16.1`; `pnpm install --frozen-lockfile`; `pnpm audit --prod`); `docs-links` (`runs-on: ubuntu-latest`; `if: github.event_name == 'workflow_dispatch' || github.event.schedule == '27 2 * * 0'`; pinned checkout; `lycheeverse/lychee-action@e7477775783ea5526144ba13e8db5eec57747ce8 # v2.9.0` over `docs/`; job-level `continue-on-error: true`) (.github/workflows/nightly.yml)
- [x] T021 [US4] Confirm the G-19 documentation is in place and matches the implementation: docs/08 §8.3.2 carries the second-schedule note (commit `4800224`) and docs/00 set version is 1.0.4; `python tools/docs/check_docs.py` → exit 0 (docs/08-operations.md)

**Checkpoint**: Nightly audit and weekly link watch fully specified; canonical doc synced.

---

## Phase 7: User Story 5 — Fast CI, demonstrated gates, owner handoff (Priority: P5)

**Goal**: The ≤ 25-minute budget served by caching; the four deliberately failing commits evidenced on GitHub; the T-QA-002 baseline; the OPS-02 checklist in the PR (spec FR-019, FR-020, FR-021; SC-002, SC-003; research D7, D17, D18).

**Independent Test**: quickstart.md §5–§6 and §8 — the failing jobs observed red, the warm re-run timed, the checklist delivered.

### Implementation for User Story 5

- [x] T022 [US5] Add caching to `.github/workflows/ci.yml` per research D7: `actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0` — in `rust-lint`, `rust-test`, `build` for `~/.cargo/registry`, `~/.cargo/git`, `target/` keyed by `hashFiles('Cargo.lock')` with `restore-keys` prefix fallback; in `ui` for the pnpm store keyed by `hashFiles('pnpm-lock.yaml')`; no vcpkg cache (no `vcpkg.json` until WP-06) (.github/workflows/ci.yml)
- [x] T023 [US5] Failure demonstration per research D17 (G-04 stacking approved by the owner 2026-11-02): from the tip of `ci/001-ci-quality-gates` create `ci/001-demo-failures`; add four commits — (a) a rustfmt violation, (b) code tripping clippy `-D warnings`, (c) an unfinished marker caught by `tools/check_placeholders.py`, (d) a commit without `-s` — all with Conventional-Commit first lines; push; open a **draft** PR to `main` whose body states it is stacked on `ci/001-ci-quality-gates` for evidence only, never to be merged; let one run finish; capture the red `rust-lint` (three failed steps) and red `dco` links plus green `commits` into `specs/001-ci-quality-gates/evidence.md`; close the draft PR; leave the branch for the owner (G-18) (.github/workflows/ci.yml)
- [ ] T024 [US5] After `/wp-finish` pushes the branch and opens the WP-01 pull request: capture the T-QA-002 baseline — let the first run warm the caches, then **Re-run all jobs** and record the re-run's total duration and URL (≤ 25 minutes with warm caches, NFR-MNT-02; 09 §9.10 T-QA-002) into `specs/001-ci-quality-gates/evidence.md` (SC-003) (specs/001-ci-quality-gates/evidence.md)
- [ ] T025 [US5] Complete the WP-01 pull-request body per FR-020/FR-021: the OPS-02 stage-two owner checklist (enable required checks `docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke` after merge — 08 §8.2), the T-SEC-001/T-QA-002 baseline references, the failure-demonstration links from T023, and the SC-001 statement that all checks are green on the PR (.github/pull_request_template.md)

**Checkpoint**: Budget mechanics in place; all four gate failures evidenced; baselines and the owner checklist delivered.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Full local validation, hygiene scans, evidence, and ledger.

- [ ] T026 Run the full quickstart.md validation (§1–§8 as far as executable pre-push; §4–§8 complete after the PR exists via T023–T025); fix anything it catches at the source (specs/001-ci-quality-gates/quickstart.md)
- [x] T027 Pin-hygiene scan (quickstart §3): every `uses:` in `.github/workflows/*.yml` resolves to a full 40-hex commit SHA (the six D1 actions only) and both workflows declare `permissions: contents: read`; any unpinned reference is a defect (FR-017, SC-005) (.github/workflows/ci.yml)
- [x] T028 Run `pwsh -File tools/verify.ps1` → exit 0 with the expected final table: `docs`, `commits`, `rust-format`, `rust-lint`, `dependency-policy`, `crate-rules`, `placeholders`, `rust-test` PASS; `bindings-drift` and the four UI gates SKIPPED (targets arrive WP-02/04/05) (SC-006; FR-018) (tools/verify.ps1)
- [x] T029 Update STATUS.md for WP-01: state stays `in-progress` until merge; next step `finish` (docs/10 §10.10) (STATUS.md)
- [ ] T030 Assemble the pull-request evidence per G-09 in `specs/001-ci-quality-gates/evidence.md`: T001 before-table, T009 zero-test output, T010 deny output, T016 T-SEC-001 baselines, T019 mode checks, T023 failure-demo links, T024 T-QA-002 duration — commands, results, and test IDs (specs/001-ci-quality-gates/evidence.md)
- [x] T031 G-19 final confirmation: the docs/08 §8.3.2 second-schedule note (commit `4800224`) and docs/00 set version 1.0.4 match the shipped behavior; `python tools/docs/check_docs.py` → exit 0; no other canonical-doc change is required by this package (docs/00-index.md)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — baseline capture.
- **Foundational (Phase 2)**: Depends on Phase 1 — **blocks every user story** (all jobs edit the file T004 creates).
- **US1 (Phase 3)**: Depends on Phase 2.
- **US2 (Phase 4)**: Depends on Phase 2; T013 wires steps whose tools T010–T015 create — run T013 after them (single-file sequence inside `ci.yml`).
- **US3 (Phase 5)**: Depends on Phase 2 (T017 is independent of Phase 3/4 files, but lands after them in priority order).
- **US4 (Phase 6)**: Depends on Phase 2 (nightly is a new file; strictly it only needs D1 pins, but the D10 design assumes the tool conventions of D5/D8).
- **US5 (Phase 7)**: Depends on **all** prior phases (T022 edits `ci.yml` last; T023's demo needs the complete tree; T024/T025 need the push/PR).
- **Polish (Phase 8)**: Depends on all stories; T026§4–8 and T030 depend on T023–T025.

### Within Each Story

- Tools before the workflow steps that invoke them (T010–T015 → T013).
- `ci.yml` is a single file: every task touching it is sequential, even across phases.
- Commit after each task or logical group (docs/10 §10.4.4): `ci(ci): …` for workflows and dependabot, `build(deps)` is **not** used (no manifest changes); `feat(tools)`/`chore(tools)` for the check tools — types and scopes from docs/10 §10.4.5, DCO sign-off.

### Parallel Opportunities

- T002, T003 — independent reads (Phase 1).
- T010 (`deny.toml`), T011 (`tools/check_npm_licenses.py`), T012 (`.github/dependabot.yml`), T014 (`tools/check_deps.py`), T015 (`tools/check_placeholders.py`) — five different files (Phase 4), before the sequential T013/T016.

---

## Parallel Example: User Story 2

```text
Task: "Create deny.toml (05 §5.6 policy, D12 wrapper set)"        # T010
Task: "Create tools/check_npm_licenses.py (D15)"                  # T011
Task: "Create .github/dependabot.yml (D11)"                       # T012
Task: "Create tools/check_deps.py (D13)"                          # T014
Task: "Create tools/check_placeholders.py (D14)"                  # T015
# then sequentially: T013 (single ci.yml edit wires rust-lint + ui step), T016 (gate table + baseline)
```

---

## Implementation Strategy

### MVP First (Phases 1–3)

1. Baseline captured (Phase 1), workflow container + `docs` job (Phase 2), core jobs + proven coverage pipeline (Phase 3).
2. **STOP and VALIDATE**: five check names exist; the pipeline runs green locally on the zero-test baseline.

### Incremental Delivery

1. Phase 4 → policy tools + `rust-lint` job + Dependabot; local `dependency-policy` gate active; T-SEC-001 baseline.
2. Phase 5 → `commits`/`dco`; all eight names present.
3. Phase 6 → nightly watch.
4. Phase 7 → caching, failure demos, T-QA-002, OPS-02 checklist (T024/T025 straddle `/wp-finish` by design — the PR must exist).
5. Phase 8 → full validation, evidence pack, ledger, `/wp-finish`.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks; every `ci.yml` task is sequential.
- [Story] labels map to spec.md user stories for traceability.
- No unit-test tasks: the deliverables are checkers and workflows; their verification is execution (locally green; failing paths evidenced on GitHub in T023).
- Every action reference uses the D1 commit SHAs; version tags in `uses:` are a defect (FR-017).
- SPDX headers: the three new Python tools carry the header pair (docs/02 §2.14 enumerates Python among header-bearing source kinds) and `deny.toml` carries an SPDX comment (manifest precedent: `Cargo.toml`, `pnpm-workspace.yaml`); the workflow and Dependabot YAML files carry **no** header — docs/02 §2.14 does not enumerate YAML, and CI configuration follows the repository's headerless-config precedent (`.gitattributes`, `.editorconfig`, `.gitignore`, `.github/pull_request_template.md`).
- Never edit `tools/verify.ps1` or any hook (G-06, G-08; research D16); gates activate by target presence.
- No workspace/npm dependency changes (G-11); CI tools install via the pinned install-action.
- The `e2e-smoke` no-op is the only sanctioned green-without-work step (card); it is visibly marked and replaced in WP-05.
- Stop and ask (G-17) on any discrepancy with the canonical docs or the lockfile; do not improvise.

---

## Phase 9: Convergence

- [ ] T032 After the owner merges WP-01, dispatch the Nightly CI workflow manually (Actions → Nightly CI → Run workflow) and record the run URL and outcomes in specs/001-ci-quality-gates/evidence.md (`dependency-audit` green; `docs-links` informational) — GitHub offers schedule/dispatch only for workflows present on the default branch, so this proof is structurally post-merge (quickstart §7; FR-011) (partial)
