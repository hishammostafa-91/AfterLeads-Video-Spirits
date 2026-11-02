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
| `commits` | ubuntu-latest | checkout with `fetch-depth: 0` (the range needs full history); `python3 tools/git/check_commits.py origin/main..HEAD --only conventional` | — |
| `dco` | ubuntu-latest | checkout with `fetch-depth: 0`; `python3 tools/git/check_commits.py origin/main..HEAD --only dco` | — |
| `rust-lint` | windows-2025 | `rustup show` (toolchain from `rust-toolchain.toml`); `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; install `cargo-deny`; `cargo deny check`; `python tools/check_deps.py`; `python tools/check_placeholders.py` | bindings drift check (WP-02) |
| `rust-test` | windows-2025 | `rustup show`; install `cargo-nextest` + `cargo-llvm-cov`; `rustup component add llvm-tools-preview`; `cargo llvm-cov nextest --workspace --locked --no-tests=pass --lcov --output-path lcov.info` (env `ALVS_FORCE_WARP: 1`); upload `lcov.info` | vcpkg binary cache (WP-06); golden smoke set (later WPs) |
| `ui` | windows-2025 | pnpm setup (D8); `pnpm install --frozen-lockfile`; `pnpm -C app build` (the type-checking build the skeleton defines) + `python tools/check_npm_licenses.py` | `lint`, `test -- --coverage`, `i18n:check`, `test:harness` (WP-04/05) |
| `build` | windows-2025 | `cargo build -p afterleads-video-spirits --locked` (debug) | executable-start in `cli version` mode (CLI entry point, later WP) |
| `e2e-smoke` | windows-2025 | no-op step that prints the WP-05 replacement note and exits 0 (D9) | real WebdriverIO + tauri-driver smoke (WP-05) |

- **Rationale**: card ("jobs of 08 §8.3.1 that apply to the current skeleton; `e2e-smoke` added as a no-op job that reports success until WP-05"); 08 §8.2 lists all eight as the required status checks the owner enables after WP-01 merges — so the *names* must all exist now, while G-02 forbids building future targets early. `pnpm -C app build` runs `tsc && vite build` (verified: `app/package.json` scripts), which is the skeleton's type-correctness proof; the `ui`-job steps of 08 §8.3.1 whose scripts do not exist yet are not fabricated (G-08). The `commits`/`dco` jobs check out with `fetch-depth: 0` because `pull_request` events otherwise fetch only the merge ref (depth 1) and `origin/main..HEAD` is unresolvable — the script would exit 2 and redden both jobs on every PR. The WARP variable is passed as step-level `env:` because the `VAR=1 cmd` prefix form is invalid in pwsh, the Windows runners' default shell. The `rust-lint` gate steps run with `if: always()` so every gate reports independently in one run — a developer (and the WP-01 failure-demonstration evidence, D17) sees all failing gates, not only the first.
- **Alternatives considered**: creating empty `lint`/`test` npm scripts now to run more steps — rejected: inventing scripts is UI-tooling work owned by WP-04/05 (G-02).

## D4 — `commits` and `dco` jobs share `tools/git/check_commits.py` via a new `--only` flag

- **Decision**: extend the existing script (card: "exists; wire it") with an optional third argument `--only conventional|dco`; without the flag it behaves exactly as today (both checks) so its existing caller — `tools/verify.ps1`, line 51 — keeps calling `python tools/git/check_commits.py origin/main..HEAD` unchanged. The `commits` job runs `--only conventional`; the `dco` job runs `--only dco`.
- **Rationale**: the script already implements both rules (read in full: `CONVENTIONAL` regex per 10 §10.4.5 types; `SIGNOFF` match against author email, GitHub merge commits exempt) — one source of truth for `tools/verify.ps1` (its only caller today, line 51) and both CI jobs; the two OPS-02 check names need two job ids, not two rule sets. Repo state corrected during review: `.githooks/commit-msg` implements its own bash copy of the same rules and never called the Python tool — the pre-existing duplication is noted (consolidating the hook onto the tool is out of scope: it would touch `.githooks/`, which WP-01 does not modify). **Bug found and fixed by the failure demonstration (T023, PR #2 run 41500649548)**: the GitHub `pull_request` test-merge commit ("Merge \<sha> into \<sha>") was not exempted, so the `commits` job failed on generated machinery; the canonical rule is "merge commits allowed" (10 §10.4.13) — the tool now exempts all merge commits from the conventional check and both GitHub-generated merge forms from both checks, verified locally against a plumbing-built replica of the exact failing commit.
- **Alternatives considered**: firstcontrib/dco-action + a separate conventional action — rejected: adds third-party actions to re-verify what a first-party tool already checks verbatim (constitution VIII simplicity); duplicating the logic in workflow bash — rejected (drift risk).

## D5 — Rust toolchain and tool installation on runners

- **Decision**: `actions/checkout` (pinned) then `rustup show` as the first Rust-job step — `rustup show` installs the toolchain selected by `rust-toolchain.toml` (1.99.1 + `rustfmt` + `clippy`) when absent. `cargo-deny`, `cargo-nextest`, and `cargo-llvm-cov` are installed via `taiki-e/install-action` (pinned, D1) with `tool: <name>`; the `rust-test` job adds `rustup component add llvm-tools-preview` (required by cargo-llvm-cov).
- **Rationale**: the toolchain pin is the canonical mechanism (00 §0.8; WP-00 D2); install-action fetches prebuilt release binaries in seconds (protecting the 25-minute budget, NFR-MNT-02) and supports exactly these tool names (D1); these are CI-environment tools, not workspace dependencies (G-11 untouched — no manifest changes, no ADR needed).
- **Alternatives considered**: `dtolnay/rust-toolchain` action — rejected: `rust-toolchain.toml` + built-in rustup needs no third-party action; `cargo install cargo-deny` — rejected: compiles from source for minutes on every cold run.

## D6 — Coverage wiring (the NFR-MNT-01 "gates" portion)

- **Decision**: `rust-test` runs `cargo llvm-cov nextest --workspace --locked --no-tests=pass --lcov --output-path lcov.info` with step-level `env: ALVS_FORCE_WARP: 1` (D3) and uploads `lcov.info` via `actions/upload-artifact` (pinned).
- **Rationale**: 08 §8.3.1 `rust-test` row ("coverage report (`cargo llvm-cov`) uploaded as artifact" with `ALVS_FORCE_WARP=1`); the card implements NFR-MNT-01's *gate infrastructure* — the measurement pipeline — while the ≥ 80%/≥ 60% thresholds bind in the crates' owning work packages (spec Assumptions). `llvm-cov nextest` subcommand and `--lcov` verified (D1).
- **Zero-test baseline (explicit)**: the workspace currently has **zero tests**. Verified at implementation (T009, 2026-11-02): plain `cargo llvm-cov nextest` exits **4** here — nextest errors with "no tests to run (hint: use `--no-tests` to customize)" and no lcov is written. The invocation therefore carries `--no-tests=pass`: exit 0, `0 passed`, and a valid lcov (which already carries line data for built code such as the app crate's `run`). This is the sanctioned "adjust the invocation, never the gate" case — the gate is *tests pass and coverage is measured*, and `--no-tests=pass` keeps exactly that semantics when a workspace legitimately has no tests yet; real failures still fail, and the flag stays correct once tests exist.
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

## D10 — `nightly.yml`: two schedules with per-job guards, two fully set-up jobs

- **Decision**: `on.schedule` with `17 2 * * *` (daily) and `27 2 * * 0` (weekly Sunday) plus `workflow_dispatch`; per-job guards select the cadence: `dependency-audit` runs `if: github.event_name == 'workflow_dispatch' || github.event.schedule == '17 2 * * *'`; `docs-links` runs `if: github.event_name == 'workflow_dispatch' || github.event.schedule == '27 2 * * 0'` (on `schedule` events `github.event.schedule` holds the cron entry that fired, so exactly one job runs per tick; dispatch runs both). The guards are required — without them both jobs would run on every tick and docs-links would be daily, contradicting 08 §8.3.2. The second schedule entry is a small canonical-doc deviation and is documented in docs/08 §8.3.2 in this work package (G-19; owner-approved with the F3 decision, 2026-11-02).
- **Job setup (both jobs are self-contained, like every `ci.yml` job)**:
  - `dependency-audit` — runner `windows-2025` (keeps the Rust toolchain context of the other cargo jobs); steps: `actions/checkout` (pinned); `rustup show` (toolchain; `cargo-deny` shells out to `cargo metadata`); install `cargo-deny` via `taiki-e/install-action` (pinned); `cargo deny check advisories`; `pnpm/action-setup` (pinned, 12.16.1); `pnpm install --frozen-lockfile`; `pnpm audit --prod`.
  - `docs-links` — runner `ubuntu-latest`; steps: `actions/checkout` (pinned); `lycheeverse/lychee-action` (pinned) over `docs/**/*.md`; `continue-on-error: true` on the job (informational, 08 §8.3.2 — it pages the owner in logs, never blocks anything).
- **Rationale**: 08 §8.3.2 "Schedule daily (02:17 UTC) plus manual dispatch" with the docs-links job marked "(weekly, informational)"; the card scopes the file to "dependency audit + docs links now". Two cron entries with schedule-string guards are the plain GitHub Actions mechanism for two cadences in one workflow; the G-19 note keeps the canonical trigger line truthful.
- **Alternatives considered**: two separate workflows — rejected: 08 §8.3.2 defines one `nightly.yml`; single daily cron with an in-job day-of-week check — rejected: less idiomatic and hides the weekly cadence from the workflow declaration; running the audit inside `ci.yml` — rejected: 08 §8.3.1 does not list it and it would slow every PR (NFR-MNT-02).

## D11 — `dependabot.yml`

- **Decision**: version 2; three update sets, all `schedule.interval: weekly` — `package-ecosystem: cargo` at `directory: "/"`; `package-ecosystem: npm` at `directory: "/app"`; `package-ecosystem: github-actions` at `directory: "/"`.
- **Rationale**: 05 §5.6 rule 5 ("Dependabot proposes updates weekly for cargo, npm, and actions"); the Cargo workspace root is `/` (Dependabot reads the workspace manifest there); npm's manifest root is `app/`; schema values verified (D1).
- **Alternatives considered**: `groups:` bundling — deferred (not named by the canonical docs; minimal config now, VIII).

## D12 — `deny.toml` content (the 05 §5.6 policy, machine-readable)

- **Decision**: `[licenses] allow` = exactly the 05 §5.6 allowlist SPDX ids (`MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `0BSD`, `Unicode-3.0`, `Unicode-DFS-2016`, `BSL-1.0`, `MPL-2.0`, `CC0-1.0`, `Unlicense`, `WTFPL`, `CDLA-Permissive-2.0`, `LGPL-2.1-or-later`, `LGPL-2.1-only`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later`, `AGPL-3.0-or-later`, `OFL-1.1`, `CC-BY-4.0`), `copyleft` unset (explicit list governs); `[advisories]` defaults (vulnerabilities deny); `[bans]` `multiple-versions = "warn"` (05 §5.6: duplicate major versions *reviewed*); `[sources]` `unknown-registry = "deny"`, `unknown-git = "deny"` (crates.io, the default registry, remains allowed).
- **HTTP-client ban (owner decision F1, 2026-11-02 — tauri-wrapped allowance)**: `[bans] deny` lists the HTTP-client crates with cargo-deny *wrapper* allowances for the one sanctioned chain in the current tree: `reqwest` with `wrappers = ["tauri"]`, and `hyper` with `wrappers = ["reqwest", "hyper-util"]` — cargo-deny checks the wrapper exemption **per direct-parent edge** (verified against cargo-deny `src/bans.rs` and docs), and `hyper`'s direct parents in `Cargo.lock` are exactly `reqwest` (line 2499) and `hyper-util` (line 1332; itself reachable only via `reqwest`, line 2500), while `tauri` 2.12.5 depends on `reqwest` directly (line 3099). Every edge of the chain is therefore explicitly allowed and any *other* route to these crates stays denied. The remaining HTTP-client crates are denied unconditionally: `ureq`, `isahc`, `surf`, `attohttpc`, `minreq`, `curl`, `curl-sys`. When WP-21 adds the model-download client, `ureq` gains `wrappers = ["alvs-stt"]` — the same mechanism.
- **Rationale**: card ("`deny.toml` (allowlist of 05 §5.6, bans: HTTP clients except allowed crates)"); 05 §5.6 rule 3; SEC-10 bans HTTP clients in "*no other crate*" than the sanctioned modules — and ADR-023 states "the updater plugin uses its own client": the app shell `tauri` (CMP-12, the sanctioned manifest dependency in `app/src-tauri/Cargo.toml` line 21) *is* that sanctioned consumer. The wrapper encoding is strictly tighter than a global allow: if a future tauri version stops depending on `reqwest` directly, the check fails and forces a conscious re-decision instead of silently staying open.
- **Correction record (reviewer F1, CRITICAL)**: the first draft claimed "none are dependencies today, so the deny is fully restrictive" — false: `reqwest 0.13.5` (line 2487) and `hyper 1.12.0` (line 1301) are in `Cargo.lock` via `tauri`. Root cause: the lockfile was not consulted (G-10 breach); this decision now cites the lockfile directly. The re-review then caught a second missing edge (`hyper-util → hyper`) — the wrapper set above now covers every direct-parent edge in the lockfile. The wrapper semantics against cargo-deny 0.20.2 are re-verified at implementation (locally installed) before the first CI run.
- **Target scoping (verified at implementation, T010)**: the policy graph is scoped with `[graph] targets = [{ triple = "x86_64-pc-windows-msvc" }]` — cargo-deny otherwise resolves *all* targets' graphs, and the first run failed advisories on RUSTSEC-2024-0370 (`proc-macro-error` unmaintained, no safe upgrade) reachable only through tauri's Linux-only GUI chain (`muda → gtk → glib → glib-macros`), which the Windows-only product (constitution technology constraints) never builds. Scoping to the shipped target keeps the advisory check strict for everything actually shipped; it is a target-scope decision, not an advisory ignore.
- **Local parity (verified)**: `cargo-deny 0.20.2` is installed on the workstation (D1), so `cargo deny check` also runs as the local `dependency-policy` gate via `tools/verify.ps1` line 54.

## D13 — `tools/check_deps.py` design

- **Decision**: a no-argument checker that reads the workspace manifests and enforces docs/02 §2.4: (1) each `crates/alvs-*` member depends only on the crates of its CMP row and `app/src-tauri` only on CMP-01..CMP-10 (plus its own external allowance), no cycles; (2) exclusive-crate rules — `ffmpeg-next` only in `alvs-media`, `wgpu` only in `alvs-render`, `cpal` only in `alvs-audio`, `rusqlite` only in `alvs-store`, `whisper-rs` only in `alvs-stt`, `tauri`/`webview2-com` only in `app/src-tauri`; (3) UI code imports Rust types only through `app/src/ipc/bindings/` (checked as: no `../src-tauri` imports from `app/src` beyond that path — full enforcement lands with real UI imports, WP-04+); (4) `#![deny(missing_docs)]` present in `alvs-core`, `alvs-model`, `alvs-formats` `lib.rs`. Exit 0/1; findings printed one per line. *Review fix (F1, 2026-11-02)*: the app crate is checked against CMP-12 and CMP-11 like every crate; exclusivity and the testkit ban cover all dependency sections (regular, dev, build, target-specific); row checks cover the build graph.
- **Rationale**: docs/02 §2.4 says verbatim "Rules (checked by `tools/check_deps.py` in CI, VER-09)" — all four rules; the card deliverable is the tool itself. Rule 4 is already satisfied on the tree (verified: attributes added in WP-00 review fix `50a2c6d`), so the checker passes today.
- **Alternatives considered**: cargo-deny bans for crate-exclusivity — rejected: cargo-deny cannot express "only crate X may depend on Y within this workspace"; hand review — rejected: VER-09 names the tool.

