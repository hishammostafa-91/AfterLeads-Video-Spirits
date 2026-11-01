# 08 — Operations

Canonical for: environments, GitHub repository settings, CI/CD pipelines, quality gates, versioning and release, packaging, updates, signing, observability and diagnostics, support, and runbooks (OPS).
Depends on: [05-security.md](05-security.md), [09-quality.md](09-quality.md), [10-execution.md](10-execution.md).

Every service used here is free for public open-source repositories (OD-05): GitHub (repository, Actions, Releases, Pages, Dependabot, secret scanning, CodeQL, private vulnerability reporting).

---

## 8.1 Environments

| Environment | Purpose | Notes |
|---|---|---|
| Developer workstation | Owner's Windows 11 PC running OpenCode | Setup in 10 §10.2; doubles as reference machine R-MID or R-LOW when it matches (09 §9.2) |
| CI Windows | GitHub-hosted `windows-2025` runners | No GPU: tests render on WARP |
| CI Linux | GitHub-hosted `ubuntu-latest` | Docs checks and fuzzing of platform-independent crates |
| Release build | GitHub-hosted `windows-2025` | Builds, signs updater artifacts, attests provenance |
| Clean-install test | Windows Sandbox (built into Windows 11 Pro/Enterprise) or a spare Windows 11 machine | Verifies the installer on a clean system (T-UPD-004) |

---

## 8.2 Repository settings (OPS-02)

The owner applies these settings once (WP-01 provides a checklist; agents may not change settings, G-18):

| Area | Setting |
|---|---|
| Default branch | `main` |
| Ruleset "Protect main" (target: `main`) | Restrict deletions; block force pushes; require a pull request before merging (required approvals **0** while there is one maintainer — the owner cannot approve their own PRs; raise to 1 when a second maintainer joins); dismiss stale approvals; require conversation resolution; require status checks to pass and branches to be up to date: `docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke`; bypass list empty |
| Ruleset "Protect release tags" (target: tags `v*`) | Restrict creations, updates, deletions to the owner |
| Merge methods | Allow merge commits only (title: pull request title); disable squash merging and rebase merging (G-05) |
| Branches | Automatically delete head branches after merge |
| Actions | Workflow permissions read-only by default; allow GitHub-authored actions and an explicit list of pinned third-party actions; require approval for first-time contributors' workflow runs |
| Security | Private vulnerability reporting on; Dependabot alerts and security updates on; secret scanning and push protection on; CodeQL default setup on |
| Pages | Source: GitHub Actions (documentation site and update manifests, OPS-09) |
| Issues | Templates from `.github/ISSUE_TEMPLATE/`; labels in 8.12 |

---

## 8.3 Pipelines

### 8.3.1 Pull-request CI (OPS-03) — `.github/workflows/ci.yml`

Triggers: `pull_request` to `main`, `push` to `main`. Concurrency group per branch (cancel in progress). Total budget ≤ 25 minutes with warm caches (NFR-MNT-02).

