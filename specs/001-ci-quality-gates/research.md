# Research: WP-01 — CI and quality gates

**Date**: 2026-11-02 · **Inputs**: WP-01 card (docs/10 §10.7); docs/01 NFR-MNT-01, NFR-MNT-02, NFR-SEC-01; docs/05 §5.6 (§5.7 SEC-10 for the HTTP ban); docs/08 §8.2–8.4; docs/09 §9.1, §9.10; docs/02 §2.4; docs/06 §6.13; repository scripts (read directly, G-10); GitHub API and tool repositories (queried 2026-11-02, cited per decision).

No NEEDS CLARIFICATION markers remain. Every external fact below was verified on 2026-11-02 (GitHub API releases + raw sources); no version, SHA, or schema is assumed (G-10).

## D1 — Verified external facts (the pin list)

- **Decision**: pin exactly these action versions/SHAs (latest stable as of 2026-11-02, resolved via `api.github.com` `releases/latest` + `commits/{tag}`):

| Action | Version | Commit SHA (pin) |
|---|---|---|
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/cache` | v6.1.0 | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` |
| `actions/upload-artifact` | v7.0.2 | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |
| `taiki-e/install-action` | v2.89.0 | `1919c4a95903d2c33e4a221102edca87309fc14b` |
| `lycheeverse/lychee-action` | v2.9.0 | `e7477775783ea5526144ba13e8db5eec57747ce8` |
| `pnpm/action-setup` | v6.1.0 | `ea17c68df8912ef543352723c149a84f56e3d413` |

- **Verified alongside**: `windows-2025` is a current GitHub-hosted runner image (actions/runner-images README, 2026-11-02); `taiki-e/install-action` supports the tool names `cargo-deny`, `cargo-nextest`, `cargo-llvm-cov` (per-tool manifests exist in its `tools/codegen/base/`); `cargo-llvm-cov` supports the `llvm-cov nextest` subcommand and `--lcov` (README); cargo-deny's own configuration demonstrates the `[licenses] allow` SPDX list and `[sources] unknown-registry/unknown-git = "deny"` schema (EmbarkStudios/cargo-deny `deny.toml`); Dependabot v2 schema supports `cargo`, `npm`, `github-actions` ecosystems, `weekly` interval, `directory` (schemastore `dependabot-2.0.json`).
- **Workstation (verified locally)**: `cargo-deny 0.20.2`, `cargo-nextest 0.9.150`, `cargo-llvm-cov` installed; `tools/setup/Test-DevEnvironment.ps1` line 73 checks exactly these three — so the local `dependency-policy` gate activates the moment `deny.toml` exists.
- **Alternatives considered**: floating version tags (`@v7`) — rejected: 05 §5.6 rule 5 and the card require full commit SHAs.

## D2 — Workflow skeleton: triggers, concurrency, permissions

- **Decision**: `ci.yml` triggers on `pull_request` (target `main`) and `push` (branch `main`); concurrency group `${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true`; top-level `permissions: contents: read`.
- **Rationale**: 08 §8.3.1 ("Triggers … Concurrency group per branch (cancel in progress)"); read-only default workflow permissions (08 §8.2 Actions row); `contents: read` is the least privilege checkout needs.
- **Alternatives considered**: no concurrency (wastes runner minutes against the 25-minute budget), broader permissions — rejected (08 §8.2; NFR-MNT-02).

## D3 — Job set and skeleton-applicable steps (the core scoping decision)

- **Decision**: all eight jobs of 08 §8.3.1 exist with these job ids (they are the OPS-02 required check names): `docs`, `commits`, `dco`, `rust-lint`, `rust-test`, `ui`, `build`, `e2e-smoke`. Per job, the steps that apply to the current skeleton run now; steps whose targets do not exist yet are not present (they arrive with their owning work packages). Matrix (now → deferred):

