# Feature Specification: WP-00 — Workstation and repository bootstrap

**Feature Branch**: `chore/000-bootstrap`

**Created**: 2026-11-02

**Status**: Draft

**Input**: User description: "Implement WP-00 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: foundations for G-03..G-08 (no product REQ). Acceptance tests: none with T IDs — gates are Test-DevEnvironment.ps1 all green; cargo build --workspace --locked; pnpm -C app build; window opens with the correct title; python tools/docs/check_docs.py green; a test commit on main rejected by the hook (manual check recorded in the PR). Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents."

**Canonical basis**: WP-00 declares no product REQ/NFR IDs (card: "Implements: foundations for G-03..G-08 (no product REQ)"). Per docs/00 §0.3.3 and the constitution (I. Canonical Documentation First), every requirement below is derived from the WP-00 card (docs/10-execution.md §10.7, WP-00) and cites its canonical section. No requirement is added beyond those documents (G-01, G-02).

## User Scenarios & Testing *(mandatory)*

Actors: the project owner, human contributors, and AI agents — all working on Windows 11 x64 workstations (docs/00 §0.1). This work package builds no end-user feature; its "users" are the people and agents who build the product.

### User Story 1 - Verified workstation and protected history (Priority: P1)

A contributor (or agent) on a Windows 11 machine can verify that the workstation meets the project baseline and that the repository's git protections are active, before any product code is written. The workstation check reports every tool as green, the git hooks are installed and effective, and a direct commit attempt on `main` is rejected.

**Why this priority**: Nothing else can be built safely without a verified toolchain and the branch/commit protections that enforce G-03..G-08. This is the foundation the whole project stands on.

**Independent Test**: Run `pwsh -File tools/setup/Test-DevEnvironment.ps1` (all checks PASS) and attempt a test commit on `main` (rejected by the hook). Delivers a trustworthy development environment on its own.

**Acceptance Scenarios**:

1. **Given** a Windows 11 x64 machine prepared per docs/10 §10.2, **When** the contributor runs `Test-DevEnvironment.ps1`, **Then** every check reports PASS with no FAIL and no WARN.
2. **Given** the cloned repository with hooks installed (`tools/git/Install-Hooks.ps1`, docs/10 §10.4), **When** a commit is attempted directly on `main`, **Then** the commit is rejected by the hook (recorded as a manual check in the pull request).

---

### User Story 2 - Spec Kit workflow is usable (Priority: P2)

A contributor or agent can run the Spec Kit workflow on this repository: the Spec Kit infrastructure is initialized (`.specify/`, `.opencode/commands/speckit.*.md`), the constitution is preserved unchanged, and the generated files are committed.

**Why this priority**: Every subsequent work package follows the specify → plan → tasks → implement flow (docs/06 §6.11); without this initialization the per-package workflow cannot start.

**Independent Test**: The speckit slash commands exist and the constitution file is byte-identical to the baseline (`git diff .specify/memory/constitution.md` empty). Already satisfied on this branch; recorded here as a WP-00 deliverable.

**Acceptance Scenarios**:

1. **Given** the repository on branch `chore/000-bootstrap`, **When** `specify init --here --force --integration opencode --script ps --non-interactive` runs (docs/06 §6.11), **Then** `.specify/` and `.opencode/commands/speckit.*.md` are created and committed.
2. **Given** the initialized repository, **When** `git diff .specify/memory/constitution.md` runs, **Then** the output is empty (constitution preserved).

---

### User Story 3 - Cargo workspace skeleton builds (Priority: P3)

A contributor can clone the repository and build the complete Rust workspace skeleton: the root workspace manifest, the eleven internal crates (`alvs-core` … `alvs-testkit`), and the app crate location, all under the pinned Rust toolchain, with the license header on every source file.

**Why this priority**: The crate skeleton is the structural frame every later work package fills in (docs/02 §2.13); it must build cleanly before any crate gains code.

**Independent Test**: `cargo build --workspace --locked` exits 0 on a fresh clone (the pinned toolchain installs automatically from `rust-toolchain.toml`).

**Acceptance Scenarios**:

1. **Given** a fresh clone on a verified workstation, **When** `cargo build --workspace --locked` runs, **Then** it exits 0.
2. **Given** the created crates, **When** any source file is inspected, **Then** it carries the SPDX header and copyright line (docs/02 §2.14) and library-level documentation.

---

### User Story 4 - Application shell scaffold opens with the exact identity (Priority: P4)

A contributor can build and run the desktop application scaffold: a Tauri 2 (React + TypeScript) project in `app/`, adjusted to the project identity constants, with template demo code and assets removed, opening an empty window titled exactly "AfterLeads-Video-Spirits".

**Why this priority**: The shell proves the UI toolchain end to end; the real UI arrives in WP-04/05, so this slice only proves the scaffold is correct.

**Independent Test**: `pnpm -C app build` exits 0 and `pnpm -C app tauri dev` opens a window with the correct title.

**Acceptance Scenarios**:

1. **Given** the scaffolded `app/`, **When** `pnpm -C app build` runs, **Then** it exits 0.
2. **Given** the scaffolded `app/`, **When** `pnpm -C app tauri dev` runs, **Then** an empty window opens titled "AfterLeads-Video-Spirits" (docs/00 §0.5).

---

### User Story 5 - Local quality gates run green on the skeleton (Priority: P5)

A contributor can run every local quality gate on the skeleton in one step, and the documentation checks stay green, so later work packages inherit a green baseline.

**Why this priority**: Gates that do not run green on the skeleton cannot prove anything once code arrives (G-08); establishing the green baseline now makes every later regression visible.

**Independent Test**: `pwsh -File tools/verify.ps1` exits 0 (gates whose targets do not exist yet are skipped by design) and `python tools/docs/check_docs.py` exits 0.

