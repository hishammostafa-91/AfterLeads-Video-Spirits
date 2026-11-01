# AGENTS.md — Operating manual for AI agents

You are building **AfterLeads-Video-Spirits**: a free, open-source (AGPL-3.0-or-later), professional video editor for Windows 11 x64. Stack: Tauri 2 shell + Rust engine (wgpu/Direct3D 12, FFmpeg 9, WASAPI) + React/TypeScript UI. Owner: AfterLeads <afterleadseg@gmail.com>.

`docs/` is the single source of truth. This file is a digest; when anything here seems to differ from the canonical document, the canonical document wins (docs/00-index.md §0.2–§0.3) and you report the difference.

## 1. Global rules (verbatim copy of docs/00-index.md §0.4)

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

## 2. Start of every session

1. Run `/status` (or `git status --short --branch`). If you are on `main`, do not edit anything: start a work branch first (§3).
2. Read `STATUS.md`, then the active work-package card in `docs/10-execution.md` §10.7, then **only** the sections listed in its "Read" line.
3. When resuming, read `specs/0NN-*/tasks.md` (checked boxes = done).
4. After any context compaction, re-read the card and `tasks.md` before continuing.

## 3. Git protocol (canonical: docs/10-execution.md §10.4)

| Step | Command (PowerShell 7) |
|---|---|
| Start work (branch from up-to-date `main`) | `pwsh -File tools/git/New-WorkBranch.ps1 -Type feat -Number 12 -Slug editing-engine` |
| Commit (Conventional Commits + DCO) | `git commit -s -m "feat(model): add trim EditOp"` |
| Bring in new `main` (never rebase pushed work) | `pwsh -File tools/git/Sync-Main.ps1` |
| Push your branch | `git push -u origin <branch>` |
| Open the pull request | `gh pr create --base main --fill` (then complete the template) |
| Parallel work package | `pwsh -File tools/git/New-Worktree.ps1 -Type feat -Number 17 -Slug titles-generators` |

- Branch names: `<type>/<NNN>-<slug>`; types `feat fix docs chore ci refactor test perf i18n build`; `NNN` = work-package number or issue number.
- Commit types: `feat fix docs style refactor perf test build ci chore revert i18n`; scopes listed in docs/10-execution.md §10.4.5.
- The owner merges with a merge commit. You never merge, never push to `main`, never force-push, never use `--no-verify`, never change `core.hooksPath`, never create or push tags, never run `git reset --hard` / `git clean -fdx` / `git branch -D` without explicit owner approval.

## 4. Work-package flow (canonical: docs/06-ai-automation.md §6.11)

`/wp-start WP-NN` → `/speckit.specify SPECIFY_FEATURE_DIRECTORY=specs/0NN-<slug> <brief>` → `/speckit.clarify` (only if needed; answers must come from docs) → `/speckit.plan <constraints>` → **stop: owner approves plan.md** → `/speckit.tasks` → `/speckit.analyze` (no CRITICAL/HIGH) → `/speckit.implement <stage>` with `/wp-verify` after each stage → `/speckit.converge` until "Converged" → `/review` → `/wp-finish`.

Skip Spec Kit only for docs-only changes, dependency bumps, and fixes under ~50 changed lines (use a `fix/` or `docs/` branch and a normal pull request).

## 5. Stop and ask the owner when

- a requirement, card, or doc is unclear, missing, or contradictory;
- an API, flag, version, or path cannot be verified (§8);
- a new dependency or architecture decision seems necessary;
- a canonical document would have to change;
- a gate cannot pass without weakening it;
- an action is destructive or irreversible;
- a security, privacy, or licensing question arises;
- the plan checkpoint is reached.

Ask one precise question, propose options with your recommendation, and wait.

## 6. Commands

| Purpose | Command |
|---|---|
| All local gates (skips gates whose targets do not exist yet) | `pwsh -File tools/verify.ps1` |
| Documentation checks | `python tools/docs/check_docs.py` |
| Workstation check | `pwsh -File tools/setup/Test-DevEnvironment.ps1` |
| Rust build / tests | `cargo build --workspace --locked` / `cargo nextest run --workspace --locked` |
| Rust format / lint | `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| App in development | `pnpm -C app tauri dev` |
| UI harness (mock backend) | `pnpm -C app harness` |
| UI checks | `pnpm -C app typecheck`, `pnpm -C app lint`, `pnpm -C app test`, `pnpm -C app i18n:check` |
| Test media | `pwsh -File tools/media/generate_test_media.ps1` |

Some commands exist only after the work package that creates them (see the cards).

## 7. Where things live (docs/02-architecture.md §2.13)

`crates/alvs-*` Rust crates (core, model, formats, media, render, audio, engine, store, jobs, stt, testkit) · `app/src-tauri` app backend · `app/src` UI · `assets/` fonts, LUTs, templates, help, model manifest · `tests/` end-to-end and fixtures · `tools/` scripts · `specs/` Spec Kit artifacts · `docs/` canonical documentation.

## 8. Anti-hallucination rules (docs/06-ai-automation.md §6.13)

1. Verify every API, flag, version, or path in this order: repository code → exact dependency source (`$env:USERPROFILE\.cargo\registry\src\…`, `app\node_modules\…`) or `cargo doc -p <crate>` → official docs for the locked version. Then compile and test.
2. Cite the doc section or ADR for every design choice in specs and plans; if none exists, stop and ask.
3. Write the smallest compiling slice first; test early; commit small.
4. No placeholders in merged code: no `todo!()`, `unimplemented!()`, `TODO`/`FIXME` comments, fake production paths, or commented-out code.
5. Never add behavior that is not in the docs; never change identity constants (docs/00-index.md §0.5) or catalog IDs.
6. Report honestly in the pull request and STATUS.md what is done, what is not, and what was not verified.

## 9. Coding rules (docs/02-architecture.md §2.14)

- Every source file starts with `SPDX-License-Identifier: AGPL-3.0-or-later` and `Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>` comments.
- Rust: errors map to E-codes (docs/12-catalogs.md §12.13); no `unwrap`/`expect` outside tests; `unsafe` only where docs/02 allows, with `// SAFETY:`.
- Time is integer flicks (705,600,000 per second); TS fields use `#[ts(type = "number")]`. Space is normalized (docs/03-data.md §3.3).
- UI: strict TypeScript, React Aria Components, no JSX text literals (react-intl IDs), CSS Modules with tokens and logical properties only, six locales (en, ar, fr, es, de, pt-BR) for every key, RTL rules in docs/07-experience.md §7.11.
- New dependencies only through an approved ADR and the license allowlist (docs/05-security.md §5.6).

## 10. Definition of Done (docs/10-execution.md §10.9)

Card tests pass locally and in CI · all gates green · `/speckit.converge` converged · docs updated with behavior · strings in six locales · STATUS.md updated · evidence in the pull request · owner merged with a merge commit.

## 11. Reading map (docs/00-index.md §0.12)

Engine → docs/02 §2.5–2.10, docs/03 §3.3–3.5, docs/04 §4.3–4.8 · UI → docs/07 + docs/04 §4.2–4.4 + docs/12 · Persistence → docs/03 · Security-sensitive → docs/05 · Tests → docs/09 · CI/release → docs/08.
