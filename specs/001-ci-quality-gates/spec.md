# Feature Specification: WP-01 — CI and quality gates

**Feature Branch**: `ci/001-ci-quality-gates`

**Created**: 2026-11-02

**Status**: Draft

**Input**: User description: "Implement WP-01 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: NFR-MNT-01, NFR-MNT-02, NFR-SEC-01 (gates). Acceptance tests: T-SEC-001, T-QA-002. Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents."

**Canonical basis**: WP-01 card (docs/10-execution.md §10.7, WP-01): "Implements: NFR-MNT-01, NFR-MNT-02, NFR-SEC-01 (gates)". The three NFR rows in docs/01-product.md are quoted where they bind; this package implements their *gate infrastructure*, not their full measurable targets (those bind as code lands in later work packages). Per the constitution (I. Canonical Documentation First) every requirement below cites the card and the canonical sections in its Read line (05 §5.6; 08 §8.2–8.4; 09 §9.1), plus the sections the card's deliverables themselves name (02 §2.4 for `tools/check_deps.py`; 09 §9.10 for the test IDs; 05 §5.7 SEC-10 / ADR-023 for the HTTP-client ban). Nothing is added beyond those documents (G-01, G-02).

## User Scenarios & Testing *(mandatory)*

Actors: the project owner, human contributors, and AI agents. This work package builds no end-user feature; its "users" are the people and processes that keep every later change verifiable (G-08, constitution V).

### User Story 1 - Every change is verified automatically by pull-request CI (Priority: P1)

A contributor (or agent) opens a pull request to `main`; the applicable quality gates run automatically on GitHub-hosted runners, and a change that breaks a gate fails exactly the job responsible for that gate, blocking the merge until it is fixed.

**Why this priority**: Quality gates only become non-negotiable (constitution V, G-08) when they run on every pull request without anyone opting in. This is the foundation milestone M0 depends on ("CI green on `main`", 10 §10.6).

**Independent Test**: CI runs green on the WP-01 pull request itself (card Acceptance); a deliberately broken commit fails the matching job (FR-019).

**Acceptance Scenarios**:

1. **Given** the WP-01 branch in its passing state, **When** a pull request to `main` is opened or updated, **Then** every CI job reports success (card Acceptance: "CI green on the PR").
2. **Given** a pull request containing a formatting violation, **When** CI runs, **Then** the lint job fails while the other jobs report their own results independently (card Acceptance: deliberately failing commits fail the right job).

---

### User Story 2 - Dependency, license, and advisory policy is enforced automatically (Priority: P2)

The owner wants the shipped dependency tree held to the license allowlist and kept free of known high/critical advisories (NFR-SEC-01: "0 known high/critical advisories in shipped dependencies at release"). From this package on, the policy is checked by machines — in pull-request CI via `cargo deny check` and the npm license check, and nightly via the audit job — instead of by manual review.

**Why this priority**: NFR-SEC-01 is a release-blocking NFR implemented by this card; every dependency added from now on must pass the policy from day one (05 §5.6 rule 1).

**Independent Test**: The dependency-policy checks pass green on the committed lockfiles and are recorded as the T-SEC-001 baseline in the pull request (card Acceptance).

**Acceptance Scenarios**:

1. **Given** the repository with its committed `Cargo.lock` and `pnpm-lock.yaml`, **When** the dependency-policy checks run in CI, **Then** they pass: every license is on the 05 §5.6 allowlist, package sources are crates.io/npm only, and no known high/critical advisory is present (T-SEC-001 baseline).
2. **Given** a pull request that adds a dependency whose license is not on the allowlist (05 §5.6 "Denied"), **When** CI runs, **Then** the dependency-policy gate fails the pull request.

---

### User Story 3 - Commit discipline is enforced remotely, not only locally (Priority: P3)

The owner receives pull requests whose every commit follows Conventional Commits (10 §10.4.5) and carries a DCO `Signed-off-by:` trailer matching the author email. Local hooks check this today; from this package on, CI re-checks it on every pull request so history stays clean even when a hook was not installed.

**Why this priority**: The local hooks are advisory (a clone without `Install-Hooks.ps1` bypasses them); CI makes G-06/G-07 hygiene structural. It ranks below the build/test gates because a bad message is correctable, while a broken build is not.

**Independent Test**: A deliberately malformed commit message fails the `commits` job; a commit without sign-off fails the `dco` job (card Acceptance: missing sign-off must fail the right job).

**Acceptance Scenarios**:

1. **Given** a pull request containing one commit whose message violates Conventional Commits (10 §10.4.5), **When** CI runs, **Then** the `commits` job fails.
2. **Given** a pull request containing one commit without a `Signed-off-by:` trailer matching the author email, **When** CI runs, **Then** the `dco` job fails.

