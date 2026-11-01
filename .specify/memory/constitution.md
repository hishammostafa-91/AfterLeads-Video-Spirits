# AfterLeads-Video-Spirits Constitution

This constitution governs every Spec Kit feature (`specs/0NN-*`) of AfterLeads-Video-Spirits. It condenses the canonical documentation in `docs/`; when a conflict appears, `docs/00-index.md` and the canonical topic document win, and the conflict must be reported to the owner.

## Core Principles

### I. Canonical Documentation First

Specifications, plans, and tasks derive only from `docs/00`–`docs/12`. Every requirement in a spec cites its REQ/NFR ID from `docs/01-product.md`; every design choice in a plan cites a section of `docs/02`–`docs/05` or an ADR. Specs MUST NOT introduce requirements, features, dependencies, or interfaces that the canonical documents do not contain. Missing or contradictory information stops the work until the owner decides.

### II. Scope Discipline

A feature implements exactly the requirements listed in its work-package card (`docs/10-execution.md` §10.7). Items tagged for release 1.1 or 1.2 are never built early. No speculative abstractions, no unrequested options.

### III. Test-First Evidence

Every requirement has tests with the IDs from `docs/09-quality.md`; tests are written with or before the code they verify. Tests are deterministic (seeded, offline, no wall-clock dependence). A feature is done only with recorded evidence: commands run, results, and test IDs.

### IV. Protected Main and Clean History

All work happens on a branch created from an up-to-date `main` and reaches `main` only through a pull request merged by the owner with a merge commit. No direct commits to `main`, no force pushes, no history rewriting, no hook bypassing.

### V. Quality Gates Are Non-Negotiable

Formatting, linting, type checks, unit/integration/golden/UI tests, localization checks, dependency/license checks, and documentation checks must pass locally (`tools/verify.ps1`) and in CI. Gates and tests are never weakened, skipped, or disabled to get green.

### VI. Global and Accessible by Default

The UI ships in English, Arabic, French, Spanish, German, and Brazilian Portuguese with correct right-to-left behavior. No hard-coded user-facing strings. Every feature is keyboard-operable, has accessible names, visible focus, and meets WCAG 2.2 AA contrast.

### VII. Private, Secure, Local-First

No telemetry and no cloud dependency. Network access only for update checks (opt-out) and user-initiated model downloads through the network guard. All inputs are untrusted: validate, bound, isolate risky parsing in worker processes, and fuzz parsers. Secrets never enter the repository.

### VIII. Simplicity and Performance Budgets

Prefer the simplest design that satisfies the requirement and the architecture in `docs/02`. Performance and resource budgets (NFR-PERF, NFR-RES) are requirements; regressions above 10% block merging.

### IX. Open-Source Integrity

All code and documentation are AGPL-3.0-or-later with SPDX headers. Dependencies and assets use only allowlisted licenses and are recorded in the notices. Nothing is copied from other products, and other video editing products are never referenced.

## Technology Constraints

Windows 11 x64 only for 1.x. Tauri 2 + WebView2; Rust 1.99.1 (edition 2024); wgpu 30 on Direct3D 12 with WARP fallback; FFmpeg 9.0.2 static via vcpkg (`x64-windows-static`); cpal (WASAPI); cosmic-text; whisper-rs; SQLite; React 19 + TypeScript 6.0 + Vite 8 + React Aria Components + dockview + Zustand + react-intl. Exact versions: `docs/00-index.md` §0.8 and the lockfiles. Changes require an ADR approved by the owner.

## Development Workflow

1. `/wp-start` creates the branch from `main` and prints the `/speckit.specify` line.
2. Specify → clarify (only from docs) → plan → **owner approves the plan** → tasks → analyze (no CRITICAL/HIGH findings) → implement in stages with `/wp-verify` → converge until converged → review → `/wp-finish` (push and open the pull request).
3. The owner reviews against the checklist in `docs/08-operations.md` (OPS-04) and merges with a merge commit.

