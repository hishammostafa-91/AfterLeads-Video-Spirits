# 06 — AI and Automation

Canonical for: (A) AI features inside the product — speech-to-text, caption segmentation, model management, AI roadmap; (B) the agentic development system that builds the product — OpenCode v2 configuration, agents (AGT), commands (CMD), Spec Kit workflow, session protocol, anti-hallucination protocol, parallel work.
Depends on: [00-index.md](00-index.md) (G-rules), [10-execution.md](10-execution.md) (git protocol, work packages), [12-catalogs.md](12-catalogs.md) (models).

---

# Part A — AI in the product

## 6.1 Scope

| Feature | Release | Technology | Runs where |
|---|---|---|---|
| Speech-to-text captions and transcripts (REQ-STT-001..006) | 1.0 | whisper.cpp via whisper-rs, models MDL-01..05 | Worker process, CPU or Vulkan GPU |
| Voice denoise (AFX-06) | 1.0 | RNNoise network via `nnnoiseless` | Audio engine, real time |
| Text-to-speech incl. Arabic (REQ-AUD-011) | 1.1 | Local neural TTS (Piper family, GPL-3.0 engine; per-voice license review) | Worker |
| Silence detection / jump cuts (REQ-EDT-019) | 1.1 | Loudness series (non-ML) | Worker |
| Motion tracking (REQ-FX-013) | 1.1 | Feature tracking (non-ML) | Worker |

Rules: everything runs locally; no user data is used for training; every model has a recorded license and SHA-256 (05 SEC-11); AI output is always editable and never overwrites user work (REQ-STT-004).

## 6.2 Speech-to-text pipeline

