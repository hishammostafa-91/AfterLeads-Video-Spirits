# 10 — Execution

Canonical for: workstation setup, repository bootstrap, the git protocol, the work-package protocol, milestones (M0–M9), work packages (WP-00–WP-29), dependency graph, Definition of Done, STATUS.md protocol, and execution risks (R).
Depends on: every other document; the cards below list exactly which sections to read.

---

## 10.1 How to use this document

- **Owner:** set up the workstation (10.2, which includes the one-time `main` bootstrap of 10.4.14), then for each work package start a session, run the commands in 06 §6.11, approve plans, review and merge pull requests.
- **Agent:** follow `AGENTS.md`; for the active work package read its card (10.7) and only the listed sections; never start a package whose dependencies (10.8) are not `done` in STATUS.md.

All commands are PowerShell 7 (`pwsh`), run from the repository root unless stated.

---

## 10.2 Windows 11 workstation setup

### 10.2.1 Requirements

Windows 11 x64 22H2 or newer, administrator rights for installation, 60 GB free disk (vcpkg builds, caches), 16 GB RAM recommended, internet access.

| Tool | winget ID | Purpose |
|---|---|---|
| Git for Windows | `Git.Git` | Version control, `sh` for git hooks |
| PowerShell 7 | `Microsoft.PowerShell` | Shell for OpenCode and scripts |
| Visual Studio 2022 Build Tools | `Microsoft.VisualStudio.2022.BuildTools` | MSVC compiler, linker, Windows 11 SDK |
| Rustup | `Rustlang.Rustup` | Rust toolchain (version pinned by `rust-toolchain.toml`) |
| LLVM | `LLVM.LLVM` | `libclang` for FFmpeg bindings |
| Node.js 26 LTS | `OpenJS.NodeJS.LTS` | UI toolchain |
| pnpm | `pnpm.pnpm` | Package manager |
| Python 3.12 | `Python.Python.3.12` | Repository tools |
| uv | `astral-sh.uv` | Installs Spec Kit |
| GitHub CLI | `GitHub.cli` | Pull requests |
| CMake, Ninja | `Kitware.CMake`, `Ninja-build.Ninja` | Native builds (whisper.cpp) |
| WebView2 Runtime | `Microsoft.EdgeWebView2Runtime` | Present on Windows 11; listed for completeness |
| Vulkan SDK (optional) | install from the LunarG website | GPU speech-to-text builds |

### 10.2.2 Steps

Keep this order: the repository scripts need Git and PowerShell 7, and they run from the clone.

1. Open **Windows PowerShell** (preinstalled) and install Git, PowerShell 7, and the GitHub CLI. Then close the window; every later step uses **PowerShell 7** (`pwsh`).

```powershell
winget install --id Git.Git --exact --source winget --accept-package-agreements --accept-source-agreements
winget install --id Microsoft.PowerShell --exact --source winget --accept-package-agreements --accept-source-agreements
winget install --id GitHub.cli --exact --source winget --accept-package-agreements --accept-source-agreements
```

2. Open **PowerShell 7 as administrator** once and allow local scripts and long paths:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name LongPathsEnabled -Value 1 -PropertyType DWORD -Force
```

3. Configure git identity and safe defaults:

```powershell
git config --global user.name "AfterLeads"
git config --global user.email "afterleadseg@gmail.com"
git config --global init.defaultBranch main
git config --global core.autocrlf false
git config --global core.longpaths true
git config --global pull.ff only
git config --global merge.ff false
git config --global fetch.prune true
```

4. Sign in to GitHub with `gh auth login`: choose GitHub.com, HTTPS, yes to authenticating Git with your GitHub credentials, and the web browser login.
5. Only while the repository has no `main` branch: do the one-time bootstrap in 10.4.14 now, before cloning.
6. Clone into a short path such as `C:\dev` (vcpkg, Cargo, and pnpm create deep folder trees) and install the repository hooks. `git status` must print `## main...origin/main`.

```powershell
New-Item -ItemType Directory -Force -Path C:\dev | Out-Null
Set-Location C:\dev
git clone https://github.com/hishammostafa-91/AfterLeads-Video-Spirits.git
Set-Location C:\dev\AfterLeads-Video-Spirits
git status --short --branch
pwsh -File tools/git/Install-Hooks.ps1
```

7. Install the remaining tools from **PowerShell 7 as administrator** in the repository folder. The script installs missing tools with winget, prints what it did, and is safe to re-run. Run it, open a **new** PowerShell 7 window (PATH changes), and run it again to install the pinned cargo tools. If `cargo --version` reports that no default toolchain is configured, run `rustup default stable` and run the script once more.

```powershell
Set-Location C:\dev\AfterLeads-Video-Spirits
pwsh -File tools/setup/Install-DevTools.ps1
```

8. In the Visual Studio Installer, make sure the Build Tools have the workload **Desktop development with C++** with **MSVC v143** and the **Windows 11 SDK** (the script requests them; verify visually once).
9. vcpkg (once; the pinned baseline commit is set in `vcpkg-configuration.json` from WP-06):

```powershell
git clone https://github.com/microsoft/vcpkg.git "$env:USERPROFILE\vcpkg"
& "$env:USERPROFILE\vcpkg\bootstrap-vcpkg.bat" -disableMetrics
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "$env:USERPROFILE\vcpkg", "User")
```

10. OpenCode 2 and Spec Kit (`uv tool update-shell` puts the folder of uv-installed tools on PATH):

```powershell
npm install -g @opencode/cli
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v1.2.5
uv tool update-shell
```

11. Open a new PowerShell 7 window and verify everything; fix every `FAIL` before starting work:

```powershell
Set-Location C:\dev\AfterLeads-Video-Spirits
opencode --version
specify version
pwsh -File tools/setup/Test-DevEnvironment.ps1
```

12. Start OpenCode in the repository (`opencode`), run `/connect` to choose any model provider, then `/status`. The first work package is WP-00 (10.7), started with `/wp-start WP-00`.

---

## 10.3 Repository baseline