**Acceptance Scenarios**:

1. **Given** the completed skeleton, **When** `pwsh -File tools/verify.ps1` runs, **Then** it exits 0.
2. **Given** the completed skeleton, **When** `python tools/docs/check_docs.py` runs, **Then** it exits 0.

---

### Edge Cases

- A contributor on a machine without the pinned Rust toolchain: `rust-toolchain.toml` must cause the exact toolchain (1.99.1, with `rustfmt` and `clippy` components) to install automatically rather than building with whatever is present.
- A fresh clone lacks hooks until `tools/git/Install-Hooks.ps1` runs; the workstation check reports hook status so the gap is visible (docs/06 CMD-06).
- The scaffold step downloads a template that contains demo code, assets, and placeholder identity; all must be removed or replaced with the project's identity constants (docs/00 §0.5; G-12 originality).
- Files must use UTF-8 with LF line endings and a final newline per `.editorconfig` (docs/02 §2.14), including the PowerShell scripts and manifests this package creates.

## Requirements *(mandatory)*

All requirements are scoped to the WP-00 card (docs/10 §10.7, WP-00). IDs below are feature-local (FR-xxx); they introduce nothing beyond the card and cited sections.

### Functional Requirements

- **FR-001**: The workstation MUST pass every check of `tools/setup/Test-DevEnvironment.ps1` with no FAIL and no WARN before the work package is considered done (card Acceptance; docs/10 §10.2).
- **FR-002**: The repository hooks (pre-commit, commit-msg, pre-push) MUST be active in the clone, and a test commit attempted directly on `main` MUST be rejected (card Acceptance; docs/10 §10.4; G-03, G-06).
- **FR-003**: Spec Kit MUST be initialized with the opencode integration and PowerShell scripts (`.specify/`, `.opencode/commands/speckit.*.md`), the existing constitution MUST be preserved unchanged, and the generated files MUST be committed (card Deliverables; docs/06 §6.11; the `specify extension add git` extension MUST NOT be used).
- **FR-004**: The repository root MUST contain a Cargo workspace whose members are `crates/*` and `app/src-tauri`, a `rust-toolchain.toml` pinning Rust 1.99.1 with the `rustfmt` and `clippy` components, and a `.cargo/config.toml` setting `rustflags = ["-C", "target-feature=+crt-static", "-C", "control-flow-guard"]` for `x86_64-pc-windows-msvc` (card Deliverables; docs/00 §0.8; docs/02 §2.13).
- **FR-005**: The eleven internal crates `alvs-core`, `alvs-model`, `alvs-formats`, `alvs-media`, `alvs-render`, `alvs-audio`, `alvs-engine`, `alvs-store`, `alvs-jobs`, `alvs-stt`, and `alvs-testkit` MUST exist as empty libraries, each with the SPDX/copyright header and library documentation in `lib.rs` (card Deliverables; docs/02 §2.13–2.14).
- **FR-006**: A Tauri 2 application scaffold (React + TypeScript template) MUST exist in `app/`, adjusted to the identity constants: product name "AfterLeads-Video-Spirits", identifier "com.afterleads.videospirits", window title "AfterLeads-Video-Spirits"; template demo code and assets MUST be removed; SPDX headers MUST be added; dependencies MUST be pinned to the docs/00 §0.8 baseline (card Deliverables and Steps; docs/00 §0.5).
- **FR-007**: A root `package.json` and `pnpm-workspace.yaml` MUST exist, with TypeScript pinned to 6.0.x (card Deliverables; docs/02 §2.13; docs/00 §0.8).
- **FR-008**: `pwsh -File tools/verify.ps1` MUST exit 0 on the completed skeleton (card Deliverables; docs/06 §6.10 CMD-02).
- **FR-009**: `python tools/docs/check_docs.py` MUST exit 0 (card Acceptance).
- **FR-010**: Scope boundary — this work package MUST NOT add CI workflows (WP-01), FFmpeg/media integration (WP-06), or real UI (WP-04/05) (card "Not included"; G-02).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `Test-DevEnvironment.ps1` reports PASS on 100% of its checks (zero FAIL, zero WARN).
- **SC-002**: `cargo build --workspace --locked` exits 0 on the completed skeleton.
- **SC-003**: `pnpm -C app build` exits 0, and `pnpm -C app tauri dev` opens a window whose title is exactly "AfterLeads-Video-Spirits".
- **SC-004**: `pwsh -File tools/verify.ps1` and `python tools/docs/check_docs.py` both exit 0.
- **SC-005**: The main-protection check is demonstrated once and recorded with its output in the pull request (a commit attempt on `main` rejected by the hook).
- **SC-006**: A fresh clone on a verified workstation reaches a successful `cargo build --workspace --locked` using only the repository's pinned manifests (no manual toolchain steps beyond the documented workstation setup).

## Assumptions

- The workstation preparation of docs/10 §10.2 is complete and green (verified 2026-11-02 on this machine: all checks PASS, including Node.js 26 LTS and pnpm 12.16.1); FR-001 therefore verifies an already-achieved state and guards regressions.
- The Spec Kit initialization deliverable (FR-003) is already committed on this branch (`311a773`, verified: constitution diff empty); the remaining work is the workspace, crates, scaffold, and gates.
- WP-00 has no product REQ/NFR IDs by design (card "Implements"); the citation duty is met through the card and the canonical sections listed in its Read line (00 all; 02 §2.13–2.14; 06 §6.8–6.11; 10 §10.2–10.4).
- `tools/verify.ps1` already exists in the baseline; this work package makes it pass on the skeleton, it does not rewrite it.
- STATUS.md row WP-00 is set to `done` only after the owner merges the pull request (card Deliverables; docs/10 §10.10 protocol).