## D14 — `tools/check_placeholders.py` design

- **Decision**: a no-argument checker over first-party files (`crates/`, `app/src`, `app/src-tauri/`, `tools/`, `.githooks/`, `tests/` when present; excluding `app/src/ipc/bindings/` generated code) for the unfinished-work markers of docs/06 §6.13 rule 4: the placeholder macros `todo!()`/`unimplemented!()`, `TODO`/`FIXME` marker comments, **mocked production paths**, and commented-out code. Per the same rule's explicit exemption, **test code and the mock backend are exempt** — implemented structurally: `tests/` trees, files matching test patterns (`*_test.rs`, `*.test.ts(x)`), `#[cfg(test)]` regions, and the mock-backend module (when it exists, WP-04/05) are out of scan scope. The script builds marker strings by concatenation so it does not match itself.
- **Rationale**: 08 §8.4 "Placeholders — `python tools/check_placeholders.py`"; 06 §6.13 rule 4 verbatim ("`todo!()`, `unimplemented!()`, `TODO`/`FIXME` comments, mocked production paths, or commented-out code are not allowed on `main` (CI greps for them; test code and the mock backend are exempt)"); the rust-lint step of 08 §8.3.1 runs this "placeholder grep". Without the exemption the tool would be stricter than the rule it enforces from WP-02 on (reviewer F8).
- **Alternatives considered**: extending `check_docs.py` — rejected: its scope is documentation (VER-01..10), the canonical gate table lists a separate tool; ripgrep in bash — rejected: Windows runners + escaping make a first-party Python tool the maintainable form (and it must run locally identically).