The baseline commit contains: `docs/00–12`, `AGENTS.md`, `README.md`, `LICENSE`, `TRADEMARKS.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `STATUS.md`, `opencode.jsonc`, `.opencode/agents/`, `.opencode/commands/`, `.specify/memory/constitution.md`, `.githooks/`, `.github/` (pull-request and issue templates), `tools/` (setup, git, docs, verify), `.gitattributes`, `.editorconfig`, `.gitignore`. No product code exists yet; WP-00 creates the skeleton.

---

## 10.4 Git protocol (canonical)

### 10.4.1 Model

GitHub Flow with merge commits: `main` is always releasable and protected (08 OPS-02); every change is a short-lived branch from `main`, merged back through a pull request with a merge commit; releases are signed tags on `main`. There are no long-lived `develop` or `release` branches.

### 10.4.2 Branch names

`<type>/<NNN>-<slug>` where `type` ∈ `feat`, `fix`, `docs`, `chore`, `ci`, `refactor`, `test`, `perf`, `i18n`, `build`; `NNN` = the work-package number (`000`–`029`) or the GitHub issue number (3–6 digits); `slug` = lowercase words joined by `-`. Examples: `feat/012-editing-engine`, `fix/154-relink-crash`, `docs/030-clarify-captions`. The pre-push hook rejects other names.

### 10.4.3 Starting work (always from up-to-date `main`)

```powershell
pwsh -File tools/git/New-WorkBranch.ps1 -Type feat -Number 12 -Slug editing-engine
```

The script refuses to run with uncommitted changes, then runs `git fetch origin`, `git switch main`, `git pull --ff-only`, and `git switch -c feat/012-editing-engine`. Manual equivalent:

```powershell
git switch main
git pull --ff-only
git switch -c feat/012-editing-engine
```

### 10.4.4 Committing

- One logical change per commit; the tree builds and tests pass at every commit.
- Message: Conventional Commits, first line ≤ 100 characters, then a blank line and a body explaining why; trailer `Signed-off-by` from `git commit -s` (DCO, required).
- Hooks (10.4.13) enforce: not on `main`, message format, sign-off, no secrets, no files larger than 5 MB.

```powershell
git add -A
git commit -s -m "feat(model): add trim EditOp with handle limits" -m "Implements REQ-EDT-004 (ripple/roll) per docs/04 §4.4."
```

### 10.4.5 Commit types and scopes

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`, `i18n`.
Scopes: `core`, `model`, `formats`, `media`, `render`, `audio`, `engine`, `store`, `jobs`, `stt`, `testkit`, `app`, `ui`, `timeline`, `viewer`, `inspector`, `export`, `captions`, `i18n`, `a11y`, `docs`, `ci`, `deps`, `release`, `tools`, `agents`.

### 10.4.6 Keeping a branch current

When `main` moved (or before opening a pull request), merge it into the branch — never rebase a pushed branch:

```powershell
pwsh -File tools/git/Sync-Main.ps1
```

(= `git fetch origin` then `git merge --no-ff origin/main`, resolving conflicts, running gates, committing.)

### 10.4.7 Pull requests and merging

1. Push: `git push -u origin <branch>` (never `main`).
2. Open: `gh pr create --base main --fill` with the template (`.github/pull_request_template.md`), linking the WP card and listing REQ/T IDs and evidence (G-09).
3. CI must be green; the owner reviews with the checklist (08 OPS-04).
4. The **owner** merges with "Create a merge commit" (the only enabled method, G-05) and deletes the branch. Agents merge only when explicitly told to in the current session: `gh pr merge <number> --merge --delete-branch`.

### 10.4.8 After merge

```powershell
git switch main
git pull --ff-only
git branch -d <branch>
```

### 10.4.9 Stacked branches (exception)

A branch may be created from another unmerged branch only when the owner approves it because work package B cannot wait for A. The pull request of B targets `main`, states "Stacked on #<A>", and is merged only after A; B then merges `origin/main` (10.4.6) before its own merge.

### 10.4.10 Parallel work with worktrees

```powershell
pwsh -File tools/git/New-Worktree.ps1 -Type feat -Number 17 -Slug titles-generators
```

Creates `../AfterLeads-Video-Spirits.worktrees/feat-017-titles-generators` on a new branch from `origin/main`. Each worktree has its own OpenCode session. Remove after merge: `git worktree remove <path>`. Limits in 06 §6.15.

### 10.4.11 Forbidden operations (agents and humans)

`git push` to `main`; `git push --force`/`-f`/`+refspec`; `git commit --no-verify`; changing `core.hooksPath`; `git rebase` of pushed commits; `git reset --hard` or `git clean -fdx` without owner approval; deleting unmerged branches; creating or pushing tags (owner only, 08 OPS-09); editing history of `main`. OpenCode policies (06 §6.8) and server-side protections (08 OPS-02) enforce most of these.

### 10.4.12 Recovery playbooks

| Situation | Fix |
|---|---|
| Committed on `main` locally (hook bypassed or misconfigured) | `git branch <type>/<NNN>-<slug>` (keeps the commits), `git switch main`, `git reset --hard origin/main` (owner approval), `git switch <new branch>` |
| Branch created from the wrong base | Create a fresh branch from `main` and `git cherry-pick` the commits; delete the wrong branch after the new PR is open |
| Merge conflict during sync | Resolve file by file, keep both behaviors when unsure and stop to ask; run all gates before committing the merge |
| Secret committed | Stop; do not push; tell the owner; if already pushed, rotate the secret immediately (05 SEC-15) — history rewriting needs the owner |
| Hooks not running | `pwsh -File tools/git/Install-Hooks.ps1`; `git config --get core.hooksPath` must print `.githooks` |

### 10.4.13 Hooks

| Hook | Checks |
|---|---|
| `pre-commit` | Current branch is not `main`/`master`; no staged file larger than 5 MB; no private-key blocks or `.env` files staged |
| `commit-msg` | Conventional Commits first line (merge commits allowed); `Signed-off-by:` trailer present |
| `pre-push` | No push to `main`/`master`; no tag push unless `ALVS_OWNER_RELEASE=1` is set by the owner; branch name matches 10.4.2 |

