# Tasks: WP-00 — Workstation and repository bootstrap

**Input**: Design documents from `/specs/000-bootstrap/` (spec.md, plan.md, research.md, data-model.md, quickstart.md)

**Prerequisites**: plan.md (approved by the owner, OD-08, 2026-11-02), spec.md, research.md, data-model.md, quickstart.md

**Tests**: No test tasks are generated — the WP-00 card defines acceptance as gates and command outcomes, and carries no T-xx IDs (docs/10 §10.7). Each verification task names the command and its required outcome instead.

**Organization**: Tasks are grouped by user story (spec.md US1–US5) to enable independent implementation and verification of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Every task names concrete file paths

## Path Conventions

Monorepo per docs/02 §2.13: Rust workspace at the repository root (`crates/*`, `app/src-tauri`), pnpm workspace containing `app/`. All paths below are repository-relative.

## Phase 1: Setup (Shared Infrastructure — verified state)

**Purpose**: Record the already-established baseline as evidence (constitution III, G-09). These tasks verify and capture; they create no product code.

- [x] T001 Run `pwsh -File tools/setup/Test-DevEnvironment.ps1` and confirm every check prints PASS (zero FAIL, zero WARN); save the full table as pull-request evidence (tools/setup/Test-DevEnvironment.ps1)
- [x] T002 Confirm hooks are active in this clone: `git config --get core.hooksPath` prints `.githooks`, and `pwsh -File tools/git/Install-Hooks.ps1` reports already-installed (docs/10 §10.4.12) (tools/git/Install-Hooks.ps1)
- [x] T003 Capture (one-time) Spec Kit evidence for the pull-request evidence pack: `git diff .specify/memory/constitution.md` is empty, `.opencode/commands/speckit.*.md` files exist and are committed (commit `311a773`), and no `specify extension add git` was used (docs/06 §6.11) (.specify/memory/constitution.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Root manifests every later story builds on. No user-story label by design.

**⚠ CRITICAL**: US3 and US4 cannot begin until this phase is complete.

- [x] T004 [P] Create `rust-toolchain.toml` at the repository root: channel `1.99.1`, components `rustfmt` and `clippy` (card; docs/00 §0.8) (rust-toolchain.toml)
- [x] T005 [P] Create `.cargo/config.toml` with `[target.x86_64-pc-windows-msvc] rustflags = ["-C", "target-feature=+crt-static", "-C", "control-flow-guard"]` verbatim from the card (.cargo/config.toml)
- [x] T006 [P] Create root `package.json` (private workspace root, name `afterleads-video-spirits` — an implementation assumption matching the lowercase identity pattern; docs/00 §0.5 pins only the Cargo package name — and no scripts that duplicate app tooling) and `pnpm-workspace.yaml` with `packages: ["app"]` (docs/02 §2.13) (package.json, pnpm-workspace.yaml)
- [x] T007 Verify the pinned toolchain resolves: `cargo --version` reports 1.99.1 (rustup auto-installs from `rust-toolchain.toml` on first use); `rustup component list --toolchain 1.99.1` shows `rustfmt` and `clippy` installed (rust-toolchain.toml)

**Checkpoint**: Root manifests in place — user-story implementation can begin.

---

## Phase 3: User Story 1 — Verified workstation and protected history (Priority: P1) ⭐ MVP

**Goal**: A fully green workstation and demonstrable git protections (spec FR-001, FR-002; SC-001, SC-005).

**Independent Test**: quickstart.md Scenario A (all checks PASS) and Scenario E (commit on `main` rejected) — neither needs any code from US3/US4/US5.

### Implementation for User Story 1

- [x] T008 [US1] Demonstrate main protection exactly as quickstart.md Scenario E: `git switch main`, `git commit --allow-empty -m "test: hook rejection evidence"` (no `-s`), confirm `.githooks/pre-commit` rejects with `committing on 'main' is not allowed (G-03)`, then `git switch chore/000-bootstrap`; capture the rejection output for the pull request; if the commit succeeds, STOP and report a gate failure (G-03, G-08) (.githooks/pre-commit)

**Checkpoint**: Workstation (T001) and protection (T008) evidence exist — US1 is fully deliverable on its own.

---

## Phase 4: User Story 2 — Spec Kit workflow is usable (Priority: P2)

**Goal**: FR-003 verified as complete (this story was delivered on this branch before the checkpoint; nothing remains to build).

**Independent Test**: quickstart.md Scenario F — constitution diff empty; speckit commands present.

### Implementation for User Story 2

- [x] T009 [US2] Final US2 sign-off — re-run the FR-003 checks captured in T003 (constitution diff empty, speckit commands and `.specify/scripts/powershell/` present, commit `311a773`) to confirm nothing regressed during later phases; reference T003's captured evidence instead of duplicating it (specs/000-bootstrap/spec.md §FR-003)

**Checkpoint**: US2 confirmed done; no new artifacts.

---

## Phase 5: User Story 3 — Cargo workspace skeleton builds (Priority: P3)

**Goal**: FR-004 and FR-005 — root workspace manifest and the eleven empty crates; `cargo build --workspace --locked` exits 0 (SC-002).

**Independent Test**: `cargo build --workspace --locked` exits 0 after this phase (with `app/src-tauri` still absent, the workspace manifest lists `crates/*` only; US4 adds the app member — plan Structure Decision).

### Implementation for User Story 3

- [x] T010 [US3] Create root `Cargo.toml`: workspace manifest with `members = ["crates/*"]` (the `app/src-tauri` member is added in T026), `resolver = "3"` (the resolver that edition-2024 member crates imply; without an explicit resolver cargo warns "virtual workspace defaulting to resolver 1" on every build), SPDX header comment, no `[package]` (Cargo.toml)
- [x] T011 [P] [US3] Create `crates/alvs-core`: `Cargo.toml` (SPDX header comment; `name = "alvs-core"`, `edition = "2024"`, `license = "AGPL-3.0-or-later"`) and `src/lib.rs` with the SPDX header and a one-line crate doc comment (`//!`) (crates/alvs-core/src/lib.rs)
- [x] T012 [P] [US3] Create `crates/alvs-model` identically (name `alvs-model`) (crates/alvs-model/src/lib.rs)
- [x] T013 [P] [US3] Create `crates/alvs-formats` identically (name `alvs-formats`) (crates/alvs-formats/src/lib.rs)
- [x] T014 [P] [US3] Create `crates/alvs-media` identically (name `alvs-media`) (crates/alvs-media/src/lib.rs)
- [x] T015 [P] [US3] Create `crates/alvs-render` identically (name `alvs-render`) (crates/alvs-render/src/lib.rs)
- [x] T016 [P] [US3] Create `crates/alvs-audio` identically (name `alvs-audio`) (crates/alvs-audio/src/lib.rs)
- [x] T017 [P] [US3] Create `crates/alvs-engine` identically (name `alvs-engine`) (crates/alvs-engine/src/lib.rs)
- [x] T018 [P] [US3] Create `crates/alvs-store` identically (name `alvs-store`) (crates/alvs-store/src/lib.rs)
- [x] T019 [P] [US3] Create `crates/alvs-jobs` identically (name `alvs-jobs`) (crates/alvs-jobs/src/lib.rs)
- [x] T020 [P] [US3] Create `crates/alvs-stt` identically (name `alvs-stt`) (crates/alvs-stt/src/lib.rs)
- [x] T021 [P] [US3] Create `crates/alvs-testkit` identically (name `alvs-testkit`) (crates/alvs-testkit/src/lib.rs)
- [x] T022 [US3] Run `cargo build --workspace --locked` → exit 0; commit the generated `Cargo.lock` at the repository root (docs/00 §0.8; card acceptance `--locked` requires it) (Cargo.lock)
- [x] T023 [US3] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked` → all exit 0 on the empty crates (tools/verify.ps1 activates these gates the moment Cargo.toml exists — plan research D7) (Cargo.toml)

**Checkpoint**: Workspace + eleven crates build, lint, and test green under the pinned toolchain.

---

## Phase 6: User Story 4 — Application scaffold opens with the exact identity (Priority: P4)

**Goal**: FR-006, FR-007 — Tauri 2 React+TypeScript scaffold in `app/` carrying the docs/00 §0.5 identity; `pnpm -C app build` exits 0; dev window opens with the exact title (SC-003).

**Independent Test**: quickstart.md Scenario C.

*Depends on*: Phase 2 (pnpm workspace) and T010 (root `Cargo.toml` to add the member).

### Implementation for User Story 4

- [ ] T024 [US4] Scaffold with `pnpm create tauri-app@latest` into `app/` (React, TypeScript, pnpm per card Steps 4); adjust the package names: npm package `afterleads-video-spirits` (same implementation assumption as T006), Cargo package `afterleads-video-spirits` in `app/src-tauri/Cargo.toml` (docs/00 §0.5 "Cargo package of the app") (app/package.json)
- [ ] T025 [US4] Set identity in `app/src-tauri/tauri.conf.json`: `productName` `AfterLeads-Video-Spirits`, `identifier` `com.afterleads.videospirits`, main window `title` `AfterLeads-Video-Spirits` — exact constants, never shortened (docs/00 §0.5; G-20) (app/src-tauri/tauri.conf.json)
- [ ] T026 [US4] Add `"app/src-tauri"` to the root `Cargo.toml` workspace members so the final manifest reads `members = ["crates/*", "app/src-tauri"]` (card) (Cargo.toml)
- [ ] T027 [US4] Remove template demo code and demo assets from `app/src/` and `app/public/`; keep only `app/index.html`, `app/src/main.tsx`, one empty root component (`App` rendering an empty container), and base styling — no demo components, logos, icons, or sample assets remain (card Steps 4; G-12) (app/src/App.tsx)
- [ ] T028 [US4] Add the SPDX header pair (`SPDX-License-Identifier: AGPL-3.0-or-later`, `Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>`) as comments to every scaffold source file: `.rs`, `.ts`, `.tsx`, `.css` (docs/02 §2.14) (app/src/main.tsx)
- [ ] T029 [US4] Pin dependencies to docs/00 §0.8: TypeScript `6.0.x`, React `19.3.x`, Vite `8.3.x`, `@vitejs/plugin-react` `6.x`, `@tauri-apps/api` and `@tauri-apps/cli` `2.12.4`, `tauri` crate `2.12.5`; run `pnpm install` at the repository root and commit the generated root `pnpm-lock.yaml`, then run `cargo build --workspace --locked` and commit the updated root `Cargo.lock` (the workspace gained the app member and Tauri dependencies in T026–T029; the `--locked` gates in T032 fail on a stale lockfile) (app/package.json, pnpm-lock.yaml, Cargo.lock)
- [ ] T030 [US4] Run `pnpm -C app build` → exit 0 (card acceptance) (app/package.json)
- [ ] T031 [US4] Run `pnpm -C app tauri dev` → an empty window opens titled exactly `AfterLeads-Video-Spirits`; capture a screenshot for the pull request (SC-003; docs/00 §0.5) (app/src-tauri/tauri.conf.json)

**Checkpoint**: The app scaffold builds and opens with the exact identity; workspace manifest matches the card.

---

## Phase 7: User Story 5 — Local quality gates run green on the skeleton (Priority: P5)

**Goal**: FR-008, FR-009 — `tools/verify.ps1` exits 0 with only its by-design SKIPPEDs; `check_docs.py` exits 0 (SC-004).

**Independent Test**: quickstart.md Scenario D.

*Depends on*: US3 and US4 complete (gates activate on their artifacts).

### Implementation for User Story 5

- [ ] T032 [US5] Run `pwsh -File tools/verify.ps1` → exit 0; confirm the table shows `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test` = PASS and every other gate SKIPPED with the script's own reasons; never edit the script (G-08; plan research D7) (tools/verify.ps1)
- [ ] T033 [US5] Run `python tools/docs/check_docs.py` → exit 0 (card acceptance) (tools/docs/check_docs.py)

**Checkpoint**: The skeleton is gate-green; all card acceptance gates demonstrated.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and the evidence pack.

- [ ] T034 Run the full quickstart.md validation (Scenarios A–F) against the finished skeleton; fix anything it catches at the source (specs/000-bootstrap/quickstart.md)
- [ ] T035 Update the STATUS.md row for WP-00: state stays `in-progress` until merge; set branch `chore/000-bootstrap`, today's date, next step `finish` (docs/10 §10.10; STATUS.md)
- [ ] T036 Assemble the pull-request evidence per G-09: commands and outputs for T001, T008, T022, T023, T030, T031, T032, T033, and the window screenshot; reference the WP-00 card (specs/000-bootstrap/)
- [ ] T037 Confirm no canonical-doc update is required (G-19): the skeleton implements what docs/02 §2.13–2.14 already specify; if any discrepancy surfaced, stop and report instead of editing docs unilaterally (docs/02-architecture.md)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — verification of established state.
- **Foundational (Phase 2)**: Depends on Phase 1 — **blocks US3 and US4**.
- **US1 (Phase 3)**: Depends on Phase 1 only — independent of all code stories (MVP).
- **US2 (Phase 4)**: Depends on Phase 1 only — verification of completed work.
- **US3 (Phase 5)**: Depends on Phase 2 (T004, T005, T007).
- **US4 (Phase 6)**: Depends on Phase 2 (T006) **and US3's T010** (root `Cargo.toml` receives the `app/src-tauri` member in T026).
- **US5 (Phase 7)**: Depends on US3 **and** US4 (gates activate on their artifacts).
- **Polish (Phase 8)**: Depends on all stories.

### Within Each Story

- Root manifests before crates (US3), scaffold before identity/pins (US4), everything before gates (US5).
- Commit after each task or logical group (docs/10 §10.4.4): `build` type for manifests and crates, `chore`/`build` for scaffold steps, with DCO sign-off.

### Parallel Opportunities

- T004, T005, T006 — different files (Phase 2).
- T011–T021 — the eleven crates are independent directories (Phase 5).

---

## Parallel Example: User Story 3

```text
Task: "Create crates/alvs-core (Cargo.toml + src/lib.rs)"        # T011
Task: "Create crates/alvs-model (Cargo.toml + src/lib.rs)"       # T012
Task: "Create crates/alvs-formats (Cargo.toml + src/lib.rs)"     # T013
# ... all eleven crates in parallel, then T022/T023 sequentially
```

---

## Implementation Strategy

### MVP First (Phase 1 + US1)

1. Complete Phase 1 (evidence) and Phase 3/US1 (protection demo).
2. **STOP and VALIDATE**: workstation green + main protected — the project's safety floor exists.

### Incremental Delivery

1. Phase 2 → foundation manifests.
2. US3 → workspace builds (SC-002). 3. US4 → scaffold opens with identity (SC-003). 4. US5 → gates green (SC-004). 5. Phase 8 → evidence pack and `/wp-finish`.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks.
- [Story] labels map to spec.md user stories for traceability.
- No test-writing tasks: the card's acceptance is gate-based (no T-xx IDs); verification tasks name commands and required outcomes.
- Every commit: Conventional Commits type+scope from docs/10 §10.4.5, DCO sign-off, tree builds at every commit (G-07).
- Never weaken or edit a gate, hook, or check script (G-06, G-08).
- Stop and ask (G-17) on any discrepancy with the canonical docs; do not improvise.