| Job | Runner | Steps now | Deferred (owning WP) |
|---|---|---|---|
| `docs` | ubuntu-latest | `python3 tools/docs/check_docs.py` | — |
| `commits` | ubuntu-latest | `python3 tools/git/check_commits.py origin/main..HEAD --only conventional` | — |
| `dco` | ubuntu-latest | `python3 tools/git/check_commits.py origin/main..HEAD --only dco` | — |
| `rust-lint` | windows-2025 | `rustup show` (toolchain from `rust-toolchain.toml`); `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; install `cargo-deny`; `cargo deny check`; `python tools/check_deps.py`; `python tools/check_placeholders.py` | bindings drift check (WP-02) |
| `rust-test` | windows-2025 | install `cargo-nextest` + `cargo-llvm-cov`, `rustup component add llvm-tools-preview`; `ALVS_FORCE_WARP=1 cargo llvm-cov nextest --workspace --locked --lcov --output-path lcov.info`; upload `lcov.info` artifact | vcpkg binary cache (WP-06); golden smoke set (later WPs) |
| `ui` | windows-2025 | pnpm setup (D8); `pnpm install --frozen-lockfile`; `pnpm -C app build` (the type-checking build the skeleton defines) + `python tools/check_npm_licenses.py` | `lint`, `test -- --coverage`, `i18n:check`, `test:harness` (WP-04/05) |
| `build` | windows-2025 | `cargo build -p afterleads-video-spirits --locked` (debug) | executable-start in `cli version` mode (CLI entry point, later WP) |
| `e2e-smoke` | windows-2025 | no-op step that prints the WP-05 replacement note and exits 0 (D9) | real WebdriverIO + tauri-driver smoke (WP-05) |

- **Rationale**: card ("jobs of 08 §8.3.1 that apply to the current skeleton; `e2e-smoke` added as a no-op job that reports success until WP-05"); 08 §8.2 lists all eight as the required status checks the owner enables after WP-01 merges — so the *names* must all exist now, while G-02 forbids building future targets early. `pnpm -C app build` runs `tsc && vite build` (verified: `app/package.json` scripts), which is the skeleton's type-correctness proof; the `ui`-job steps of 08 §8.3.1 whose scripts do not exist yet are not fabricated (G-08).
- **Alternatives considered**: creating empty `lint`/`test` npm scripts now to run more steps — rejected: inventing scripts is UI-tooling work owned by WP-04/05 (G-02).

## D4 — `commits` and `dco` jobs share `tools/git/check_commits.py` via a new `--only` flag

- **Decision**: extend the existing script (card: "exists; wire it") with an optional third argument `--only conventional|dco`; without the flag it behaves exactly as today (both checks) so hooks and `verify.ps1` keep calling `python tools/git/check_commits.py origin/main..HEAD` unchanged. The `commits` job runs `--only conventional`; the `dco` job runs `--only dco`.
- **Rationale**: the script already implements both rules (read in full: `CONVENTIONAL` regex per 10 §10.4.5 types; `SIGNOFF` match against author email, GitHub merge commits exempt) — one source of truth for local hooks, `verify.ps1`, and both CI jobs; the two OPS-02 check names need two job ids, not two rule sets.
- **Alternatives considered**: firstcontrib/dco-action + a separate conventional action — rejected: adds third-party actions to re-verify what a first-party tool already checks verbatim (constitution VIII simplicity); duplicating the logic in workflow bash — rejected (drift risk).

## D5 — Rust toolchain and tool installation on runners

- **Decision**: `actions/checkout` (pinned) then `rustup show` as the first Rust-job step — `rustup show` installs the toolchain selected by `rust-toolchain.toml` (1.99.1 + `rustfmt` + `clippy`) when absent. `cargo-deny`, `cargo-nextest`, and `cargo-llvm-cov` are installed via `taiki-e/install-action` (pinned, D1) with `tool: <name>`; the `rust-test` job adds `rustup component add llvm-tools-preview` (required by cargo-llvm-cov).
- **Rationale**: the toolchain pin is the canonical mechanism (00 §0.8; WP-00 D2); install-action fetches prebuilt release binaries in seconds (protecting the 25-minute budget, NFR-MNT-02) and supports exactly these tool names (D1); these are CI-environment tools, not workspace dependencies (G-11 untouched — no manifest changes, no ADR needed).
- **Alternatives considered**: `dtolnay/rust-toolchain` action — rejected: `rust-toolchain.toml` + built-in rustup needs no third-party action; `cargo install cargo-deny` — rejected: compiles from source for minutes on every cold run.

## D6 — Coverage wiring (the NFR-MNT-01 "gates" portion)

- **Decision**: `rust-test` runs `ALVS_FORCE_WARP=1 cargo llvm-cov nextest --workspace --locked --lcov --output-path lcov.info` and uploads `lcov.info` via `actions/upload-artifact` (pinned).
- **Rationale**: 08 §8.3.1 `rust-test` row ("coverage report (`cargo llvm-cov`) uploaded as artifact" with `ALVS_FORCE_WARP=1`); the card implements NFR-MNT-01's *gate infrastructure* — the measurement pipeline — while the ≥ 80%/≥ 60% thresholds bind in the crates' owning work packages (spec Assumptions). `llvm-cov nextest` subcommand and `--lcov` verified (D1).
- **Alternatives considered**: `cargo llvm-cov run` — rejected: 08 §8.3.1 names nextest as the test runner; Tarpaulin — rejected: not canonical on Windows.

## D7 — Caching (budget protection for NFR-MNT-02)

- **Decision**: `actions/cache` (pinned) with: `~/.cargo/registry`, `~/.cargo/git`, `target/` keyed by `hashFiles('Cargo.lock')` with restore-keys prefix fallback (rust-lint, rust-test, build); pnpm store keyed by `hashFiles('pnpm-lock.yaml')` (ui job). No vcpkg cache yet (no `vcpkg.json`).
- **Rationale**: 08 §8.3.1 caching paragraph names exactly these cache targets and keys; vcpkg binary caching waits for `vcpkg.json` (WP-06, card).
- **Alternatives considered**: `Swatinem/rust-cache` — rejected: 08 §8.3.1 names `actions/cache`; it is also one more third-party action to pin and review (VIII).

## D8 — pnpm on runners

- **Decision**: `pnpm/action-setup` (pinned, D1) with `version: 12.16.1` (matching the verified workstation version), using the runner image's Node; then `pnpm install --frozen-lockfile` at the workspace root.
- **Rationale**: 05 §5.6 rule 2 ("CI installs with … `--frozen-lockfile` (pnpm)"); pnpm 12.16.1 is the verified workstation/baseline version (WP-00 D6), so local and CI installs are identical.
- **Alternatives considered**: corepack with a `packageManager` field — rejected: would change `package.json` (root manifest is not a card deliverable) and adds no pinning benefit over the action's `version:` input.

## D9 — `e2e-smoke` no-op shape

- **Decision**: a `windows-2025` job with one step that states it is a no-op reporting success until WP-05 (card wording) and exits 0 — it *runs*, it is not skipped.
- **Rationale**: card ("no-op job that reports success until WP-05"); a `if:`-skipped job would not reliably satisfy the future required status check, and the card explicitly says "reports success". `windows-2025` keeps the job's contract identical to its 08 §8.3.1 row so WP-05 only replaces the step, not the job meta.
- **Alternatives considered**: `if: false` skip — rejected (see above); ubuntu runner — rejected: diverges from the canonical runner for this job.

## D10 — `nightly.yml`: two schedules, two jobs

- **Decision**: `on.schedule` with `17 2 * * *` (daily, dependency audit) and `27 2 * * 0` (weekly Sunday, docs links) plus `workflow_dispatch`; jobs: `dependency-audit` (`cargo deny check advisories` + `pnpm audit --prod`) and `docs-links` (`lycheeverse/lychee-action`, pinned, over `docs/**/*.md` links, `continue-on-error: true` as it is informational).
- **Rationale**: 08 §8.3.2 "Schedule daily (02:17 UTC) plus manual dispatch" with the docs-links job marked "(weekly, informational)"; the card scopes the file to "dependency audit + docs links now". Two cron entries are the plain mechanism for two cadences in one workflow; `continue-on-error` encodes "informational" (a broken external link pages the owner in logs, never blocks anything).
- **Alternatives considered**: two separate workflows — rejected: 08 §8.3.2 defines one `nightly.yml`; running the audit inside `ci.yml` — rejected: 08 §8.3.1 does not list it and it would slow every PR (NFR-MNT-02).

## D11 — `dependabot.yml`

- **Decision**: version 2; three update sets, all `schedule.interval: weekly` — `package-ecosystem: cargo` at `directory: "/"`; `package-ecosystem: npm` at `directory: "/app"`; `package-ecosystem: github-actions` at `directory: "/"`.
- **Rationale**: 05 §5.6 rule 5 ("Dependabot proposes updates weekly for cargo, npm, and actions"); the Cargo workspace root is `/` (Dependabot reads the workspace manifest there); npm's manifest root is `app/`; schema values verified (D1).
- **Alternatives considered**: `groups:` bundling — deferred (not named by the canonical docs; minimal config now, VIII).

## D12 — `deny.toml` content (the 05 §5.6 policy, machine-readable)

- **Decision**: `[licenses] allow` = exactly the 05 §5.6 allowlist SPDX ids (`MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `0BSD`, `Unicode-3.0`, `Unicode-DFS-2016`, `BSL-1.0`, `MPL-2.0`, `CC0-1.0`, `Unlicense`, `WTFPL`, `CDLA-Permissive-2.0`, `LGPL-2.1-or-later`, `LGPL-2.1-only`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later`, `AGPL-3.0-or-later`, `OFL-1.1`, `CC-BY-4.0`), `copyleft` unset (explicit list governs); `[advisories]` defaults (vulnerabilities deny); `[bans]` `deny` = HTTP-client crates `reqwest`, `ureq`, `isahc`, `surf`, `attohttpc`, `minreq`, `hyper`, `curl`, `curl-sys` (none are dependencies today, so the deny is fully restrictive), `multiple-versions = "warn"` (05 §5.6: duplicate major versions *reviewed*); `[sources]` `unknown-registry = "deny"`, `unknown-git = "deny"` (crates.io, the default registry, remains allowed).
- **Rationale**: card ("`deny.toml` (allowlist of 05 §5.6, bans: HTTP clients except allowed crates)"); 05 §5.6 rule 3 ("advisories, licenses, bans (duplicate major versions reviewed), and sources (crates.io only)"); the single sanctioned HTTP client is `ureq` 3 used only by `alvs-stt`'s download module (05 §5.7 SEC-10; ADR-023) — when WP-21 adds it, that deny entry gains the targeted allowance (`wrappers = ["alvs-stt"]`), which is the "except allowed crates" mechanism: nothing is pre-allowed before the canonical consumer exists.
- **Alternatives considered**: adding `ureq` to an allow list today — rejected: pre-allowing a dependency that does not exist yet weakens the ban for no present need (G-08 posture); omitting `hyper`/`curl` from the deny list — rejected: they are the transitive backbones of the banned clients (SEC-10 bans HTTP clients, not merely client facades).
- **Local parity (verified)**: `cargo-deny 0.20.2` is installed on the workstation (D1), so `cargo deny check` also runs as the local `dependency-policy` gate via `tools/verify.ps1` line 54.

## D13 — `tools/check_deps.py` design

- **Decision**: a no-argument checker that reads the workspace manifests and enforces docs/02 §2.4: (1) each `crates/alvs-*` member depends only on the crates of its CMP row and `app/src-tauri` only on CMP-01..CMP-10 (plus its own external allowance), no cycles; (2) exclusive-crate rules — `ffmpeg-next` only in `alvs-media`, `wgpu` only in `alvs-render`, `cpal` only in `alvs-audio`, `rusqlite` only in `alvs-store`, `whisper-rs` only in `alvs-stt`, `tauri`/`webview2-com` only in `app/src-tauri`; (3) UI code imports Rust types only through `app/src/ipc/bindings/` (checked as: no `../src-tauri` imports from `app/src` beyond that path — full enforcement lands with real UI imports, WP-04+); (4) `#![deny(missing_docs)]` present in `alvs-core`, `alvs-model`, `alvs-formats` `lib.rs`. Exit 0/1; findings printed one per line.
- **Rationale**: docs/02 §2.4 says verbatim "Rules (checked by `tools/check_deps.py` in CI, VER-09)" — all four rules; the card deliverable is the tool itself. Rule 4 is already satisfied on the tree (verified: attributes added in WP-00 review fix `50a2c6d`), so the checker passes today.
- **Alternatives considered**: cargo-deny bans for crate-exclusivity — rejected: cargo-deny cannot express "only crate X may depend on Y within this workspace"; hand review — rejected: VER-09 names the tool.

