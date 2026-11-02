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