### 10.4.14 Bootstrap exception (one time)

The baseline documentation arrives on a single baseline branch while the repository has no `main`. The owner creates `main` from that branch once, on GitHub, before cloning (10.2.2 step 5). A local `git push` to `main` is rejected by the pre-push hook by design, so use the GitHub CLI:

```powershell
$repo = 'hishammostafa-91/AfterLeads-Video-Spirits'
$sha = gh api "repos/$repo/git/ref/heads/<baseline-branch>" --jq '.object.sha'
gh api --method POST "repos/$repo/git/refs" -f ref=refs/heads/main -f "sha=$sha"
gh repo edit $repo --default-branch main
```

The same on the website: the repository's Branches page → **New branch** named `main` with the baseline branch as the source, then Settings → General → Default branch → `main`.

Then apply OPS-02 (08 §8.2) except the required status checks: they name CI jobs that exist only after WP-01, and requiring them earlier blocks every pull request. The WP-01 pull request lists the checks for the owner to add after it merges. Finally delete the baseline branch on GitHub. This is the only time `main` changes without a pull request.

---

## 10.5 Work-package protocol

States in STATUS.md: `todo` → `in-progress` → `in-review` (PR open) → `done` (merged) or `blocked` (waiting for the owner).

1. Check dependencies are `done` (10.8).
2. `/wp-start WP-NN` (creates the branch, prints the Spec Kit line).
3. Spec Kit flow (06 §6.11) including the owner plan checkpoint (OD-08).
4. Implement in stages; `/wp-verify` after each stage; commit.
5. `/speckit.converge` until converged; `/review`; fix findings.
6. `/wp-finish` (push, PR, STATUS `in-review`).
7. After the owner merges: STATUS `done` (the owner or the next session updates it on `main` through the next work branch).

---

## 10.6 Milestones

| Milestone | Work packages | Exit criteria (demonstrable) |
|---|---|---|
| M0 Foundation | WP-00..WP-03 | CI green on `main`; project model with undo, save/open/autosave, settings, logs — all tested |
| M1 Shell | WP-04, WP-05 | App window in six locales with themes, docking, menus, command palette, Welcome screen (mock backend + real backend for project open/save) |
| M2 Media | WP-06..WP-08 | Import the generated corpus; thumbnails, waveforms, proxies run as jobs; media panel complete |
| M3 Engine and playback | WP-09..WP-11 | A clip plays in the Program viewer with synced audio at target performance; WARP fallback works |
| M4 Timeline | WP-12, WP-13 | Full editing on the timeline with undo, keyboard model, and list view |
| M5 Effects, transitions, titles | WP-14..WP-17 | All VFX/TRS/TAN/TPL/GEN catalog items work with keyframes and golden tests |
| M6 Audio and color | WP-18, WP-19 | Mixer, AFX, loudness, voice-over, scopes, color workspace |
| M7 Captions and AI | WP-20, WP-21 | Captions end to end including Arabic; local transcription |
| M8 Export and interchange | WP-22, WP-23 | All presets export and verify; queue; EDL/markers; optimize/stabilize; collect files |
| M9 Productization and release | WP-24..WP-29 | Settings, localization, accessibility, help, reliability, packaging, updates; GATE-01..GATE-12 pass; v1.0.0 tagged |

---

## 10.7 Work packages

Each card lists: branch, dependencies, requirements implemented, required reading, deliverables, steps, acceptance, and exclusions. "Tests" are IDs from 09 §9.10 that MUST pass before the PR.

### WP-00 — Workstation and repository bootstrap

- **Branch:** `chore/000-bootstrap` · **Depends on:** baseline on `main` (10.4.14)
- **Implements:** foundations for G-03..G-08 (no product REQ)
- **Read:** 00 (all); 02 §2.13–2.14; 06 §6.8–6.11; 10 §10.2–10.4
- **Deliverables:** verified workstation; active hooks; Spec Kit initialized (`.specify/`, `.opencode/commands/speckit.*.md`); Cargo workspace (`Cargo.toml` with members `crates/*` and `app/src-tauri`, `rust-toolchain.toml` = `1.99.1` with `rustfmt`, `clippy`; `.cargo/config.toml` with `rustflags = ["-C", "target-feature=+crt-static", "-C", "control-flow-guard"]` for `x86_64-pc-windows-msvc`); empty crates `alvs-core` … `alvs-testkit` with SPDX headers and `lib.rs` docs; Tauri 2 app scaffold in `app/` (React + TypeScript template) adjusted to 00 §0.5 (product name, identifier, window title); root `package.json` + `pnpm-workspace.yaml`; TypeScript pinned to `6.0.x`; `tools/verify.ps1` passing on the skeleton; STATUS.md WP-00 `done` after merge.
- **Steps:**
  1. Complete every step of 10.2.2; fix anything `Test-DevEnvironment.ps1` reports.
  2. `/wp-start WP-00`; run `specify init --here --force --integration opencode --script ps --non-interactive`; confirm the constitution file is unchanged (`git diff .specify/memory/constitution.md` empty).
  3. Create the Cargo workspace and crates; `cargo build --workspace` must pass.
  4. Scaffold the app with `pnpm create tauri-app@latest` (React, TypeScript, pnpm) into `app/`; set `productName` "AfterLeads-Video-Spirits", `identifier` "com.afterleads.videospirits", window title "AfterLeads-Video-Spirits"; pin dependencies to the 00 §0.8 baselines; remove template demo code and assets; add SPDX headers.
  5. Ensure `pnpm -C app build` and `pnpm -C app tauri dev` open the empty window.
  6. Commit in logical steps; `/wp-finish`.
- **Acceptance:** `Test-DevEnvironment.ps1` all green; `cargo build --workspace --locked`; `pnpm -C app build`; window opens with the correct title; `python tools/docs/check_docs.py` green; a test commit on `main` is rejected by the hook (manual check recorded in the PR).
- **Not included:** CI workflows (WP-01), FFmpeg (WP-06), real UI (WP-04/05).

### WP-01 — CI and quality gates