## D14 — `tools/check_placeholders.py` design

- **Decision**: a no-argument checker over first-party files (`crates/`, `app/src`, `app/src-tauri/`, `tools/`, `.githooks/`, `tests/` when present; excluding `app/src/ipc/bindings/` generated code) for the unfinished-work markers of docs/06 §6.13 rule 4: the placeholder macros `todo!()`/`unimplemented!()`, `TODO`/`FIXME` marker comments, commented-out code blocks, and fake production paths (stubs returning success without work — pattern-checked in obvious forms). The script builds marker strings by concatenation so it does not match itself.
- **Rationale**: 08 §8.4 "Placeholders — `python tools/check_placeholders.py`"; 06 §6.13 rule 4 (the no-placeholders rule, verbatim scope); the rust-lint step of 08 §8.3.1 runs this "placeholder grep".
- **Alternatives considered**: extending `check_docs.py` — rejected: its scope is documentation (VER-01..10), the canonical gate table lists a separate tool; ripgrep in bash — rejected: Windows runners + escaping make a first-party Python tool the maintainable form (and it must run locally identically).

## D15 — `tools/check_npm_licenses.py` design

- **Decision**: a no-argument checker that runs `pnpm -C app licenses list --json --prod` (requires the installed store, hence it runs in the `ui` job after `pnpm install --frozen-lockfile`), parses the license fields, and fails when any package's license is not on the 05 §5.6 allowlist. Not a `tools/verify.ps1` gate — the 08 §8.4 table has no such local row; it is the CI-side npm enforcer 05 §5.6 names.
- **Rationale**: 05 §5.6 ("enforced by `cargo deny check licenses` and `tools/check_npm_licenses.py`") — the two-sided policy: cargo-deny for the Rust graph, this tool for the pnpm graph; the asymmetry with `verify.ps1` is canonical (08 §8.4), documented in the spec (FR-018).
- **Alternatives considered**: parsing `pnpm-lock.yaml` statically — rejected: lockfiles carry no license fields; `license-checker` npm package — rejected: new dependency (G-11) when pnpm's built-in command provides the data.