| Job (required check name) | Runner | Steps |
|---|---|---|
| `docs` | ubuntu-latest | `python tools/docs/check_docs.py` (VER-01..VER-10) |
| `commits` | ubuntu-latest | Every commit message follows Conventional Commits (10 §10.4.5) |
| `dco` | ubuntu-latest | Every commit has a `Signed-off-by:` trailer matching the author email |
| `rust-lint` | windows-2025 | Toolchain from `rust-toolchain.toml`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo deny check`; `python tools/check_deps.py`; bindings drift check; placeholder grep (06 §6.13) |
| `rust-test` | windows-2025 | Restore vcpkg binary cache; `cargo nextest run --workspace --locked` with `ALVS_FORCE_WARP=1`; golden smoke set; coverage report (`cargo llvm-cov`) uploaded as artifact |
| `ui` | windows-2025 | `pnpm install --frozen-lockfile`; `pnpm -C app typecheck`; `pnpm -C app lint` (ESLint + Stylelint + Prettier check); `pnpm -C app test -- --coverage`; `pnpm -C app i18n:check`; `pnpm -C app test:harness` (Playwright: visual, axe, RTL) |
| `build` | windows-2025 | `cargo build -p afterleads-video-spirits --locked` (debug); verifies the executable starts in `cli version` mode |
| `e2e-smoke` | windows-2025 | Only when `app/**` or `crates/**` changed: release build without bundle, WebdriverIO + tauri-driver smoke suite (T-E2E-001) |

vcpkg caching: binary cache directory `${{ runner.temp }}/vcpkg-bincache` restored and saved with `actions/cache`, key = hash of `vcpkg.json`, `vcpkg-configuration.json`, and the triplet; environment `VCPKG_BINARY_SOURCES=clear;files,<dir>,readwrite`. Rust caches: `~/.cargo/registry`, `~/.cargo/git`, `target/` keyed by `Cargo.lock`. pnpm store cache keyed by `pnpm-lock.yaml`.

### 8.3.2 Nightly CI (OPS-05) — `.github/workflows/nightly.yml`

Schedule daily (02:17 UTC) plus manual dispatch.

| Job | Purpose |
|---|---|
| Full test suite | All Rust tests including the full golden matrix (09 §9.6) |
| Full E2E | WebdriverIO suite T-E2E-001..006 |
| Fuzz | `cargo fuzz` targets (SEC-05), 10 minutes each, on ubuntu-latest; crashes uploaded as artifacts and turned into issues by the owner |
| Benchmarks | Criterion benchmarks on WARP (trend only; gating happens on reference machines) |
| Dependency audit | `cargo deny check advisories`, `pnpm audit --prod` |
| Docs links | External link check (weekly, informational) |

### 8.3.3 Release pipeline (OPS-06) — `.github/workflows/release.yml`

Trigger: push of a tag `v*.*.*` (created by the owner, OPS-08). Steps:

1. Verify the tag points to a merge commit on `main` and matches the version in `Cargo.toml`, `app/package.json`, and `tauri.conf.json`.
2. Build FFmpeg (vcpkg, cache allowed) and the app: `pnpm tauri build` producing the NSIS installer; updater artifacts signed with `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets.
3. Create the portable ZIP (`AfterLeads-Video-Spirits_<version>_x64_portable.zip`) containing the executable, resources, `portable.flag`, LICENSE, and notices.
4. Generate: SBOMs (`cargo cyclonedx`, `@cyclonedx/cyclonedx-npm`), third-party notices (`cargo about generate` + `tools/release/npm_notices.py` + `assets/notices/static.md` for FFmpeg, codec libraries, fonts, icons, models), the third-party source archive (`tools/release/collect_vcpkg_sources.ps1` zips the exact source archives vcpkg downloaded), SHA-256 checksums (`SHA256SUMS.txt`).
5. Attest build provenance for every artifact (`actions/attest-build-provenance`).
6. Create a **draft** GitHub Release with notes generated by git-cliff and all artifacts plus `latest.json`.
7. The owner reviews and publishes the draft. Publishing triggers `.github/workflows/publish-updates.yml`, which deploys the update manifest (stable or beta) and the docs site to GitHub Pages.

---

## 8.4 Quality gates (local and CI)

`tools/verify.ps1` runs the same gates locally (G-08); each step is skipped with a notice when its target does not exist yet (early work packages).

| Gate | Command | Blocks merge |
|---|---|---|
| Docs consistency | `python tools/docs/check_docs.py` | yes |
| Rust format | `cargo fmt --all -- --check` | yes |
| Rust lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` | yes |
| Dependency policy | `cargo deny check`; `python tools/check_deps.py` | yes |
| Rust tests | `cargo nextest run --workspace --locked` | yes |
| UI typecheck/lint/format | `pnpm -C app typecheck`; `pnpm -C app lint` | yes |
| UI tests | `pnpm -C app test` | yes |
| Localization | `pnpm -C app i18n:check` (keys identical in 6 locales, ICU syntax valid, no unused keys) | yes |
| Bindings drift | `cargo test -p alvs-model export_bindings` then `git diff --exit-code app/src/ipc/bindings` | yes |
| Placeholders | `python tools/check_placeholders.py` | yes |
| Commit format and sign-off | `python tools/git/check_commits.py origin/main..HEAD` | yes |

---

## 8.5 Versioning and changelog (OPS-07)

- Semantic Versioning `MAJOR.MINOR.PATCH`; pre-releases `X.Y.Z-beta.N` go to the beta channel.
- The version lives in `Cargo.toml` (workspace package), `app/package.json`, and `app/src-tauri/tauri.conf.json`; `tools/release/Set-Version.ps1 -Version X.Y.Z` updates all three; CI checks they match.
- `CHANGELOG.md` is generated with git-cliff (`cliff.toml` groups Conventional Commit types) in a `chore/release-vX.Y.Z` pull request before tagging.
- Project file `schemaVersion` changes are listed under "Breaking/Data" in the changelog.

## 8.6 Identity constant locations

The repository URL and identity constants (00 §0.5) appear only in: `crates/alvs-core/src/identity.rs`, `app/src/config/identity.ts`, `app/src-tauri/tauri.conf.json` (identifier, product name, updater endpoints), `README.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `TRADEMARKS.md`, `docs/00-index.md`. VER-07 checks consistency.

---

## 8.7 Packaging

| Item | Setting |
|---|---|
| Bundler target | NSIS (`bundle.targets = ["nsis"]`), `installMode: "currentUser"` (no admin) |
| Installer languages | English, Arabic, French, Spanish, German, PortugueseBR (NSIS language selector) |
| Executable name | `AfterLeads-Video-Spirits.exe` (configure Tauri `mainBinaryName`; verified in WP-28) |
| File association | `.alvs` → "AfterLeads-Video-Spirits Project", icon `assets/icons/project.ico` |
| WebView2 | Default bootstrapper behavior (downloads only if missing; Windows 11 includes it) |
| Resources | `assets/fonts`, `assets/help`, `assets/luts`, `assets/templates`, `assets/models/whisper.json`, `assets/notices`, LICENSE |
| Uninstaller | Keeps user data unless "Delete the application data" is ticked (NFR-INS-01) |
| Portable ZIP | Same files + `portable.flag`; updates disabled (REQ-UPD-003) |

## 8.8 Updates (OPS-09)

- Plugin: tauri-plugin-updater with the public key in `tauri.conf.json`.
- Endpoints: `https://hishammostafa-91.github.io/AfterLeads-Video-Spirits/updates/stable.json` and `…/updates/beta.json` (selected by `alvs.general.updateChannel`).
- Manifest content (generated by the release pipeline): version, notes URL, publication date, and for platform `windows-x86_64` the NSIS installer URL on GitHub Releases with its Ed25519 signature.
- Key generation (once, by the owner on the development machine): `pnpm tauri signer generate -w "$env:USERPROFILE\.alvs-keys\updater.key"`; store the private key and password as repository secrets; keep an encrypted offline backup; never commit either (SEC-15).
- Before installing, the app saves the current installer path for rollback (REQ-UPD-002) in `updates\` (03 §3.8).

## 8.9 Tag signing (OPS-08)

The owner signs release tags with an SSH key: `git config --global gpg.format ssh`, `git config --global user.signingkey <path-to-public-key>`, then on `main` after the release-prep merge: `git tag -s vX.Y.Z -m "AfterLeads-Video-Spirits X.Y.Z"` and `git push origin vX.Y.Z`. Agents never create or push release tags (G-18).

## 8.10 Optional Windows code signing (OPS-10)

Default (OD-06): installers are not Authenticode-signed; users verify `SHA256SUMS.txt` and GitHub attestations. When ready, the owner applies to the SignPath Foundation program for open-source projects (free); if accepted: publish the required code-signing policy page, add the SignPath GitHub Action to the release workflow after the build step, and note that the certificate publisher is SignPath Foundation. No paid certificate is required.

## 8.11 winget publication (OPS-11)

After each stable release the owner (or an agent instructed by the owner) creates a manifest update for `AfterLeads.AfterLeads-Video-Spirits` with Microsoft's free `wingetcreate` tool and submits it as a pull request to the community winget repository.

---

## 8.12 Observability, diagnostics, and support

- **Logs:** 03 §3.8; structured JSON lines; log level from settings.
- **Crash data:** panic reports and minidumps (ADR-022) in `crashes\`; the next start shows "The app closed unexpectedly — Report a problem?" with REQ-HLP-002 flow.
- **Local session counters:** sessions started, clean exits, abnormal exits, GPU device losses, worker crashes — shown in Help › Create Diagnostics Bundle (and in the bundle). Beta testers report them manually for NFR-REL-01.
- **Issue templates:** Bug report (version, steps, expected/actual, diagnostics bundle optional), Feature request (problem, proposal, which REQ it touches), Translation (locale, keys), Security (redirects to SECURITY.md).
- **Labels:** `bug`, `enhancement`, `translation`, `documentation`, `good first issue`, `help wanted`, `needs-info`, `performance`, `accessibility`, `security`, `wp:<NN>`.
- **Triage:** see OPS-16 in 8.13.

---

## 8.13 Runbooks

| ID | Situation | Steps |
|---|---|---|
| OPS-12 | Security incident (vulnerability or key compromise) | 1 Confirm and assess (THR mapping). 2 For key compromise: delete affected releases/manifests, rotate the updater key (05 §5.9), revoke tokens. 3 Fix on a `fix/` branch, release a patch. 4 Publish a GitHub Security Advisory. 5 Post-mortem in `docs/` if process changes. |
| OPS-13 | Hotfix | Branch `fix/0NN-<slug>` from `main` (G-04), minimal fix + regression test, PR, merge, release-prep PR, tag `vX.Y.(Z+1)`. |
| OPS-14 | Bad release | Unpublish the update manifest (re-run publish-updates for the previous version), mark the release as pre-release with a warning, publish a fixed patch version; the updater never downgrades automatically — users can roll back via About › previous version (REQ-UPD-002). |
| OPS-15 | Repository backup | Monthly `git clone --mirror` of the repository to an offline disk; keep the updater key backup separately. |
| OPS-17 | CI outage or quota issue | Run `tools/verify.ps1` locally and attach the output to the PR; merge only after CI recovers (required checks stay mandatory). |
| OPS-18 | Dependency advisory | Dependabot PR or manual `chore/deps-<name>` branch; run full gates; release within SEC-12 deadlines. |
| OPS-16 | Issue triage | Weekly: label new issues, ask for missing information (`needs-info`), link them to REQ/WP IDs, close duplicates; acknowledge within 7 days; security reports follow 05 §5.10. |
| OPS-19 | User reports a corrupted project | Ask for the project file and diagnostics bundle; run `cli validate`; restore from `Backups\` or the autosave; add a regression fixture. |

---

## 8.14 Code review checklist (OPS-04)

The owner (and the `reviewer` agent, AGT-05) checks every pull request against this list before merging:

1. Scope: only the REQ IDs of the card/issue; no unrelated changes; docs updated in the same PR when behavior changed (G-19).
2. Branch: created from `main`, named per 10 §10.4.2, up to date with `main`, merge commit method (G-03..G-05).
3. Commits: Conventional Commits, DCO sign-off, no secrets, no generated files edited by hand.
4. Tests: new behavior has tests with the T-IDs from the card; no skipped or weakened tests; evidence in the PR body (G-09).
5. Security: inputs validated, paths through SEC-04, no new network access, no new runtime code loading; THR/SEC IDs listed when relevant.
6. Licensing: SPDX headers present; new dependencies have an approved ADR and allowed license.
7. UX: strings localized in all six locales, RTL screenshots for UI changes, keyboard access, focus visible, contrast tokens used.
8. Performance: hot paths keep their budgets; no per-frame allocations; benchmarks updated.
9. Code quality: no placeholders (06 §6.13), errors mapped to E-codes, logging without personal data.