## Global Rules (verbatim copy of docs/00-index.md §0.4)

<!-- G-RULES:BEGIN -->
- **G-01 Canonical docs.** The documents in `docs/` are the single source of truth. Implement only what they specify. If something is missing, ambiguous, or contradictory, stop and ask the owner; never guess.
- **G-02 Scope.** Build only the requirements assigned to the active work package. Never implement items tagged for a later release, and never add unrequested features.
- **G-03 Protected main.** Never commit, edit, or push directly on `main`. Every change starts on a new branch created from an up-to-date `main`.
- **G-04 Branch from main only.** Create branches only from `main`. A branch from another branch (stacked work) is allowed only when the owner approves it and the pull request states it.
- **G-05 Merge commits only.** Changes reach `main` only through a pull request merged with a merge commit (`--no-ff`). No squash, no rebase-merge, no fast-forward.
- **G-06 No history rewriting.** Never force-push, never rewrite pushed commits, never bypass git hooks (`--no-verify`), never change `core.hooksPath`.
- **G-07 One unit of work per branch.** One work package or fix per branch and per pull request. Commits are small, build cleanly, use Conventional Commits, and carry a DCO `Signed-off-by` line.
- **G-08 Quality gates.** Formatting, linting, type checks, tests, localization checks, and documentation checks MUST pass locally before pushing and in CI before merging. Never disable, skip, or weaken a test or gate to get green.
- **G-09 Evidence.** "Done" requires evidence: the commands run, their results, and the test IDs covered, recorded in the pull request.
- **G-10 No invention.** Never invent APIs, crate or package versions, file paths, command flags, or facts. Verify against installed sources, lockfiles, or official documentation before use.
- **G-11 Approved dependencies only.** Use only dependencies listed in the version baseline (0.8) or added through an ADR approved by the owner, with a license on the allowlist (05 §5.6).
- **G-12 License and originality.** All code is AGPL-3.0-or-later with SPDX headers. Never copy code, text, icons, or assets from other products. Third-party assets need a compatible license recorded in the notices manifest.
- **G-13 Localization by default.** No hard-coded user-facing strings. Every string has a key in all six locales (en, ar, fr, es, de, pt-BR). Layout uses logical (RTL-safe) properties.
- **G-14 Accessibility by default.** Every feature is keyboard-operable, has accessible names, and shows visible focus.
- **G-15 Privacy and security by default.** No telemetry. Network access only for update checks (opt-out) and user-initiated downloads. Validate all inputs. Never execute user-provided strings as commands. Never commit secrets.
- **G-16 Data safety.** Never lose user data: atomic writes, autosave, backups, forward-only migrations with a backup first.
- **G-17 Stop conditions.** Stop and ask the owner when: a requirement is unclear or conflicting; a new dependency or ADR seems necessary; a canonical doc must change; a gate cannot be made green without weakening it; an action is destructive or irreversible; a security or licensing question arises.
- **G-18 Owner authority.** Agents never merge pull requests, delete unmerged branches, change repository settings, publish releases, rotate keys, or edit LICENSE, TRADEMARKS.md, SECURITY.md, the constitution, or agent configuration unless the owner explicitly instructs it in the current session.
- **G-19 Docs move with code.** A change to behavior, configuration, or an interface updates the canonical document in the same pull request.
- **G-20 Exact identity.** Use the identity constants in 00 §0.5 exactly. Never reference other video editing products or their names in code, assets, UI, or docs.
<!-- G-RULES:END -->

## Governance

This constitution is amended only through a documentation pull request approved by the owner that updates `docs/00-index.md`, `AGENTS.md`, and this file together (the global-rules blocks must stay identical). Amendments bump the version below: patch for clarifications, minor for new or changed principles, major for removed principles.

**Version**: 1.0.0 | **Ratified**: 2026-11-01 | **Last Amended**: 2026-11-01