## D16 — `tools/verify.ps1` is not edited

- **Decision**: no changes to `tools/verify.ps1`. Verified by reading it: `dependency-policy` activates when `deny.toml` exists and `cargo-deny` is installed (line 54), `crate-rules` when `tools/check_deps.py` exists (line 55), `placeholders` when `tools/check_placeholders.py` exists (line 56) — exactly the deliverables of this package.
- **Rationale**: the script's documented design ("Gates whose targets do not exist yet are reported as SKIPPED") means creating the targets *is* the wiring; WP-00 D7 already decided the script is not to be edited; G-08 (no weakening) is satisfied because nothing is skipped once targets exist.
- **Alternatives considered**: adding an npm-licenses gate row to `verify.ps1` — rejected: contradicts the canonical 08 §8.4 table (and the spec FR-018 now records that).

## D17 — Failure-demonstration procedure (card acceptance)

- **Decision**: after the real work is green, create one demonstration branch from `main` (e.g. `ci/001-demo-failures`) carrying four deliberately broken commits (formatting violation; clippy-violating code; an unfinished-marker; a commit lacking sign-off), open a **draft** PR so the `pull_request` trigger fires once, capture the failing job/step evidence (rust-lint red with the three failing steps; dco red), record the run links in `specs/001-ci-quality-gates/evidence.md` and the PR body, then close the draft PR. The demonstration branch is left for the owner to delete (G-18 — agents never delete branches).
- **Rationale**: card acceptance ("deliberately failing commits (format, lint, placeholder, missing sign-off) fail the right job (evidence in PR)"); one run covers all four demos because the four commits ride one branch; draft status plus closing keeps it out of the merge path (G-03/G-05 — the failures never reach `main`).
- **Alternatives considered**: four separate PRs — rejected: four times the noise for the same evidence; pushing failures to the work branch and reverting — rejected: pollutes the real history the `commits`/`dco` jobs then check (G-07).

## D18 — Baselines and the OPS-02 handoff

- **Decision**: T-SEC-001 baseline = the outputs of `cargo deny check` (advisories + licenses clean on the committed lockfiles) and `python tools/check_npm_licenses.py` (all allowlisted), recorded in `evidence.md`; T-QA-002 baseline = the wall-clock duration of a re-run of the WP-01 PR checks with warm caches (run URL + minutes), recorded the same way. The PR body carries the owner checklist for OPS-02 stage two: enable the eight required status checks (`docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke`).
- **Rationale**: card acceptance ("T-SEC-001, T-QA-002 baseline recorded") and deliverable ("owner checklist for OPS-02 in the PR body"); 08 §8.2 (the two-stage settings protocol — required checks are enabled *after* WP-01 merges precisely because the jobs must exist first); 09 §9.10 rows T-SEC-001 (I, dependency advisories and license policy, NFR-SEC-01) and T-QA-002 (I, PR CI duration, NFR-MNT-02).
- **Alternatives considered**: recording only pass/fail without the raw outputs — rejected: G-09 evidence standard (commands, results, IDs).
