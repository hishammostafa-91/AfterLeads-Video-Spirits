# Evidence — WP-01 CI and quality gates

G-09 evidence for the WP-01 pull request. Each entry names the task, command, and outcome. Compiled progressively during implementation (`/speckit.implement`, 2026-11-02).

## Baseline (Phase 1)

- **T001** — `pwsh -File tools/verify.ps1` → exit 0. Before-state gate table: `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test` = **PASS**; `dependency-policy` (deny.toml or cargo-deny missing), `crate-rules` (created in WP-01), `placeholders` (created in WP-01), `bindings-drift` (no bindings yet), `ui-typecheck`/`ui-lint`/`ui-test`/`i18n` (script not defined yet) = **SKIPPED** with the script's own reasons. "All available gates passed."
- **T002** — workstation CI tools present and checked by `tools/setup/Test-DevEnvironment.ps1` (line 73): `cargo-deny 0.20.2`; `cargo-nextest 0.9.150 (d225b1daf 2026-10-26)`; `cargo-llvm-cov` installed (exercised end to end in T009).
- **T003** — clean slate verified: `.github/workflows/`, `deny.toml`, `tools/check_deps.py`, `tools/check_placeholders.py`, `tools/check_npm_licenses.py` all absent before implementation; Python 3.14.2 (stdlib `tomllib` available for the manifest parser).
