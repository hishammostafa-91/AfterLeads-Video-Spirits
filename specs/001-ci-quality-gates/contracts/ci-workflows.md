# Contract: CI workflows (`.github/workflows/ci.yml`, `nightly.yml`, `dependabot.yml`)

The workflows' externally observable contract: what triggers them, which check names they publish, and what each job guarantees when green. Sources: 08 §8.3.1–8.3.2, §8.2; 05 §5.6; card. Pin list: research.md D1.

## `ci.yml` — pull-request CI (OPS-03)

| Aspect | Contract |
|---|---|
| Triggers | `pull_request` targeting `main`; `push` to `main` |
| Concurrency | per-branch group, cancels in-progress superseded runs |
| Permissions | `contents: read` (read-only default, 08 §8.2) |
| Budget | total ≤ 25 min with warm caches (NFR-MNT-02, T-QA-002) |
| Published checks | exactly: `docs`, `commits`, `dco`, `rust-lint`, `rust-test`, `ui`, `build`, `e2e-smoke` (the OPS-02 required-check set) |

### Job guarantees

| Check | Runner | Guarantees when green | Steps (skeleton scope) |
|---|---|---|---|
| `docs` | ubuntu-latest | Documentation consistency VER-01..VER-10 pass | `python3 tools/docs/check_docs.py` |
| `commits` | ubuntu-latest | Every commit in range is a Conventional Commit (10 §10.4.5) | checkout `fetch-depth: 0`; `python3 tools/git/check_commits.py origin/main..HEAD --only conventional` |
| `dco` | ubuntu-latest | Every commit carries `Signed-off-by:` matching the author email | checkout `fetch-depth: 0`; `python3 tools/git/check_commits.py origin/main..HEAD --only dco` |
| `rust-lint` | windows-2025 | Format, clippy `-D warnings` (locked), dependency policy, crate rules, no placeholders — on the pinned toolchain | `rustup show`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; install `cargo-deny`; `cargo deny check`; `python tools/check_deps.py`; `python tools/check_placeholders.py` |
| `rust-test` | windows-2025 | All workspace tests pass under nextest (WARP forced via step-level `env:`); coverage artifact published — zero-test baseline is expected until crates gain tests | `rustup show`; install `cargo-nextest` + `cargo-llvm-cov`; `rustup component add llvm-tools-preview`; `cargo llvm-cov nextest --workspace --locked --no-tests=pass --lcov --output-path lcov.info` (env `ALVS_FORCE_WARP: 1`); upload `lcov.info` |
| `ui` | windows-2025 | Frozen pnpm install; skeleton UI type-correctness build; every npm license allowlisted | pnpm setup (12.16.1); `pnpm install --frozen-lockfile`; `pnpm -C app build`; `python tools/check_npm_licenses.py` |
| `build` | windows-2025 | The application crate builds debug, locked | `cargo build -p afterleads-video-spirits --locked` |
| `e2e-smoke` | windows-2025 | Explicitly a no-op reporting success until WP-05 (card) | one explanatory step, exit 0 |

Deferred steps and their owners: bindings drift (WP-02), vcpkg cache + golden smoke (WP-06/later), `ui` lint/test/i18n/harness (WP-04/05), executable-start `cli version` check (CLI entry-point WP), real e2e-smoke (WP-05). Adding any of them early violates G-02; leaving a gap undocumented violates this contract.

## `nightly.yml` — nightly CI (OPS-05, card-scoped subset)

| Aspect | Contract |
|---|---|
| Triggers | `schedule: 17 2 * * *` (daily) and `27 2 * * 0` (weekly Sunday); `workflow_dispatch`. Per-job guards select the cadence: `dependency-audit` on the daily entry (or dispatch), `docs-links` on the weekly entry (or dispatch) — `github.event.schedule` holds the cron string that fired. The second schedule entry is documented in docs/08 §8.3.2 (G-19 note in this work package) |
| Permissions | `contents: read` |
| Jobs now | `dependency-audit` (windows-2025): checkout (pinned); `rustup show`; install `cargo-deny` (install-action, pinned); `cargo deny check advisories`; `pnpm/action-setup` (pinned, 12.16.1); `pnpm install --frozen-lockfile`; `pnpm audit --prod` — failures surface nightly; NFR-SEC-01 holds them to 0 high/critical at release. `docs-links` (ubuntu-latest): checkout (pinned); pinned `lycheeverse/lychee-action` over `docs/`; `continue-on-error: true` (informational, 08 §8.3.2) |
| Jobs later (out of scope) | full test suite, full E2E, fuzz, benchmarks — added by their owning WPs (card) |

## `dependabot.yml`

| Aspect | Contract |
|---|---|
| Cadence | weekly, all ecosystems (05 §5.6 rule 5) |
| Sets | `cargo` at `/`; `npm` at `/app`; `github-actions` at `/` |
| Side condition | every Dependabot PR is an ordinary PR: it must pass `ci.yml` unchanged (lockfiles revalidated frozen, 05 §5.6 rule 2) |

## Cross-cutting guarantees

- Every `uses:` is pinned to a full commit SHA (05 §5.6 rule 5; SC-005) — the six pinned actions of research.md D1; no other third-party actions.
- Workflow files use LF, two-space indent, no trailing whitespace, and carry no SPDX header: docs/02 §2.14 enumerates header-bearing source kinds (Rust, TS/TSX, CSS, WGSL, PowerShell, Python, sh) — YAML is not among them, and CI configuration follows the repository's headerless-config precedent (`.gitattributes`, `.github/pull_request_template.md`); the license-bearing artifacts of this package are the Python tools and `deny.toml`.
- A red job names its failing step; deliberately failing commits map: format/lint/placeholder → `rust-lint`; missing sign-off → `dco` (card acceptance, SC-002).