- **Branch:** `ci/001-ci-quality-gates` · **Depends on:** WP-00
- **Implements:** NFR-MNT-01, NFR-MNT-02, NFR-SEC-01 (gates)
- **Read:** 05 §5.6; 08 §8.2–8.4; 09 §9.1
- **Deliverables:** `.github/workflows/ci.yml` (jobs of 08 §8.3.1 that apply to the current skeleton; `e2e-smoke` added as a no-op job that reports success until WP-05), `nightly.yml` (dependency audit + docs links now; more jobs added by later WPs), `.github/dependabot.yml`, `deny.toml` (allowlist of 05 §5.6, bans: HTTP clients except allowed crates), `tools/check_deps.py` (02 §2.4 rules), `tools/check_placeholders.py`, `tools/git/check_commits.py` (exists; wire it), `tools/check_npm_licenses.py`, actions pinned to commit SHAs, owner checklist for OPS-02 in the PR body.
- **Acceptance:** CI green on the PR; deliberately failing commits (format, lint, placeholder, missing sign-off) fail the right job (evidence in PR); T-SEC-001, T-QA-002 baseline recorded.
- **Not included:** release workflow (WP-28).

### WP-02 — Core types and project model

- **Branch:** `feat/002-core-model` · **Depends on:** WP-01
- **Implements:** REQ-PRJ-001 (model), REQ-PRJ-005 (model), REQ-PRJ-007 (undo engine), REQ-PRJ-008
- **Read:** 02 ADR-008/009, §2.4, §2.11; 03 §3.2–3.5, §3.9.6; 04 §4.2, §4.4, §4.9; 12 §12.8–12.9, §12.13
- **Deliverables:** `alvs-core` (flicks, rational, timecode incl. drop-frame, IDs, `AppError` + codes, path policy SEC-04); `alvs-model` (all 03 §3.4 types with `extra` maps, validation of 03 §3.5, `EditOp` enum with application for project/sequence/track/media/bin/marker/caption-track ops, undo/redo with Arc snapshots and 512 MB budget, `EntityPatch` diffing, JSON (de)serialization, JSON Schema generation, migrations framework with v1 fixtures, ts-rs export test).
- **Acceptance:** T-PRJ-005, T-PRJ-006, T-PRJ-007, T-PLY-005 (timecode functions), property tests of 09 §9.4 for implemented ops; coverage ≥ 80% (NFR-MNT-01); `cli`-free.
- **Not included:** clip editing semantics (WP-12), persistence services (WP-03).

### WP-03 — Storage and application services

- **Branch:** `feat/003-storage-services` · **Depends on:** WP-02
- **Implements:** REQ-PRJ-002, REQ-PRJ-003, REQ-PRJ-004, REQ-PRJ-011, REQ-SET-003, REQ-REL-001 (services), REQ-REL-003, REQ-REL-007 (save path)
- **Read:** 03 §3.6–3.9; 02 §2.5, §2.11; 05 SEC-03/04
- **Deliverables:** `alvs-store` (paths incl. portable mode, settings store with schema/defaults/backup, SQLite cache with migrations of 03 §3.7, recent projects, atomic write, project save/open with relative-path fallback, autosave scheduler, backups, lock files, session marker), logging setup (tracing, rolling files, redaction helper).
- **Acceptance:** T-PRJ-002, T-PRJ-003, T-PRJ-004, T-PRJ-008, T-SET-003, T-REL-003, T-REL-007 (save part).

### WP-04 — Design system and UI foundation

- **Branch:** `feat/004-design-system` · **Depends on:** WP-01
- **Implements:** REQ-APP-009, REQ-APP-010, REQ-I18N-001 (infrastructure), REQ-I18N-002 (infrastructure), REQ-I18N-003, REQ-I18N-004 (infrastructure), REQ-I18N-007 (UI fonts), REQ-A11Y-002 (components), REQ-A11Y-003, REQ-A11Y-004, REQ-HLP-003 (components)
- **Read:** 07 §7.1, §7.4–7.6, §7.11–7.13; 02 ADR-011, §2.14; 12 §12.14, §12.16, §12.17
- **Deliverables:** tokens and themes (dark, light, high contrast, accents); fonts FNT-01..03 bundled with license files; component library of 07 §7.6 on React Aria Components with harness stories; i18n runtime (react-intl, locale loading, `dir`/`lang`, pseudo-locales LOC-90/91 in tests), `en.json` + five locales with the initial keys, `en.meta.json`; ESLint/Stylelint/Prettier configs (i18n literal rule, jsx-a11y, logical properties); UI harness (Vite + mock backend skeleton + Playwright + axe).
- **Acceptance:** T-UI-001, T-UI-002, T-I18N-003, T-I18N-004, T-A11Y-004; harness snapshots for all components in dark/light × en/ar.

### WP-05 — Application shell

- **Branch:** `feat/005-app-shell` · **Depends on:** WP-03, WP-04
- **Implements:** REQ-APP-001, REQ-APP-002, REQ-APP-003 (built-in workspaces), REQ-APP-004, REQ-APP-005, REQ-APP-006, REQ-APP-007 (framework), REQ-APP-008, REQ-APP-010 (window DPI), REQ-APP-011, REQ-APP-012, REQ-PRJ-001 (DLG-01), REQ-I18N-005, REQ-A11Y-001, REQ-A11Y-002 (shell), REQ-HLP-003
- **Read:** 07 §7.2–7.3, §7.7 (shell parts), §7.8–7.10, §7.14 (DLG-01, DLG-20); 04 §4.3.1–4.3.2, §4.10; 12 §12.10–12.11; 05 §5.4
- **Deliverables:** Tauri app modes skeleton (02 §2.3), custom title bar + menu bar from 12 §12.11, toolbar, status bar, dockview layout with WS-01..04, command registry with all 12 §12.10 IDs (unimplemented commands disabled with a tooltip "Available in a later version" — removed by the WP that implements them), keyboard dispatcher by physical keys, command palette, notifications, Welcome screen, New Project and Unsaved Changes dialogs wired to WP-03 services, single instance, capabilities and CSP per 05 §5.4, ts-rs bindings integration, mock backend coverage of every implemented command, `e2e-smoke` job activated with T-E2E-001 steps that exist so far.
- **Acceptance:** T-APP-001..007, T-A11Y-001, T-I18N-005; CSP/capabilities part of T-SEC-005.