---

### User Story 4 - Nightly watch over dependencies and documentation links (Priority: P4)

The owner wakes up to an automated nightly report: the dependency audit (`cargo deny check advisories`, `pnpm audit --prod`) and the external documentation link check have run on schedule (02:17 UTC) or on manual dispatch, surfacing advisories and rotting links while the pull-request pipeline stays fast.

**Why this priority**: Nightly jobs watch what pull-request CI should not re-check every push (08 §8.3.2); the card scopes this package to exactly the dependency-audit and docs-links jobs, with the full nightly matrix arriving later.

**Independent Test**: The nightly workflow triggers on its schedule and on manual dispatch and runs the two jobs (card Deliverables: "nightly.yml (dependency audit + docs links now; more jobs added by later WPs)").

**Acceptance Scenarios**:

1. **Given** the merged nightly workflow, **When** the 02:17 UTC schedule fires, **Then** the dependency-audit and docs-links jobs run and report their results (docs links informational, 08 §8.3.2).
2. **Given** the nightly workflow, **When** the owner dispatches it manually, **Then** the same two jobs run.

---

### User Story 5 - CI is fast, and repository protection can be completed by the owner (Priority: P5)

Pull-request CI fits the ≤ 25-minute budget with warm caches (NFR-MNT-02) through per-branch concurrency and cache restore, and the WP-01 pull request body carries the owner checklist for the second stage of OPS-02 — the required status checks (`docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke`) — which the owner, and only the owner, enables after this package merges.

**Why this priority**: Speed keeps the gates bearable (NFR-MNT-02), and the OPS-02 checklist is the closing act the card explicitly assigns to the pull-request body; both matter only once the pipeline itself exists.

**Independent Test**: The measured warm-cache duration is recorded as the T-QA-002 baseline (card Acceptance), and the pull-request body contains the OPS-02 stage-two checklist (card Deliverables).

**Acceptance Scenarios**:

1. **Given** warm caches, **When** a pull-request CI run completes, **Then** its wall-clock duration is ≤ 25 minutes (NFR-MNT-02; T-QA-002 baseline recorded in the pull request).
2. **Given** the WP-01 pull request, **When** the owner reviews it, **Then** the body lists the OPS-02 stage-two settings to apply after merge — the eight required status checks of 08 §8.2 — for the owner to enable (agents never change repository settings, G-18).

---

### Edge Cases

- The four deliberately failing commits (formatting, lint, placeholder marker, missing sign-off) are demonstration artifacts produced on the work branch and evidenced in the pull request; they must never reach `main` (G-03, G-05).
- Steps of 08 §8.3.1 whose targets do not exist yet on the skeleton (bindings drift check, vcpkg binary cache, golden smoke set, UI lint/test/i18n/harness steps, executable-start check, real E2E) must not be faked or stubbed green — the only sanctioned no-op is the `e2e-smoke` job reporting success until WP-05, exactly as the card states; every other step arrives with its owning work package (G-02, G-08).
- A Dependabot dependency update is itself a pull request: it must pass the same CI (lockfiles revalidated with `--locked` / `--frozen-lockfile`, 05 §5.6 rule 2) before merging.
- A new advisory surfacing after the baseline makes the advisory checks fail; the resolution is updating the affected dependency (or, for the release gate, NFR-SEC-01 applies "at release"), never silencing the check (G-08).
- The 25-minute budget (NFR-MNT-02) is defined "with warm caches": a cold first run may exceed it; the recorded baseline states the cache conditions of the measurement.
- CI-required tooling (e.g., `cargo-deny`, `cargo-nextest`, coverage tooling) is installed by the workflows as environment tooling, not added to the workspace manifests — the version baseline (00 §0.8) and the dependency policy (G-11) stay untouched.

## Requirements *(mandatory)*

All requirements are scoped to the WP-01 card (docs/10-execution.md §10.7, WP-01). IDs below are feature-local (FR-xxx); they introduce nothing beyond the card and the cited sections.

### Functional Requirements

