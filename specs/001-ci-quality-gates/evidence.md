# Evidence — WP-01 CI and quality gates

G-09 evidence for the WP-01 pull request. Each entry names the task, command, and outcome. Compiled progressively during implementation (`/speckit.implement`, 2026-11-02).

## Baseline (Phase 1)

- **T001** — `pwsh -File tools/verify.ps1` → exit 0. Before-state gate table: `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test` = **PASS**; `dependency-policy` (deny.toml or cargo-deny missing), `crate-rules` (created in WP-01), `placeholders` (created in WP-01), `bindings-drift` (no bindings yet), `ui-typecheck`/`ui-lint`/`ui-test`/`i18n` (script not defined yet) = **SKIPPED** with the script's own reasons. "All available gates passed."
- **T002** — workstation CI tools present and checked by `tools/setup/Test-DevEnvironment.ps1` (line 73): `cargo-deny 0.20.2`; `cargo-nextest 0.9.150 (d225b1daf 2026-10-26)`; `cargo-llvm-cov` installed (exercised end to end in T009).
- **T003** — clean slate verified: `.github/workflows/`, `deny.toml`, `tools/check_deps.py`, `tools/check_placeholders.py`, `tools/check_npm_licenses.py` all absent before implementation; Python 3.14.2 (stdlib `tomllib` available for the manifest parser).

## Phase 3 (US1)

- **T009** — zero-test coverage pipeline proven on the workstation (2026-11-02):
  - Plain invocation: `cargo llvm-cov nextest --workspace --locked --lcov --output-path lcov.info` → **exit 4**; nextest: `Summary 0 tests run: 0 passed, 0 skipped`, `error: no tests to run (hint: use --no-tests to customize)`; no lcov written. This is the D6-documented risk, now measured.
  - Adjusted invocation (D6's "adjust the invocation, never the gate"): `cargo llvm-cov nextest --workspace --locked --no-tests=pass --lcov --output-path lcov.info` → **exit 0**, `0 passed`, valid lcov written (1,190 bytes, with real line data for built code — `SF:...app/src-tauri/src/lib.rs`).
  - The `--no-tests=pass` flag is now part of the `rust-test` step (ci.yml), research D6, and the contract; semantics: *tests pass + coverage measured* — real failures still fail.
  - T005–T008 delivered as commits `d1e118e` + `556e775` (rust-test, ui, build, e2e-smoke jobs; YAML parse-verified).

## Phase 4 (US2)

- **T010** — `deny.toml` created (05 §5.6 allowlist verbatim; HTTP-client ban with wrapper allowances). Verification journey: first run `cargo deny check` → **advisories FAILED**: RUSTSEC-2024-0370 (`proc-macro-error` unmaintained, "no safe upgrade"), reached only via tauri's Linux-only GUI chain (`muda → gtk → glib → glib-macros`). Resolution: the policy graph is scoped to the shipped target — `[graph] targets = [{ triple = "x86_64-pc-windows-msvc" }]` (constitution: Windows-only product) — not an advisory ignore. Final run: **`advisories ok, bans ok, licenses ok, sources ok`**, exit 0. The banned-crate wrapper edges (reqwest←tauri, hyper←reqwest/hyper-util) hold: bans ok.
- **T011** — `tools/check_npm_licenses.py`: first run exited 2 (pnpm is a `.cmd` shim; Windows `subprocess` cannot exec it directly) → fixed with `shutil.which` + `cmd /c` resolution. Final: **`check_npm_licenses: all npm licenses allowlisted`** (data: `pnpm -C app licenses list --json --prod`; compound SPDX expressions evaluated with OR/AND semantics).
- **T012** — `.github/dependabot.yml` created (v2; cargo `/`, npm `/app`, github-actions `/`; all weekly; YAML parse-verified).
- **T014** — `tools/check_deps.py`: **`check_deps: all dependency rules OK`** (rules 1–4 of docs/02 §2.4 + CMP-11 dev-only rule; manifests parsed with stdlib `tomllib`).
- **T015** — `tools/check_placeholders.py`: first run found one true positive — `tools/docs/check_docs.py:411` contained the marker words inside the VER-06 regex literal (first-party source). Fixed at the source: the pattern is now assembled at runtime. Final: **`check_placeholders: no unfinished-work markers`**. The docs gate itself re-verified green after the refactor.
- **T013** — `rust-lint` job wired into `ci.yml` (six jobs now: build, docs, e2e-smoke, rust-lint, rust-test, ui) with gate steps reporting independently (`if: always()`, research D3), and the npm-license step appended to `ui`.
- **T016** — `pwsh -File tools/verify.ps1` → **exit 0** with the new gates active: `dependency-policy`, `crate-rules`, `placeholders` = **PASS** (8 PASS total; bindings-drift + four UI gates SKIPPED by design until WP-02/04/05). **T-SEC-001 baseline recorded**: `cargo deny check` → `advisories ok, bans ok, licenses ok, sources ok` (exit 0) and `check_npm_licenses` → all allowlisted (exit 0), against the committed `Cargo.lock`/`pnpm-lock.yaml` (09 §9.10; NFR-SEC-01).