### WP-06 — FFmpeg build and media I/O

- **Branch:** `feat/006-media-io` · **Depends on:** WP-01, WP-02
- **Implements:** REQ-MED-001 (decode), REQ-MED-002 (probe library), REQ-MED-006, REQ-MED-007 (library), REQ-MED-008 (decode side), REQ-MED-010
- **Read:** 02 ADR-004, §2.6.3–2.6.4, §2.8 (decoders); 05 SEC-01/02; 12 §12.20; 09 §9.3
- **Deliverables:** `vcpkg.json` (port `ffmpeg` with the ADR-004 features, triplet `x64-windows-static`) and `vcpkg-configuration.json` with a pinned baseline; build integration for `ffmpeg-next` following its documented Windows/vcpkg setup (verify, G-10); CI vcpkg binary cache (08 §8.3.1); `alvs-media` probe, demux, decode (software + D3D11VA with transfer to NV12/P010), PTS-accurate seek, audio decode/resample, deinterlace filter graph, SEC-02 restrictions; `tools/media/generate_test_media.ps1`; `cli probe`.
- **Acceptance:** T-MED-002 (library), T-MED-007, T-MED-008, T-MED-011, T-SEC-003 (media part); `cli probe` prints correct metadata for the whole corpus; the vcpkg-built `ffmpeg -hide_banner -hwaccels` lists `d3d11va` and `ffmpeg -hide_banner -encoders` lists every encoder named in 12 §12.7 and §12.19 (record both outputs in the PR).

### WP-07 — Jobs and workers

- **Branch:** `feat/007-jobs-workers` · **Depends on:** WP-03, WP-06
- **Implements:** REQ-JOB-001..REQ-JOB-004, REQ-APP-007 (jobs summary)
- **Read:** 02 ADR-006/007, §2.3; 04 §4.6; 03 §3.7 (jobs table); 07 PNL-14
- **Deliverables:** worker mode, `alvs-jobs` (queue, priorities and limits of 04 §4.6.4, supervision, retries, persistence, re-queue on restart), `probe` runner, Jobs panel (PNL-14), status bar job summary.
- **Acceptance:** T-JOB-001..T-JOB-004 (including killing workers and hanging workers).

### WP-08 — Media pool, import, and proxies

- **Branch:** `feat/008-media-pool` · **Depends on:** WP-05, WP-07
- **Implements:** REQ-MED-001, REQ-MED-002, REQ-MED-003, REQ-MED-004, REQ-MED-005, REQ-MED-009, REQ-PRJ-010
- **Read:** 03 §3.4.2–3.4.3, §3.7; 04 §4.3.3; 07 PNL-01, DLG-07, §7.8 (media menus); 09 §9.3
- **Deliverables:** import flows (dialog, drag-drop, paste), `addMedia` integration, folders, Media panel, thumbnails/filmstrips/waveform jobs and cache, hover scrub, proxy jobs and proxy mode, offline detection and relink dialog, image sequences.
- **Acceptance:** T-MED-001..T-MED-006, T-MED-010, T-PERF-008.

### WP-09 — Render engine core

- **Branch:** `feat/009-render-core` · **Depends on:** WP-06
- **Implements:** REQ-FX-001 (render), REQ-FX-009 (render support), REQ-FX-011, REQ-COL-003, REQ-MED-008 (color), REQ-REL-006, REQ-EDT-009 (frame mapping)
- **Read:** 02 ADR-003/013, §2.6, §2.10; 04 §4.8, §4.12; 09 §9.5–9.6; 12 §12.1.1, §12.4
- **Deliverables:** `alvs-render` (device/adapter selection with WARP, texture pool, plane upload, color conversions and tone mapping, intrinsic transform/crop/opacity, all 24 blend modes, adjustment-layer support, frame cache, device-loss recovery); `alvs-engine` timeline evaluator (graph construction, speed/reverse/freeze mapping, nested sequences); `cli frame`; golden test harness in `alvs-testkit`.
- **Acceptance:** T-RND-001..T-RND-003, T-COL-002, T-MED-009, T-REL-006, T-FX-001 (render part).

### WP-10 — Audio engine core

- **Branch:** `feat/010-audio-core` · **Depends on:** WP-06
- **Implements:** REQ-AUD-001, REQ-AUD-008, REQ-AUD-009, REQ-AUD-010, REQ-EDT-009 (audio)
- **Read:** 02 ADR-014, §2.7; 09 §9.7
- **Deliverables:** `alvs-audio` graph (clip → track → master), gain/automation/pan/fades, channel modes and 5.1 downmix, time-stretch via `atempo`, cpal output with device management and buffer presets, meters, offline render for export.
- **Acceptance:** T-AUD-001, T-AUD-007, T-AUD-008, T-AUD-009; null and A/V-sync tests of 09 §9.7 at library level.

### WP-11 — Playback and viewers

- **Branch:** `feat/011-playback-viewers` · **Depends on:** WP-05, WP-09, WP-10
- **Implements:** REQ-PLY-001..REQ-PLY-008, REQ-MED-007 (playback), REQ-APP-007 (playback statistics)
- **Read:** 02 ADR-005, §2.8; 04 §4.3.5, §4.5, §4.10; 07 PNL-06, PNL-07, §7.11; 09 §9.13
- **Deliverables:** first task = SharedBuffer spike proving 1080p60 delivery (R-01); transport state machine, clocks, scheduling, scrubbing, preview transport (SharedBuffer + channel fallback), WebGL2 presenter, Program and Source viewers with overlays and zoom, native full-screen/external window, statistics.
- **Acceptance:** T-PLY-001..T-PLY-008, T-PERF-003, T-PERF-004, T-PERF-006.

### WP-12 — Timeline editing engine

