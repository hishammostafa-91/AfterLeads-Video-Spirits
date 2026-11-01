# 00 — Canonical Master Index

| Field | Value |
|---|---|
| Document set | AfterLeads-Video-Spirits implementation documentation |
| Set version | 1.0.0 |
| Baseline date | 2026-11-01 |
| Status | Approved baseline for implementation (v1.0 scope) |
| Owner | AfterLeads (afterleadseg@gmail.com) |
| License of this documentation | AGPL-3.0-or-later (same as the code, see `/LICENSE`) |

This file is the entry point. It defines what the product is, how the documentation is organized, which document is the source of truth for each topic, the global rules every contributor and every AI agent MUST follow, identity constants, ID conventions, and the verified technology baseline.

---

## 0.1 System overview

AfterLeads-Video-Spirits is a free, open-source, professional non-linear video editor for **Windows 11 (x64)**. It is a local desktop application: no account, no cloud service, no telemetry. Media never leaves the computer unless the user exports it.

Core capabilities (v1.0): projects with multiple sequences, a multitrack timeline with professional trimming tools, GPU-accelerated effects with keyframes, transitions, titles, masks and keying, color correction with scopes, an audio mixer with loudness tools, captions with RTL support, local speech-to-text for automatic captions, background proxies, and export through a render queue with hardware encoders.

```
┌──────────────────────────────── AfterLeads-Video-Spirits.exe ───────────────────────────────┐
│  UI (WebView2):  React + TypeScript  — panels, timeline canvas, inspector, dialogs          │
│        ▲  typed IPC (Tauri commands/events)      ▲ preview frames (WebView2 SharedBuffer)    │
│        ▼                                          │                                           │
│  App core (Rust): project model + undo, commands, job orchestration, settings, cache         │
│  Engine  (Rust): media I/O (FFmpeg), GPU render (wgpu/D3D12), audio (WASAPI), playback       │
└───────────────┬───────────────────────────────────────────────────────────────────────────────┘
                │ spawns the same executable in worker mode (JSON Lines over stdio)
                ▼
     Worker processes: export, proxies, thumbnails/waveforms, transcription, analysis
External boundaries: local file system, GPU driver, audio devices, GitHub Releases (update
check, opt-out), model download host (explicit user action only).
```

The full architecture is in [02-architecture.md](02-architecture.md).

---

## 0.2 Document hierarchy and reading order

| Doc | Title | Canonical for (single source of truth) |
|---|---|---|
| [00-index.md](00-index.md) | Canonical Master Index | Global rules (G-xx), identity constants, ID conventions, version baseline, owner decisions (OD-xx), glossary |
| [01-product.md](01-product.md) | Product | Vision, users, use cases, scope and releases, functional requirements (REQ-xx), non-functional requirements (NFR-xx) |
| [02-architecture.md](02-architecture.md) | Architecture | Architecture decisions (ADR-xx), components/crates (CMP-xx), process model, render/audio/playback design, threading, error model |
| [03-data.md](03-data.md) | Data | Project file format, data model and invariants, settings keys, cache database, file locations, lifecycle (autosave, backups, migrations) |
| [04-interfaces.md](04-interfaces.md) | Interfaces | IPC commands/events, EditOp contract, preview frame transport, worker protocol, CLI, effect manifest contract, error envelope |
| [05-security.md](05-security.md) | Security | Threat model (THR-xx), security controls (SEC-xx), privacy, supply chain, update trust |
| [06-ai-automation.md](06-ai-automation.md) | AI & Automation | Product AI (speech-to-text, roadmap AI), agentic development system (OpenCode v2, Spec Kit, agents, commands, anti-hallucination protocol) |
| [07-experience.md](07-experience.md) | Experience | Design system, layout, panels (PNL-xx), dialogs (DLG-xx), workspaces (WS-xx), interaction rules, RTL and accessibility rules |
| [08-operations.md](08-operations.md) | Operations | CI/CD, repository settings, packaging, updates, release, observability, diagnostics, support and runbooks (OPS-xx) |
| [09-quality.md](09-quality.md) | Quality | Test strategy, test catalog (T-xx), reference machines, acceptance gates (GATE-xx), performance budgets verification |
| [10-execution.md](10-execution.md) | Execution | Workstation setup, git protocol, agent workflow, milestones (M0–M9), work packages (WP-00–WP-29), risks (R-xx) |
| [11-verification.md](11-verification.md) | Cross-document verification | Consistency checks (VER-xx), contradiction register, traceability rules, verification record |
| [12-catalogs.md](12-catalogs.md) | Catalogs | All enumerations: effects, transitions, blend modes, easings, presets, commands/menus/shortcuts, error codes, locales, models, fonts, templates |