## D15 — `tools/check_npm_licenses.py` design

- **Decision**: a no-argument checker that runs `pnpm -C app licenses list --json --prod` (requires the installed store, hence it runs in the `ui` job after `pnpm install --frozen-lockfile`), parses the license fields, and fails when any package's license expression is not satisfied by the 05 §5.6 allowlist. Not a `tools/verify.ps1` gate — the 08 §8.4 table has no such local row; it is the CI-side npm enforcer 05 §5.6 names. *Review fix (F2, 2026-11-02)*: expressions are evaluated by a recursive-descent SPDX parser (`OR`=any, `AND`=all, `WITH` unit, parentheses, case-insensitive ids, fail-closed) — the original top-level split mishandled parenthesized groups.
- **Rationale**: 05 §5.6 ("enforced by `cargo deny check licenses` and `tools/check_npm_licenses.py`") — the two-sided policy: cargo-deny for the Rust graph, this tool for the pnpm graph; the asymmetry with `verify.ps1` is canonical (08 §8.4), documented in the spec (FR-018).
- **Alternatives considered**: parsing `pnpm-lock.yaml` statically — rejected: lockfiles carry no license fields; `license-checker` npm package — rejected: new dependency (G-11) when pnpm's built-in command provides the data.

## D16 — `tools/verify.ps1` is not edited