- **Branch:** `feat/012-editing-engine` · **Depends on:** WP-02
- **Implements:** REQ-EDT-001..REQ-EDT-004, REQ-EDT-007..REQ-EDT-010 (model), REQ-EDT-014, REQ-PRJ-005, REQ-PRJ-006, REQ-EDT-009 (model)
- **Read:** 03 §3.4.5–3.4.6, §3.4.10, §3.5; 04 §4.4; 09 §9.4
- **Deliverables:** every clip/track/marker/sequence `EditOp` of 04 §4.4 with exact semantics (insert/overwrite/append/replace, lift/extract, ripple delete, trims with handle limits, slip, slide, split, gaps, link/group, nesting, speed/freeze, paste attributes, markers) and property tests.
- **Acceptance:** T-EDT-001, T-EDT-002, T-EDT-003, T-EDT-005, T-EDT-006, T-EDT-007, T-EDT-008 (model), T-EDT-009, T-EDT-011, T-PRJ-005.
- **Note:** can run in parallel with WP-06..WP-08 (lane B, 10.8).

### WP-13 — Timeline UI

- **Branch:** `feat/013-timeline-ui` · **Depends on:** WP-08, WP-11, WP-12
- **Implements:** REQ-EDT-001..REQ-EDT-013, REQ-EDT-015..REQ-EDT-017 (UI), REQ-AUD-007, REQ-PRJ-007 (History panel)
- **Read:** 07 §7.7 PNL-08/PNL-13, §7.7.1, §7.8, §7.9; 02 ADR-012, §2.15; 12 §12.10 (timeline commands)
- **Deliverables:** canvas timeline (rendering, hit testing, tools, gestures with local preview and single commit, snapping, zoom/scroll/follow), track headers, caption lanes placeholder, markers lane, thumbnails/waveforms, keyboard model and list view, History panel, Markers panel (PNL-12), dual-frame trim view in Program viewer.
- **Acceptance:** T-EDT-001..T-EDT-005, T-EDT-007, T-EDT-009, T-EDT-010, T-EDT-012, T-A11Y-002, T-PERF-005, T-PRJ-006 (UI).

### WP-14 — Effects framework and keyframes

- **Branch:** `feat/014-effects-framework` · **Depends on:** WP-09, WP-13
- **Implements:** REQ-FX-001 (UI), REQ-FX-002, REQ-FX-003, REQ-FX-005, REQ-FX-010, REQ-FX-012, REQ-PLY-009
- **Read:** 04 §4.8; 03 §3.4.7; 07 PNL-02, PNL-05, §7.6 (controls); 12 §12.5; 02 §2.6.6
- **Deliverables:** manifest loader/validator, parameter evaluation with all easings, keyframe engine, Inspector auto-generated UI, Effects panel, effect stack operations, presets, graph editor, on-viewer handles and eyedropper.
- **Acceptance:** T-FX-002, T-FX-003, T-FX-005.

### WP-15 — Video effects, masks, keying, LUTs

- **Branch:** `feat/015-video-effects` · **Depends on:** WP-14
- **Implements:** REQ-FX-004, REQ-FX-006, REQ-FX-007, REQ-FX-008, REQ-INT-003, REQ-COL-001 (effects)
- **Read:** 12 §12.1, §12.6, §12.18; 02 §2.6.6; 04 §4.11.4; 09 §9.6
- **Deliverables:** VFX-01..VFX-22 shaders + manifests + goldens; masks MSK-01..03 with on-viewer editing; LUT import/copy/apply; LUT-01..05 generated by `tools/assets/generate_luts.py`.
- **Acceptance:** T-FX-004, T-FX-006, T-FX-007, T-FX-008.

### WP-16 — Transitions

- **Branch:** `feat/016-transitions` · **Depends on:** WP-14
- **Implements:** REQ-TRN-001..REQ-TRN-004
- **Read:** 12 §12.3; 03 §3.4.10; 07 PNL-03
- **Deliverables:** TRS-01..TRS-16, Transitions panel, edit-point drop with alignment, duration trimming, default transition commands.
- **Acceptance:** T-TRN-001..T-TRN-003.

### WP-17 — Titles and generators

- **Branch:** `feat/017-titles-generators` · **Depends on:** WP-14
- **Implements:** REQ-TTL-001..REQ-TTL-006, REQ-FX-009, REQ-I18N-007 (title fonts)
- **Read:** 02 ADR-015, §2.6.7; 03 §3.4.9; 12 §12.12, §12.16; 07 PNL-04
- **Deliverables:** text engine (cosmic-text layout, rasterization cache, GPU composition), title clips, TAN-01..10, TPL-01..12 with previews, user templates, generators GEN-01..05, title fonts FNT-04..08, on-viewer text editing.
- **Acceptance:** T-TTL-001..T-TTL-004, T-FX-009.

### WP-18 — Audio effects and mixing tools

- **Branch:** `feat/018-audio-tools` · **Depends on:** WP-10, WP-13
- **Implements:** REQ-AUD-002..REQ-AUD-006
- **Read:** 12 §12.2; 02 §2.7.3–2.7.6; 07 PNL-09, DLG-21..23; 09 §9.7
- **Deliverables:** AFX-01..AFX-11, Mixer panel, loudness analysis and normalization, auto-ducking, voice-over recording.
- **Acceptance:** T-AUD-002..T-AUD-006.

### WP-19 — Color tools and scopes

- **Branch:** `feat/019-color-scopes` · **Depends on:** WP-15
- **Implements:** REQ-COL-001, REQ-COL-002, REQ-COL-004, REQ-COL-005
- **Read:** 07 WS-02, PNL-10; 02 §2.6
- **Deliverables:** Color workspace controls (wheels, curves), GPU scopes with channel to UI, before/after split, copy/paste grade.
- **Acceptance:** T-COL-001, T-COL-003, T-COL-004.

### WP-20 — Captions

- **Branch:** `feat/020-captions` · **Depends on:** WP-13, WP-17
- **Implements:** REQ-CAP-001..REQ-CAP-007
- **Read:** 03 §3.4.10; 04 §4.11.1; 07 PNL-11, WS-04, §7.11; 12 §12.13 (E2011/E2012)
- **Deliverables:** caption track model ops, Captions panel, timeline caption lanes, import/export with encoding detection, styling, rendering (viewer + burn-in path shared with titles), readability warnings.
- **Acceptance:** T-CAP-001..T-CAP-007.