1. **Input:** selected clips (each clip's audio within its in/out) or the sequence mixdown (rendered by the offline audio graph in the worker).
2. **Audio preparation:** decode → downmix to mono → resample to 16 kHz `f32` (FFmpeg swresample).
3. **Inference:** `whisper_full` with `language` (`auto` or the chosen code), `translate`, greedy decoding with temperature fallback (whisper.cpp defaults), `token_timestamps = true`, threads = physical cores (max 8), GPU when the build has Vulkan, `alvs.ai.sttUseGpu` is on, and the device initializes (else E7006 and CPU).
4. **Memory check before start:** required RAM (approx.): MDL-01 0.3 GB, MDL-02 0.4 GB, MDL-03 0.9 GB, MDL-04 1.8 GB, MDL-05 2.2 GB; if available memory is lower, fail with E7005 and suggest a smaller model.
5. **Progress:** whisper progress callback mapped to job progress; cancellation through the abort callback (stops within 1 s).
6. **Output:** `Transcript` (04 §4.6.3) saved to the cache `analysis` (kind `transcript`) and converted into captions by 6.3.

## 6.3 Caption segmentation (words → captions)

Inputs: words with start/end times and the readability settings (03 §3.6 `alvs.captions.*`).

1. Build sentences: split at sentence punctuation (`. ! ? … ؟ 。`) and at pauses ≥ 0.6 s.
2. Build captions from sentences greedily: add words while the caption text fits `maxLines × maxCharsPerLine` characters and duration ≤ `maxDurationSec`; prefer breaking at commas (`, ، ;`) and at pauses ≥ 0.3 s.
3. Line breaking inside a caption: balance two lines (minimize length difference), never split a word, never leave a single short word (≤ 3 characters) alone on a line.
4. Timing: caption start = first word start; end = last word end + 0.2 s, clamped so the gap to the next caption ≥ `minGapFrames`; extend captions shorter than `minDurationSec` into available gaps.
5. Text normalization: trim, collapse spaces, keep the original script; for Arabic keep Arabic punctuation and do not convert digits.
6. Name the new caption track `<Language> (auto)` (localized) and select it in the Captions workspace.

Unit tests cover English, French, and Arabic word lists with known expected captions (T-STT-004).

## 6.4 Model management

- Manifest `assets/models/whisper.json` generated from 12 §12.15 (`id`, `file`, `url`, `size`, `sha256`, `license`).
- Downloads are `modelDownload` jobs (04 §4.6.3) through the network guard (SEC-10): HTTPS, resumable range requests, `.part` file, SHA-256 verification, atomic rename (03 §3.9.7).
- The Model Manager (DLG-09) shows install state, disk usage, free space, and guidance text; deleting a model is confirmed.
- Updating the manifest (new model or hash) is a docs + code change with a fresh hash recorded by `tools/models/hash-models.py`.

## 6.5 Quality gates for speech-to-text

- Test audio: short clips with reference transcripts in en, ar, fr, es, de, pt, taken only from sources dedicated to the public domain or CC0 (for example Common Voice CC0 recordings), each ≤ 30 s, stored under `tests/fixtures/speech/` with a `SOURCES.md` listing origin and license.
- CI (T-STT-001): MDL-01 transcribes the English clip; the transcript must contain at least 80% of the reference words.
- Release gate (T-STT-001, GATE-07): MDL-03 word error rate ≤ 15% for English and ≤ 25% for the other five languages on the fixtures; results are recorded as a baseline and a later regression of more than 3 points fails.

## 6.6 Responsible AI

- Auto-generated captions are labeled "(auto)" until edited; the UI encourages review (Captions workspace opens after transcription).
- No cloud fallback exists; if a model is missing the user is guided to download it (E7001).
- Licenses of all models and voices are listed in third-party notices (REQ-LIC-001).

---

# Part B — The agentic development system

## 6.7 Overview

```text
 Owner (human)
   │ approves specs/plans, reviews and merges PRs, holds secrets, publishes releases
   ▼
 OpenCode v2 (Windows 11, PowerShell 7)
   ├─ primary agents: build (implements), plan (read-only planning)
   ├─ subagents: general, explore (built-in); reviewer, test-runner, docs-keeper, i18n, ui-auditor (custom)
   ├─ commands: Spec Kit /speckit.* + project commands /wp-start /wp-verify /wp-finish /sync-main /doc-check /status
   └─ guardrails: permissions + policies (opencode.jsonc), git hooks (.githooks), CI, branch protection
 Memory layers
   ├─ AGENTS.md (always loaded)          ├─ .specify/memory/constitution.md (Spec Kit principles)
   ├─ docs/00–12 (canonical, on demand)  ├─ specs/NNN-slug/ (per work package spec, plan, tasks)
   └─ STATUS.md (progress ledger)
```

## 6.8 OpenCode v2 setup (Windows 11)

1. Install (OpenCode 2.2.x): `npm install -g @opencode/cli` (Windows package managers are not supported by OpenCode 2), or download the standalone `opencode-windows-x64.zip` from the official site, or use the OpenCode Desktop app for Windows. Verify with `opencode --version`.
2. Shell: OpenCode on Windows prefers `pwsh` (PowerShell 7), then Windows PowerShell, then Git Bash; the repository config pins `"shell": "pwsh"`, so every command in these docs is PowerShell 7 syntax.
3. Provider: connect any provider with `/connect` (the project requires no specific provider). Use the strongest available coding model for `build` and `reviewer`; a cheaper model may be set for `explore` and `i18n` in the user's global config (`~/.config/opencode/opencode.jsonc`), never in the repository.
4. Project configuration lives in `opencode.jsonc` (repository root) and `.opencode/`:
   - `instructions`: `AGENTS.md` is auto-loaded; no other always-on instruction files (keeps context small).
   - `permissions` (ordered, last match wins) and `policies` (hard denies) implement G-03..G-06 and G-15..G-18.
   - `worktree.directory`: `../AfterLeads-Video-Spirits.worktrees` (6.15).
   - `formatter: true` (rustfmt/prettier run after edits).
   - `watcher.ignore`: build outputs and dependencies.
5. Verify the setup with CMD-06 `/status` in a new session; it must print the branch, STATUS summary, and confirm hooks are active.

The exact configuration files are committed in the repository (`opencode.jsonc`, `.opencode/agents/*.md`, `.opencode/commands/*.md`) and are the source of truth for agent behavior; changes require owner approval (G-18).

## 6.9 Agents (AGT)

| ID | Agent | Mode | Purpose | Key permissions |
|---|---|---|---|---|
| AGT-01 | `build` (built-in) | primary | Implements work packages | Repository rules from `opencode.jsonc` |
| AGT-02 | `plan` (built-in) | primary | Reads and plans without editing project files | Read-only |
| AGT-03 | `general` (built-in) | subagent | Multi-step research inside a session | Cannot launch subagents |
| AGT-04 | `explore` (built-in) | subagent | Fast code and docs search | Read-only |
| AGT-05 | `reviewer` | subagent | Reviews a diff against the card, docs, constitution, and tests; reports findings by severity | No edits; read-only git and test commands |
| AGT-06 | `test-runner` | subagent | Runs the verification commands and summarizes failures with file/line | No edits; test and build commands only |
| AGT-07 | `docs-keeper` | subagent | Keeps docs, STATUS.md, and traceability consistent; runs doc checks | Edits only `docs/**`, `STATUS.md`, `specs/**`; asks before each docs edit |
| AGT-08 | `i18n` | subagent | Adds/updates translations for missing keys using the 07 §7.12 glossary | Edits only `app/src/locales/**` |
| AGT-09 | `ui-auditor` | subagent | Runs the UI harness (visual, axe accessibility, RTL screenshots) and reports issues | No edits; UI test commands only |

Subagents never switch branches, push, or merge; they work in the parent's working tree (or are read-only).

## 6.10 Commands (CMD)

| ID | Command | What it does |
|---|---|---|
| CMD-01 | `/wp-start WP-NN` | Checks a clean tree, runs `tools/git/New-WorkBranch.ps1` (branch from up-to-date `main`), opens the WP card, prints the ready-to-paste `/speckit.specify …` line with `SPECIFY_FEATURE_DIRECTORY=specs/0NN-<slug>` |
| CMD-02 | `/wp-verify` | Runs `tools/verify.ps1` (all local gates) and summarizes failures |
| CMD-03 | `/wp-finish` | Runs CMD-02, updates STATUS.md, checks commits (Conventional + sign-off), pushes the branch, opens the PR with `gh pr create --base main` and the template, prints the PR URL. Never merges. |
| CMD-04 | `/sync-main` | Runs `tools/git/Sync-Main.ps1` (merge `origin/main` into the current branch with `--no-ff`) |
| CMD-05 | `/doc-check` | Runs `python tools/docs/check_docs.py` and launches `docs-keeper` on findings |
| CMD-06 | `/status` | Prints branch, upstream, dirty state, hook status, active WP, next WP candidates from STATUS.md |
| CMD-07 | `/review` | Launches `reviewer` on `git diff origin/main...HEAD` |
| CMD-08 | `/translate` | Launches `i18n` for keys missing in any locale |

Spec Kit adds `/speckit.constitution`, `/speckit.specify`, `/speckit.clarify`, `/speckit.plan`, `/speckit.checklist`, `/speckit.tasks`, `/speckit.analyze`, `/speckit.implement`, `/speckit.converge` (installed under `.opencode/commands/`).

## 6.11 Spec Kit integration (version 1.2.5)

**Why Spec Kit is used:** each work package is large enough that a written spec, a plan, and a dependency-ordered task list — checked by `/speckit.analyze` before coding and by `/speckit.converge` after — measurably reduces drift and invented scope. The canonical docs remain the source of truth; Spec Kit artifacts are the per-package working record.

**Install (once per workstation):**

```powershell
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v1.2.5
uv tool update-shell
```

Open a new PowerShell 7 window (PATH change), then check with `specify version`.

**Initialize (once, in WP-00, on its branch `chore/000-bootstrap`):**

```powershell
specify init --here --force --integration opencode --script ps --non-interactive
```

- The existing `.specify/memory/constitution.md` is preserved by `specify init`.
- Do **not** run `specify extension add git`: branching is owned by `tools/git/New-WorkBranch.ps1` so every branch starts from `main` (G-04).
- Commit the generated `.specify/` and `.opencode/commands/speckit.*.md` files.

**Per work package flow (owner types the slash commands; the agent does the work inside each):**

| Step | Command | Inputs and rules |
|---|---|---|
| 1 | `/wp-start WP-NN` | Creates `<type>/0NN-<slug>` from `main`; prints the next line |
| 2 | `/speckit.specify SPECIFY_FEATURE_DIRECTORY=specs/0NN-<slug> <WP brief from 10 §10.7>` | The spec MUST list only the REQ/NFR IDs of the card and cite canonical sections; no new requirements |
| 3 | `/speckit.clarify` (only if the spec has open questions) | Answers come from the docs; if the docs do not answer, stop and ask the owner (G-17) |
| 4 | `/speckit.plan <tech constraints: crates/files from 02 §2.13, interfaces from 04, data from 03>` | The plan MUST reuse the architecture; deviations require an ADR (stop) |
| 5 | Owner checkpoint (OD-08) | Owner reads `plan.md` and replies "approved" or requests changes |
| 6 | `/speckit.tasks` | Tasks reference test IDs (T-xx) from the card |
| 7 | `/speckit.analyze` | Must report no CRITICAL/HIGH issues; fix at the source and re-run |
| 8 | `/speckit.implement <phase scope>` | Implement in stages (setup → foundational → each user story → polish); run `/wp-verify` after each stage; commit per task group |
| 9 | `/speckit.converge` | Repeat 8–9 until "Converged" |
| 10 | `/review` then `/wp-finish` | Reviewer findings fixed or justified; PR opened |

Skip Spec Kit for: documentation-only changes, dependency bumps, and fixes smaller than ~50 changed lines (use a `fix/` or `docs/` branch with a normal PR).

## 6.12 Session protocol

**At session start (every session):**

1. Run `/status`. If the current branch is `main` and work is requested, run `/wp-start` first (never edit on `main`).
2. Read STATUS.md and the active WP card; read only the "Required reading" sections.
3. If resuming, read `specs/0NN-*/tasks.md` to see completed checkboxes.

**During work:**

1. Small steps: implement one task, build, run its tests, commit (`git commit -s -m "<type>(<scope>): <summary>"`).
2. After any context compaction, re-read the WP card and `tasks.md` before continuing.
3. Keep the tree green: never leave the branch in a non-building state at the end of a session.

**At session end:**

1. Run `/wp-verify`; commit; push the branch (`git push -u origin <branch>`; never `main`).
2. Update the WP row in STATUS.md (state, last step, next step).

**Stop and ask the owner** (G-17) when: the card or docs are ambiguous or contradictory; a needed API/version cannot be verified; a new dependency or ADR seems necessary; a gate cannot pass without weakening it; a destructive action is needed; a security, privacy, or licensing question arises; the plan checkpoint (step 5) is reached.

**Evidence format in PRs** (G-09): commands run with their summarized output, test IDs passed, screenshots for UI changes (dark/light, en/ar), performance numbers for hot paths.

## 6.13 Anti-hallucination protocol

1. **Verification ladder for any API, flag, version, or path:** (a) this repository's code; (b) the exact dependency source (`%USERPROFILE%\.cargo\registry\src\…`, `app\node_modules\…`) or `cargo doc --open -p <crate>`; (c) official documentation for the locked version; (d) only then general knowledge — and it must be confirmed by compiling and testing.
2. **Cite or stop:** design choices in specs/plans cite a doc section or ADR. If none exists, stop and ask.
3. **Compile early, test early:** write the smallest compiling slice first; never write large untested code.
4. **No placeholders in merged code:** `todo!()`, `unimplemented!()`, `TODO`/`FIXME` comments, mocked production paths, or commented-out code are not allowed on `main` (CI greps for them; test code and the mock backend are exempt).
5. **No silent scope:** if implementation needs behavior not in the docs, stop; do not "improve" requirements.
6. **Determinism:** tests do not depend on time, network, GPU vendor, or randomness without a seed.
7. **Identity and names:** use 00 §0.5 constants and catalog IDs exactly; never invent product, file, or command names.
8. **Honest reporting:** if something is not done or not verified, say so in the PR and STATUS.md.

## 6.14 Context management

- One work package per session when possible; very large packages are split by Spec Kit phases across sessions, with `tasks.md` checkboxes as the handoff record.
- Keep `AGENTS.md` under 12 KB; never auto-load the full docs; read sections on demand.
- Compaction keeps the last ~15k tokens (OpenCode default); after compaction re-read the card (6.12).

## 6.15 Parallel work (worktrees)

- Default: one work package at a time in the main checkout.
- Allowed parallelism: at most two work packages whose cards list no dependency between them (10 §10.8 lanes), each in its own worktree and branch created from `main`:

```powershell
pwsh -File tools/git/New-Worktree.ps1 -WorkPackage WP-17 -Slug titles-generators
```

- Each worktree runs its own OpenCode session. Worktrees never share a branch. Merge order follows the dependency graph; the second PR syncs `main` (CMD-04) before merging.

## 6.16 Owner duties

Approve plans (OD-08), answer stop-condition questions, review and merge PRs (OD-10), manage secrets (update signing key, optional code-signing setup), publish releases, and run release-gate tests on the reference machines (09 §9.2).