- **FR-001**: `.github/workflows/ci.yml` MUST trigger on pull requests to `main` and pushes to `main`, use a per-branch concurrency group that cancels in-progress runs, and keep the total warm-cache budget within the ≤ 25 minutes of NFR-MNT-02 (08 §8.3.1; docs/01 NFR-MNT-02).
- **FR-002**: The pull-request pipeline MUST create the eight jobs of 08 §8.3.1 — `docs`, `commits`, `dco`, `rust-lint`, `rust-test`, `ui`, `build`, `e2e-smoke` — because these are the required status checks of OPS-02 (08 §8.2); each job runs the steps whose targets exist on the current skeleton, and no job invents steps for targets that later work packages own (card Deliverables; 08 §8.3.1; G-02).
- **FR-003**: The `docs` job MUST run `python tools/docs/check_docs.py` (checks VER-01..VER-10) (08 §8.3.1; 09 §9.1 "every REQ has at least one test (VER-03)" relies on it).
- **FR-004**: The `commits` job MUST verify that every commit in the pull request follows Conventional Commits with the types and scopes of 10 §10.4.5, wiring the existing `tools/git/check_commits.py` into CI (card Deliverables "exists; wire it"; 08 §8.3.1, §8.4).
- **FR-005**: The `dco` job MUST verify that every commit carries a `Signed-off-by:` trailer matching the author email (08 §8.3.1).
- **FR-006**: The `rust-lint` job MUST run on Windows with the toolchain from `rust-toolchain.toml` and perform the skeleton-applicable steps of 08 §8.3.1: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo deny check`; `python tools/check_deps.py`; the unfinished-marker grep; the bindings-drift step is added when bindings exist (WP-02) (08 §8.3.1; 08 §8.4).
- **FR-007**: The `rust-test` job MUST run `cargo nextest run --workspace --locked` (with `ALVS_FORCE_WARP=1`) and upload a coverage report as an artifact — the measurement wiring for NFR-MNT-01's line-coverage targets; the vcpkg binary-cache steps are added when `vcpkg.json` exists (WP-06) and the golden smoke set when golden tests exist (08 §8.3.1; docs/01 NFR-MNT-01).
- **FR-008**: The `ui` job MUST install pnpm dependencies with `--frozen-lockfile` and run the UI verification the current skeleton supports (the TypeScript/Vite build that proves type-correctness today); the remaining 08 §8.3.1 steps (`lint`, `test -- --coverage`, `i18n:check`, `test:harness`) are added by the work packages that create their targets (WP-04/05) (08 §8.3.1; card; G-02).
- **FR-009**: The `build` job MUST build the application crate with `cargo build -p afterleads-video-spirits --locked` (debug profile); the executable-start check in `cli version` mode is added when the CLI entry point exists (08 §8.3.1).
- **FR-010**: The `e2e-smoke` job MUST exist as a no-op that reports success until WP-05 delivers the real smoke suite, and MUST be visibly marked as such in the workflow so it is replaced, not mistaken for coverage (card Deliverables; 08 §8.2 includes it among required checks).
- **FR-011**: `.github/workflows/nightly.yml` MUST run on the daily 02:17 UTC schedule and on manual dispatch with exactly the two jobs the card scopes today — dependency audit (`cargo deny check advisories`, `pnpm audit --prod`) and docs links (external link check, informational, weekly cadence); the full nightly matrix (full suite, full E2E, fuzz, benchmarks) is added by later work packages (08 §8.3.2; card).
- **FR-012**: `.github/dependabot.yml` MUST propose updates weekly for the cargo, npm, and actions ecosystems (05 §5.6 rule 5; card).
- **FR-013**: `deny.toml` MUST configure `cargo deny` to enforce 05 §5.6 exactly: the license allowlist and deny list as written there; advisories; sources limited to crates.io; bans reviewing duplicate major versions and denying HTTP-client crates everywhere except the consumers the canonical documents sanction — the application shell's embedded client (`reqwest`/`hyper` reachable only through `tauri`, per ADR-023 "the updater plugin uses its own client" and SEC-10's "no *other* crate" wording, encoded as cargo-deny wrapper allowances: `reqwest` wrapped by `tauri`, `hyper` wrapped by `reqwest`; owner decision, 2026-11-02) and, from WP-21, the model-download client (`ureq` 3 wrapped by `alvs-stt`) (card Deliverables; 05 §5.6 rule 3; 05 §5.7 SEC-10; ADR-023; `Cargo.lock` line 3070 — `tauri 2.12.5` depends on `reqwest`).
- **FR-014**: `tools/check_deps.py` MUST exist and enforce the dependency rules of 02 §2.4 (dependencies only from each component's CMP row, no cycles; exclusive crate usage — `ffmpeg-next` only in `alvs-media`, `wgpu` only in `alvs-render`, `cpal` only in `alvs-audio`, `rusqlite` only in `alvs-store`, `whisper-rs` only in `alvs-stt`, `tauri`/`webview2-com` only in CMP-12; UI importing Rust types only through the generated bindings; documented public APIs), and MUST pass on the current workspace (02 §2.4, which assigns these rules to this tool in CI as VER-09; card).
- **FR-015**: `tools/check_placeholders.py` MUST exist, detect the unfinished-work markers of 06 §6.13 (placeholder macros, marker comments, fake production paths, commented-out code), and be wired into the local gates and CI (08 §8.4; card).
- **FR-016**: `tools/check_npm_licenses.py` MUST exist and verify that every npm dependency's license is on the 05 §5.6 allowlist (05 §5.6 names it as the npm-side enforcer; card).
- **FR-017**: Every GitHub Actions reference in the workflows MUST be pinned to a full commit SHA, and workflow permissions MUST be read-only (05 §5.6 rule 5; 08 §8.2 Actions row; card).
- **FR-018**: The new local gates of 08 §8.4 — dependency policy (`cargo deny check` plus `python tools/check_deps.py`) and placeholders (`python tools/check_placeholders.py`) — MUST activate in `tools/verify.ps1`, which MUST exit 0 on the completed package; npm license enforcement runs in CI per 05 §5.6 (the 08 §8.4 table lists it only as a CI-side enforcer, not a `verify.ps1` gate) (08 §8.4; 05 §5.6; G-08).
- **FR-019**: The card's acceptance MUST be demonstrated with evidence in the pull request: CI green on the PR, and four deliberately failing commits — a formatting violation, a lint violation, an unfinished-marker, and a commit without sign-off — each failing exactly the responsible job (card Acceptance).
- **FR-020**: The baselines MUST be recorded in the pull request: T-SEC-001 (dependency advisories and license policy clean on the current lockfiles, covering NFR-SEC-01) and T-QA-002 (measured pull-request CI duration with warm caches, covering NFR-MNT-02) (card Acceptance; 09 §9.10 rows for T-SEC-001 and T-QA-002).
- **FR-021**: The WP-01 pull-request body MUST include the owner checklist for the second stage of OPS-02 — enabling the required status checks `docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke` after this package merges — because repository settings are applied only by the owner (card Deliverables; 08 §8.2; G-18).
- **FR-022**: Scope boundary — this work package MUST NOT add the release workflow (WP-28), the later nightly jobs (full suite, full E2E, fuzz, benchmarks), any change to repository settings or required checks, or any new workspace dependency (card "Not included"; 08 §8.2; G-02, G-11, G-18).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of the CI jobs are green on the WP-01 pull request (and on the push to `main` after merge).
- **SC-002**: 4 of 4 deliberately failing commits (format, lint, unfinished-marker, missing sign-off) fail exactly the responsible CI job, demonstrated with recorded evidence in the pull request.
- **SC-003**: The measured warm-cache pull-request CI duration is ≤ 25 minutes, recorded as the T-QA-002 baseline.
- **SC-004**: The dependency and license checks pass on the committed lockfiles — zero high/critical advisories and zero non-allowlisted licenses — recorded as the T-SEC-001 baseline.
- **SC-005**: 100% of workflow action references are pinned to full commit SHAs and the workflows request no write permissions.
- **SC-006**: `pwsh -File tools/verify.ps1` exits 0 locally with the new gates active.

## Assumptions

- The skeleton state is exactly what WP-00 merged (merge commit `7fc814f`): UI scripts limited to `dev`/`build`/`preview`/`tauri` (no lint/test/i18n/harness scripts yet), no `vcpkg.json`, no IPC bindings, no test/golden/fuzz/benchmark suites, and no *direct* HTTP-client dependency in any workspace manifest — the only HTTP-client crates in `Cargo.lock` (`reqwest`, `hyper`) arrive transitively through the sanctioned `tauri` app shell (FR-013; ADR-023). This defines "jobs … that apply to the current skeleton" (card): all eight job names exist, each with the subset of steps whose targets exist.
- CI-required tools (`cargo-deny`, `cargo-nextest`, coverage tooling) are installed by the workflows as environment tooling; the workspace manifests, the version baseline (00 §0.8), and the dependency policy (G-11) are unchanged by this package.
- NFR-MNT-01's coverage percentages (≥ 80% `alvs-core`/`alvs-model`/`alvs-formats`; ≥ 60% other crates; ≥ 70% UI state/logic) bind as code lands in the owning work packages; this package delivers only the measurement wiring and artifact upload the card assigns ("Implements: … (gates)").
- The owner applies the OPS-02 stage-two settings (required status checks) personally after WP-01 merges (08 §8.2 two-stage note; G-18); the checklist in the pull-request body is this package's deliverable, not the setting change.
- The T-QA-002 duration baseline is measured with warm caches, as NFR-MNT-02 defines; the baseline records the cache conditions.
- The `e2e-smoke` no-op reports success without running a suite until WP-05, exactly as the card sanctions; WP-05 replaces it with the real smoke suite.