### WP-21 — Speech-to-text

- **Branch:** `feat/021-speech-to-text` · **Depends on:** WP-07, WP-20
- **Implements:** REQ-STT-001..REQ-STT-006
- **Read:** 06 §6.1–6.6; 12 §12.15; 05 SEC-10/11; 07 DLG-08, DLG-09, PNL-15
- **Deliverables:** `alvs-stt` (manifest generated from 12 §12.15 with `tools/models/hash-models.py`, downloads, whisper inference, segmentation), transcribe and model-manager dialogs, Transcript panel, speech fixtures with `SOURCES.md`.
- **Acceptance:** T-STT-001..T-STT-005, T-SEC-004 (transcription part), T-PERF-009.

### WP-22 — Export engine and render queue

- **Branch:** `feat/022-export` · **Depends on:** WP-09, WP-10, WP-07, WP-20
- **Implements:** REQ-EXP-001..REQ-EXP-012, REQ-CAP-005 (export), REQ-FX-011 (export), REQ-REL-007 (export)
- **Read:** 02 §2.9; 04 §4.3.8, §4.6.3; 12 §12.7, §12.19; 07 DLG-04
- **Deliverables:** export pipeline in workers, GPU RGB→YUV, encoder selection and hardware probing, hardware option mapping with tests, presets XP-01..XP-20, preflight, verification, chapters, captions modes, frame export, user presets, Export dialog and queue integration.
- **Acceptance:** T-EXP-001..T-EXP-009, T-PERF-007.

### WP-23 — Media processing and interchange

- **Branch:** `feat/023-processing-interop` · **Depends on:** WP-22
- **Implements:** REQ-MED-011, REQ-MED-013, REQ-PRJ-009, REQ-INT-001, REQ-INT-002
- **Read:** 04 §4.6.3 (optimize, stabilize, collect), §4.11.2–4.11.3; 03 §3.4.2 (`derived`)
- **Deliverables:** optimize and stabilize jobs with derived-media toggles, collect files, EDL export, chapter list and CSV export/import.
- **Acceptance:** T-MED-012, T-MED-013, T-INT-001..T-INT-003.

### WP-24 — Settings and customization

- **Branch:** `feat/024-settings-keymaps` · **Depends on:** WP-22
- **Implements:** REQ-SET-001, REQ-SET-002, REQ-SET-004, REQ-APP-003 (user workspaces)
- **Read:** 03 §3.6; 07 DLG-05, DLG-06; 12 §12.10
- **Deliverables:** Settings dialog generated from the schema (all keys wired to behavior), shortcut editor with conflict detection and import/export/print, cache management, saved workspaces.
- **Acceptance:** T-SET-001, T-SET-002, T-SET-004, T-APP-002.

### WP-25 — Localization completion and accessibility pass

- **Branch:** `i18n/025-localization-a11y` · **Depends on:** WP-24
- **Implements:** REQ-I18N-001, REQ-I18N-002, REQ-I18N-004 (completion), REQ-A11Y-001, REQ-A11Y-002 (audit fixes)
- **Read:** 07 §7.11–7.13; 09 §9.8; 12 §12.14
- **Deliverables:** 100% translations in five locales using the 07 §7.12 glossary (AGT-08), native-review notes for Arabic and French recorded in the PR, RTL visual fixes, Narrator/NVDA walkthrough fixes.
- **Acceptance:** T-I18N-001, T-I18N-002, T-I18N-004, T-A11Y-001, T-A11Y-003, T-A11Y-004.

### WP-26 — Help, onboarding, and About

- **Branch:** `feat/026-help-onboarding` · **Depends on:** WP-25
- **Implements:** REQ-HLP-001, REQ-HLP-002, REQ-HLP-004, REQ-I18N-006, REQ-LIC-001 (About), REQ-LIC-003
- **Read:** 07 §7.15, DLG-14, DLG-19, DLG-25; 05 §5.8
- **Deliverables:** first-run setup and tour; in-app help viewer; user guide topics in `assets/help/en/` (1 Getting started, 2 Projects and sequences, 3 Importing and organizing media, 4 Proxies and optimized media, 5 Viewers and playback, 6 Timeline basics, 7 Trimming tools, 8 Markers, 9 Effects and keyframes, 10 Masks and keying, 11 Transitions, 12 Titles and templates, 13 Color and scopes, 14 Audio mixing, 15 Voice-over and loudness, 16 Captions, 17 Speech-to-text, 18 Exporting and the render queue, 19 Keyboard shortcuts, 20 Settings, 21 Troubleshooting and recovery, 22 Privacy and updates) and translated short topics for the other locales; shortcut sheet; Report a Problem; About dialog with license, notices, source link, trademark notice.
- **Acceptance:** T-HLP-001, T-HLP-002, T-LIC-001.

### WP-27 — Reliability, diagnostics, and performance

- **Branch:** `feat/027-reliability` · **Depends on:** WP-22
- **Implements:** REQ-REL-001 (UX), REQ-REL-002, REQ-REL-004, REQ-REL-005, NFR-REL-01..NFR-REL-03, NFR-RES-01, NFR-RES-02, NFR-PERF-01, NFR-PERF-02
- **Read:** 02 ADR-022, §2.5, §2.15; 08 §8.12; 09 §9.13; 07 DLG-15, DLG-18
- **Deliverables:** crash server and handler, panic reports, safe mode, diagnostics bundle, local session counters, soak script, performance profiling fixes until budgets are met.
- **Acceptance:** T-REL-001, T-REL-002, T-REL-004, T-REL-005, T-REL-009, T-PERF-001, T-PERF-002, T-PERF-010, T-PERF-011.

### WP-28 — Packaging, updates, and release pipeline