- **Decision**: no changes to `tools/verify.ps1`. Verified by reading it: `dependency-policy` activates when `deny.toml` exists and `cargo-deny` is installed (line 54), `crate-rules` when `tools/check_deps.py` exists (line 55), `placeholders` when `tools/check_placeholders.py` exists (line 56) — exactly the deliverables of this package.
- **Rationale**: the script's documented design ("Gates whose targets do not exist yet are reported as SKIPPED") means creating the targets *is* the wiring; WP-00 D7 already decided the script is not to be edited; G-08 (no weakening) is satisfied because nothing is skipped once targets exist.
- **Alternatives considered**: adding an npm-licenses gate row to `verify.ps1` — rejected: contradicts the canonical 08 §8.4 table (and the spec FR-018 now records that).

## D17 — Failure-demonstration procedure (card acceptance; stacked branch, owner-approved)

- **Decision**: after the real work is green, create the demonstration branch `ci/001-demo-failures` **from the tip of `ci/001-ci-quality-gates`** — the branch must *contain* `ci.yml` or the draft PR runs zero checks (on `pull_request` events GitHub resolves workflow files from the merge ref of base and head; a branch cut from `main` has no `ci.yml` on either side — reviewer F2). This is stacked work under G-04, approved by the owner in-session on 2026-11-02 with the requirement that the draft PR states the stacking. On the demo branch, add four commits — (a) a rustfmt violation, (b) code that trips clippy `-D warnings`, (c) an unfinished marker caught by `tools/check_placeholders.py`, (d) a commit made without `-s` — all four still carrying Conventional-Commit first lines so the `commits` job stays green and the failures localize to `rust-lint` (three failing steps) and `dco`. Open a **draft** PR to `main` whose body states: stacked on `ci/001-ci-quality-gates` for evidence only, never to be merged. Let one CI run finish; capture the failing job/step links in `specs/001-ci-quality-gates/evidence.md` and the WP-01 PR body; close the draft PR. The demonstration branch is left for the owner to delete (G-18 — agents never delete branches).
- **Rationale**: card acceptance ("deliberately failing commits (format, lint, placeholder, missing sign-off) fail the right job (evidence in PR)"); one run covers all four demos because the four commits ride one branch; draft status plus closing keeps it out of the merge path (G-03/G-05 — the failures never reach `main`); G-04's stacking approval is recorded here and restated in the draft PR body.
- **Alternatives considered**: demo branch cut from `main` — rejected: triggers no CI at all (F2 root cause); four separate PRs — rejected: four times the noise for the same evidence; pushing failures to the work branch and reverting — rejected: pollutes the real history the `commits`/`dco` jobs then check (G-07); post-merge demo — rejected: the card requires the evidence *in the WP-01 PR*.

## D18 — Baselines and the OPS-02 handoff

- **Decision**: T-SEC-001 baseline = the outputs of `cargo deny check` (advisories + licenses clean on the committed lockfiles) and `python tools/check_npm_licenses.py` (all allowlisted), recorded in `evidence.md`; T-QA-002 baseline = the wall-clock duration of a re-run of the WP-01 PR checks with warm caches (run URL + minutes), recorded the same way. The PR body carries the owner checklist for OPS-02 stage two: enable the eight required status checks (`docs`, `rust-lint`, `rust-test`, `ui`, `build`, `dco`, `commits`, `e2e-smoke`).
- **Rationale**: card acceptance ("T-SEC-001, T-QA-002 baseline recorded") and deliverable ("owner checklist for OPS-02 in the PR body"); 08 §8.2 (the two-stage settings protocol — required checks are enabled *after* WP-01 merges precisely because the jobs must exist first); 09 §9.10 rows T-SEC-001 (I, dependency advisories and license policy, NFR-SEC-01) and T-QA-002 (I, PR CI duration, NFR-MNT-02).
- **Alternatives considered**: recording only pass/fail without the raw outputs — rejected: G-09 evidence standard (commands, results, IDs).