Root files with operational authority (derived from the docs above, kept in sync by VER checks):

| File | Purpose |
|---|---|
| `/AGENTS.md` | Operating manual loaded automatically by OpenCode. Restates G-rules verbatim. |
| `/.specify/memory/constitution.md` | Spec Kit constitution. Restates G-rules as principles. |
| `/STATUS.md` | Progress ledger of work packages (state, branch, PR). |
| `/CONTRIBUTING.md`, `/SECURITY.md`, `/TRADEMARKS.md`, `/CODE_OF_CONDUCT.md`, `/LICENSE` | Project governance. |

**Reading order for a new agent session:** `AGENTS.md` (automatic) → `STATUS.md` → the work-package card in [10-execution.md](10-execution.md#107-work-packages) → the sections listed in that card's "Required reading". Do not read the whole set at once; the cards list exactly what is needed.

---

## 0.3 Source-of-truth rules

1. Every fact has exactly one canonical home (the table in 0.2). Other documents reference it by ID or link and MUST NOT restate it with different values.
2. Precedence when two sources disagree: `00-index` global rules → the canonical document for that topic → `12-catalogs` → `specs/` (Spec Kit feature artifacts) → code comments. A conflict is a defect: stop work and report it (G-17).
3. `specs/NNN-*` artifacts generated by Spec Kit are **derived** documents. They MUST cite REQ/NFR IDs and MUST NOT add requirements that are not in [01-product.md](01-product.md).
4. Normative keywords MUST, MUST NOT, SHOULD, SHOULD NOT, MAY follow RFC 2119.
5. Numbers (counts, limits, defaults) appear in one canonical place. If a number must be repeated for readability, it is followed by its source ID.

---

## 0.4 Global rules (G-xx)

These rules apply to the owner, human contributors, the main OpenCode agent, every subagent, and any future automation. They are restated verbatim in `/AGENTS.md` and `/.specify/memory/constitution.md`; VER-05 checks that the three copies match.

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

---

## 0.5 Identity and naming constants

| Constant | Value | Used in |
|---|---|---|
| Product name | `AfterLeads-Video-Spirits` | UI title, installer, About, docs. Always written exactly like this. |
| Author / owner | `AfterLeads` | Copyright, installer publisher, package metadata |
| Contact email | `afterleadseg@gmail.com` | Package metadata, SECURITY.md, CODE_OF_CONDUCT.md |
| Copyright line | `Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>` | LICENSE notice, About, source headers (year of first publication; do not update yearly) |
| SPDX identifier | `AGPL-3.0-or-later` | Every source file header |
| Repository | `https://github.com/hishammostafa-91/AfterLeads-Video-Spirits` | Update endpoint, About links, issue links. If the repository moves, change it only via the constant files listed in 08 §8.6. |
| Bundle identifier | `com.afterleads.videospirits` | Tauri `identifier`; determines app data folders |
| Executable | `AfterLeads-Video-Spirits.exe` | Installer output (verified in WP-28) |
| Cargo package of the app | `afterleads-video-spirits` | `app/src-tauri/Cargo.toml` |
| Internal crate prefix | `alvs-` (Rust crates), `alvs_` (Rust modules) | `crates/alvs-*` |
| Project file extension | `.alvs` (JSON) | File association, open/save dialogs |
| Project archive folder suffix | `.alvs-collected` | "Collect files" output folder |
| Settings namespace | `alvs.` | Setting keys (03 §3.6) |
| Effect ID namespaces | `fx.` (video), `afx.` (audio), `tr.` (transitions), `gen.` (generators) | Catalogs (12) |
| Error code format | `E` + 4 digits (for example `E2003`) | 04 §4.9, 12 §12.13 |
| Default project folder | `%USERPROFILE%\Videos\AfterLeads-Video-Spirits` | New Project dialog default |
| Short code in commit scopes | area names from 10 §10.4.5 | Conventional Commits |

The product name is never shortened, translated, or written without its hyphens and its `AfterLeads-` prefix; VER-07 rejects such variants.

---

## 0.6 ID conventions

IDs are stable. Never renumber; retire an ID by marking it "Retired" and never reuse it.

| Prefix | Meaning | Defined in |
|---|---|---|
| `G-NN` | Global rule | 00 §0.4 |
| `OD-NN` | Owner decision | 00 §0.10 |
| `UC-NN` | Use case | 01 §1.4 |
| `REQ-<AREA>-NNN` | Functional requirement | 01 §1.6 |
| `NFR-<AREA>-NN` | Non-functional requirement | 01 §1.7 |
| `ADR-NNN` | Architecture decision record | 02 §2.2 |
| `CMP-NN` | Component (crate / app module) | 02 §2.4 |
| `THR-NN` / `SEC-NN` | Threat / security control | 05 |
| `AGT-NN` / `CMD-NN` | Development agent / OpenCode command | 06 §6.9–6.10 |
| `PNL-NN` / `DLG-NN` / `WS-NN` | Panel / dialog / workspace | 07 |
| `OPS-NN` | Operations procedure or pipeline | 08 |
| `T-<AREA>-NNN` / `GATE-NN` | Test / release gate | 09 |
| `M<N>` / `WP-NN` / `R-NN` | Milestone / work package / risk | 10 |
| `VER-NN` | Verification check | 11 |
| `VFX-NN`, `AFX-NN`, `TRS-NN`, `BLM-NN`, `EAS-NN`, `MSK-NN`, `TAN-NN`, `TPL-NN`, `GEN-NN`, `XP-NN`, `SQP-NN`, `FPS-NN`, `KB-NNN`, `LOC-NN`, `MDL-NN`, `FNT-NN`, `LUT-NN`, `CLR-NN` | Catalog entries | 12 |
| `E<NNNN>` | Error code | 12 §12.13 |

Requirement areas: `APP` shell, `PRJ` projects, `MED` media, `PLY` playback, `EDT` timeline editing, `FX` effects/keyframes/masks, `TRN` transitions, `TTL` titles/generators, `AUD` audio, `COL` color/scopes, `CAP` captions, `STT` speech-to-text, `EXP` export, `JOB` background jobs, `SET` settings, `I18N` localization, `A11Y` accessibility, `HLP` help/onboarding, `UPD` updates, `REL` reliability/diagnostics, `INT` interoperability, `LIC` licensing UI.

Definition syntax (machine-checked by VER-01): an ID is **defined** when it is the first cell of a table row (`| REQ-PLY-001 | ...`) or the start of a heading (`### WP-07 — ...`) in its canonical document. Every other occurrence is a **reference** and MUST resolve to a definition.

---

## 0.7 Release labels and priorities

| Label | Meaning |
|---|---|
| `1.0` | Required for the first public release. |
| `1.1`, `1.2` | Roadmap. Specified at requirement level so the architecture stays ready; not built before 1.0 ships. |
| `P0` | Release-blocking for its release. |
| `P1` | Required for its release unless the owner signs a waiver (09 §9.11). |
| `P2` | Roadmap item (always paired with release `1.1` or `1.2`). |

---

## 0.8 Technology baseline (verified 2026-11-01)

Versions below were verified against the official registries on the baseline date. Exact versions are locked by `Cargo.lock`, `pnpm-lock.yaml`, `rust-toolchain.toml`, and the vcpkg `builtin-baseline` at the work package that introduces them. Upgrades happen only on a `chore/deps-*` branch with passing gates (G-11). Rationale for each choice is in 02 §2.2.

| Area | Component | Baseline | Notes |
|---|---|---|---|
| OS | Windows 11 x64 | 22H2 or newer | Development and target platform |
| Toolchain | Rust | 1.99.1 (edition 2024) | Pinned in `rust-toolchain.toml` |
| Toolchain | Visual Studio 2022 Build Tools | MSVC v143 + Windows 11 SDK | "Desktop development with C++" workload |
| Toolchain | LLVM (libclang) | latest stable from winget | Required by FFmpeg bindings generation |
| Toolchain | Node.js | 26 LTS | |
| Toolchain | pnpm | 12.x | |
| Toolchain | Python + uv | Python 3.12, uv latest | Spec Kit and repository tools |
| Toolchain | vcpkg | baseline commit pinned at WP-06 | Builds FFmpeg and its codecs |
| Shell | Tauri | 2.12.x (crate 2.12.5; `@tauri-apps/api`/`cli` 2.12.4) | Desktop shell |
| Shell | tauri-plugin-updater / -dialog / -single-instance | 2.14.x / 2.8.x / 2.5.x | |
| Shell | webview2-com | 0.39.x | Must match the version used by Tauri's runtime |
| Engine | wgpu | 30.x | Direct3D 12 backend, WARP fallback |
| Engine | FFmpeg | 9.0.2 via vcpkg port `ffmpeg` 9.0.2 | GPL-enabled build, static, see 02 ADR-004 |
| Engine | ffmpeg-next | 9.0.x | Rust bindings |
| Engine | cpal | 0.18.x | WASAPI audio I/O |
| Engine | rubato / ebur128 / nnnoiseless | 5.x / 0.1.x / 0.5.x | Resampling / loudness / voice denoise |
| Engine | cosmic-text | 0.19.x | Text shaping (Arabic, bidi) and layout |
| Engine | whisper-rs | 0.16.x | Speech-to-text (bundles whisper.cpp) |
| Data | rusqlite (feature `bundled`) | 0.41.x | Cache database |
| Data | serde / serde_json / schemars / ts-rs / uuid | 1.x / 1.x / 1.x / 12.x / 1.x | Serialization, schema, TS types, UUID v7 |
| Core | thiserror / tracing / tracing-subscriber / tracing-appender / tokio | 2.x / 0.1.x / 0.3.x / 0.2.x / 1.x | Errors, structured logs, async runtime |
| Core | crossbeam-channel / arc-swap / parking_lot / rtrb | 0.5.x / 1.x / 0.13.x / 0.4.x | Engine threading (02 §2.10) |
| Core | windows / bytemuck / half / clap | 0.62.x / 1.x / 2.x / 4.x | Win32 calls, GPU buffer packing, f16, CLI parsing |
| Core | ureq (features `platform-verifier`, `win-system-proxy`) / sha2 / zip / encoding_rs | 3.x / 0.11.x / 9.x / 0.8.x | Model downloads, hashing, diagnostics bundles, caption encodings |
| Core | crash-handler / minidumper | 0.8.x / 0.11.x | Local crash capture (ADR-022) |
| Tests | insta / proptest / criterion | 1.x / 1.x / 0.8.x | Snapshots, property tests, benchmarks |
| Dev tools | cargo-nextest / cargo-deny / cargo-llvm-cov / cargo-about / cargo-cyclonedx / git-cliff / tauri-driver | 0.9.150 / 0.20.2 / 0.9.1 / 0.9.3 / 0.5.9 / 2.14.2 / 2.1.0 | Installed by `tools/setup/Install-DevTools.ps1` |
| Dev tools | @cyclonedx/cyclonedx-npm | 6.x | npm SBOM (08 §8.3.3) |
| UI | TypeScript | **6.0.x** | 7.x is not supported by `typescript-eslint` (peer `<6.1.0`) |
| UI | React / Vite / @vitejs/plugin-react | 19.3.x / 8.3.x / 6.x | |
| UI | react-aria-components | 1.22.x | Accessible, RTL-aware primitives |
| UI | dockview | 8.8.x | Docking layout |
| UI | zustand | 5.x | UI state |
| UI | react-intl / @formatjs/cli | 12.x / 6.x | ICU MessageFormat localization |
| UI | lucide-react | 1.x | Icons (ISC) |
| UI | react-markdown | 10.x | In-app help viewer |
| UI tests | Vitest / @testing-library/react / jsdom | 5.x / 16.x / 30.x | |
| UI tests | Playwright / @axe-core/playwright | 1.64.x / 4.x | UI harness, visual and accessibility tests |
| E2E | WebdriverIO + tauri-driver | 10.x + latest | Windows end-to-end tests |
| Lint | ESLint | **9.39.x** | 10.x is not supported by `eslint-plugin-jsx-a11y` |
| Lint | typescript-eslint / jsx-a11y / react-hooks / i18next plugin / formatjs plugin | 8.x / 6.x / 7.x / 6.x / 8.x | |
| Lint | Prettier / Stylelint / stylelint-use-logical / stylelint-config-standard | 3.x / 17.x / 2.x / 40.x | |
| Agent | OpenCode | 2.2.x | `npm install -g @opencode/cli` (06 §6.6) |
| Agent | Spec Kit (specify-cli) | 1.2.5 | Installed from the tagged GitHub release (06 §6.11) |

---

## 0.9 Glossary

| Term | Definition |
|---|---|
| Project | A `.alvs` document: media references, bins, sequences, assets, and project settings. |
| Sequence | An editable timeline with its own format (resolution, frame rate, audio rate) and tracks. |
| Track | A video, audio, or caption lane inside a sequence. Video tracks composite bottom to top. |
| Clip | A timeline item referencing a media item, a title, a generator, a nested sequence, or an adjustment layer. |
| Media item | A file registered in the project (video, audio, image, image sequence). |
| Flick | Time unit: 1/705,600,000 second. All persisted times are integer flicks (03 §3.3). |
| Edit point | The boundary between two adjacent clips on a track. |
| Handles | Source media available before a clip's in point or after its out point. |
| Effect | A parameterized GPU (video) or DSP (audio) process applied to a clip, track, or master. |
| Intrinsic properties | Properties every visual clip has without adding an effect: transform, crop, opacity, blend mode. |
| Keyframe | A time-stamped parameter value with an easing to the next keyframe. |
| Proxy | A smaller, edit-friendly copy of a media file used for preview only. |
| Worker | The application executable started in worker mode to run one background job in isolation. |
| Preview transport | The mechanism that delivers rendered preview frames from the engine to the UI (04 §4.5). |
| WARP | Microsoft's software Direct3D 12 rasterizer, used as GPU fallback and for deterministic tests. |
| Work package (WP) | A unit of planned work with a card in 10 §10.7, implemented on one branch through one pull request. |
| Spec Kit feature | The `specs/NNN-slug/` folder Spec Kit creates for a work package (spec, plan, tasks). |

---

## 0.10 Owner decisions (OD-xx)

| ID | Decision | Status | Default / consequence |
|---|---|---|---|
| OD-01 | License | **Decided:** AGPL-3.0-or-later with a trademark policy and DCO (rationale in 02 ADR-016) | — |
| OD-02 | Target platform | **Decided:** Windows 11 x64 only for 1.x; ARM64 in 1.1 roadmap | — |
| OD-03 | Locales at 1.0 | **Decided:** en (source), ar, fr, es, de, pt-BR (12 §12.14) | More locales arrive by community pull requests |
| OD-04 | Telemetry | **Decided:** none. Local diagnostics only. | — |
| OD-05 | Paid services | **Decided:** none required. GitHub free features only. | — |
| OD-06 | Code signing | **Default:** unsigned builds with SHA-256 checksums and GitHub build-provenance attestations. **Optional:** apply to the free SignPath Foundation program for open-source projects; enable when approved (08 OPS-10). | Unsigned installers show a SmartScreen warning until reputation builds. |
| OD-07 | Trademark registration of "AfterLeads" and the product name | **Default:** unregistered marks used with the TRADEMARKS.md policy. Registration is optional and outside the codebase. | — |
| OD-08 | Plan checkpoint per work package | **Default:** the owner approves each Spec Kit `plan.md` before `/speckit.implement` (06 §6.12). The owner may waive it per work package. | — |
| OD-09 | Default speech-to-text model | **Decided:** MDL-03 (small, quantized) | — |
| OD-10 | Merge authority | **Decided:** the owner merges pull requests. Agents may merge only when told to in the current session (G-18). | — |

---

## 0.11 Change control for documentation

1. Documentation changes use a `docs/<NNN>-<slug>` branch from `main` (10 §10.4) and a pull request like any code change.
2. Bump the "Set version" in this file: patch for clarifications, minor for new or changed requirements, major for scope changes.
3. Run `python tools/docs/check_docs.py` (VER-01..VER-10) and fix every finding before pushing.
4. A change to a G-rule updates `docs/00-index.md`, `AGENTS.md`, and `.specify/memory/constitution.md` in the same commit (VER-05).

---

## 0.12 What to read for each kind of task

| Task | Read |
|---|---|
| Any work package | Its card in 10 §10.7, then the sections it lists |
| Rust engine work | 02 §2.5–2.10, 03 §3.3–3.5, 04 §4.3–4.8 |
| UI work | 07 (design system and the specific panel/dialog), 04 §4.2–4.4, 12 (commands, shortcuts) |
| Data/persistence | 03 entire |
| Security-sensitive work (parsers, IPC, updates, downloads) | 05 entire |
| Tests | 09 §9.3–9.10 and the T-IDs in the card |
| Release/CI | 08 entire, 09 §9.11 |
| Docs | this file §0.3, §0.11 and 11 |