- **Branch:** `ci/028-packaging-release` · **Depends on:** WP-27
- **Implements:** REQ-UPD-001..REQ-UPD-003, REQ-LIC-001 (installer, notices), REQ-LIC-002, NFR-INS-01, NFR-RES-03
- **Read:** 08 §8.3.3, §8.5–8.11; 05 §5.9; 02 ADR-019
- **Deliverables:** NSIS configuration (8.7), portable ZIP, updater with channels and rollback cache, `release.yml`, `publish-updates.yml`, notices/SBOM/source-archive tooling, `Set-Version.ps1`, git-cliff config, owner runbook for key generation; a full dry-run release `v0.9.0-beta.1`.
- **Acceptance:** T-UPD-001..T-UPD-004, T-LIC-002, T-PERF-012.

### WP-29 — Release candidate and 1.0.0

- **Branch:** `chore/029-release-v1-0-0` · **Depends on:** WP-23, WP-24, WP-25, WP-26, WP-27, WP-28
- **Implements:** NFR-UX-01, NFR-COMP-01, all release gates
- **Read:** 09 §9.11–9.14; 08 §8.3.3, §8.5, §8.9
- **Deliverables:** beta period (≥ 10 testers, 2 weeks), fixes for S1/S2 bugs, usability test (9.14), platform matrix, performance report, GATE evidence, CHANGELOG, version `1.0.0`, release PR; the owner tags and publishes.
- **Acceptance:** GATE-01..GATE-12; T-UX-001, T-QA-001, T-QA-003, T-E2E-001..T-E2E-006.

---

## 10.8 Dependency graph and lanes

```text
WP-00 → WP-01 ─┬─► WP-02 ─┬─► WP-03 ─┐
               │          ├─► WP-12 ─────────────────────────┐ (lane B)
               │          └─► WP-06 ─┬─► WP-07 ─┐            │
               └─► WP-04 ───────────►│          ▼            │
                              WP-05 ◄┘(needs 03,04) WP-08 ◄─(05,07)
                                     WP-06 ─► WP-09 ─┐       │
                                     WP-06 ─► WP-10 ─┼─► WP-11 (+05)
                                                     │       │
                              WP-13 ◄── (08, 11, 12) ┘◄──────┘
                              WP-14 ◄── (09, 13)
                    WP-15, WP-16, WP-17 ◄── WP-14
                    WP-18 ◄── (10, 13)      WP-19 ◄── WP-15
                    WP-20 ◄── (13, 17)      WP-21 ◄── (07, 20)
                    WP-22 ◄── (07, 09, 10, 20)
                    WP-23, WP-24, WP-27 ◄── WP-22
                    WP-25 ◄── WP-24 → WP-26 ;  WP-28 ◄── WP-27
                    WP-29 ◄── (23, 24, 25, 26, 27, 28)
```

Parallel lanes (at most two at a time, 06 §6.15): after WP-02, lane A runs WP-06 → WP-07 → WP-08 while lane B runs WP-12; after WP-14, WP-15/WP-16/WP-17 pair up; WP-18 can pair with WP-19.

---

## 10.9 Definition of Done (work package)

1. All acceptance tests of the card pass locally and in CI; gates of 08 §8.4 green.
2. Spec Kit `/speckit.converge` reports converged; `tasks.md` fully checked.
3. Docs updated where behavior/interfaces changed (G-19); `check_docs.py` green.
4. Strings localized in six locales (machine-assisted allowed before WP-25; WP-25 completes reviews).
5. STATUS.md updated; PR has evidence (G-09); reviewer findings resolved.
6. The owner merged the PR with a merge commit; branch deleted.

---

## 10.10 STATUS.md protocol

`STATUS.md` holds one row per work package: ID, title, state, branch, PR, last update (date), next step. Agents update their row on their work branch at each `/wp-finish`; the owner marks `done` after merge (or the next branch does). Blocked items state the question for the owner.

---

## 10.11 Execution risks (R)

| ID | Risk | Mitigation | Trigger |
|---|---|---|---|
| R-01 | WebView2 SharedBuffer integration harder or slower than planned | Spike first in WP-11; channel fallback; measure against NFR-PERF-03/04 | Spike misses 1080p60 |
| R-02 | vcpkg FFmpeg build fails or is slow on Windows | Pinned baseline, binary cache, documented troubleshooting in the WP-06 PR | Build > 90 min or failure |
| R-03 | Binding gaps in `ffmpeg-next` (hardware frames, options) | Use `ffmpeg-sys-next` FFI inside `alvs-media` with `// SAFETY:` notes and tests | Missing API |
| R-04 | GPU upload cost for 4K 10-bit | Plane formats per 02 §2.6.4, staging buffers, proxies by default | NFR-PERF-04 miss |
| R-05 | Agent drift or invented behavior | Spec Kit + constitution + reviewer + gates + small PRs + plan checkpoint | Reviewer finds invented scope |
| R-06 | Scope creep | G-02, roadmap in 01 §1.5, owner decisions | New feature requests during 1.0 |
| R-07 | Arabic shaping/bidi defects | Golden tests in `ar`, native review in WP-25 | Visual review failure |
| R-08 | A/V sync drift | Audio-clock master; sync tests from WP-10/11 | Drift > 1 frame |
| R-09 | Low-end performance | Proxies on by default for > 1440p, auto preview resolution, profiling in WP-27 | NFR-PERF-03 miss |
| R-10 | License compliance gaps | Allowlist, generated notices, source archive automation | Missing notice or source |
| R-11 | SmartScreen friction for unsigned installers | Checksums, attestations, winget, optional free signing (OPS-10) | User reports |
| R-12 | Owner availability for checkpoints and reviews | Batch reviews; per-WP waiver of the plan checkpoint (OD-08) | PRs waiting > 7 days |
| R-13 | Breaking changes in fast-moving dependencies or agent tools | Locked versions; upgrades only on `chore/deps-*`; read changelogs | Upgrade failures |
| R-14 | CI time for Windows builds | Caches, smoke subsets on PRs, full suites nightly | PR CI > 25 min |
| R-15 | Licensing of speech test fixtures | Only CC0/public-domain sources listed in `SOURCES.md` | Unknown provenance |
