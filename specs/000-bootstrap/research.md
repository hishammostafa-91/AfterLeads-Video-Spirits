# Research: WP-00 — Workstation and repository bootstrap

**Date**: 2026-11-02 · **Inputs**: WP-00 card (docs/10 §10.7), docs/00 §0.5, §0.8; docs/02 §2.13–2.14; docs/06 §6.8–6.11; docs/10 §10.2, §10.4; repository scripts and hooks (read directly, G-10).

No NEEDS CLARIFICATION markers remain. Every decision below is derived from a canonical section or verified against repository code.

## D1 — Root Cargo workspace membership

- **Decision**: one workspace manifest at the repository root with members `crates/*` and `app/src-tauri`.
- **Rationale**: the card states the membership verbatim; docs/02 §2.13 shows `Cargo.toml` at the root with `crates/alvs-*` and `app/src-tauri` below it.
- **Alternatives considered**: crate-local workspaces or a single catch-all crate — rejected: contradicts the canonical layout (G-01).

## D2 — Toolchain pinning

- **Decision**: `rust-toolchain.toml` pins channel `1.99.1` with components `rustfmt` and `clippy` (card; docs/00 §0.8 "Rust 1.99.1 (edition 2024), pinned in rust-toolchain.toml").
- **Rationale**: a fresh clone must build with the exact toolchain automatically (spec edge case 1); the pin is the canonical mechanism.
- **Alternatives considered**: documenting the version without pinning — rejected: docs/00 §0.8 names the file as the pinning mechanism.

## D3 — `.cargo/config.toml` hardening flags

- **Decision**: set `rustflags = ["-C", "target-feature=+crt-static", "-C", "control-flow-guard"]` for the `x86_64-pc-windows-msvc` target, exactly as the card writes them.
- **Rationale**: card deliverable verbatim (security hardening of the Windows build; G-15 posture).
- **Alternatives considered**: per-crate `RUSTFLAGS`, CI-only flags — rejected: not in the card (G-02, G-10).

## D4 — Empty crates and gate activation

- **Decision**: create the eleven crates `alvs-core`, `alvs-model`, `alvs-formats`, `alvs-media`, `alvs-render`, `alvs-audio`, `alvs-engine`, `alvs-store`, `alvs-jobs`, `alvs-stt`, `alvs-testkit` (docs/02 §2.13) as empty libraries: `Cargo.toml` + `src/lib.rs` with the SPDX header (docs/02 §2.14) and crate-level documentation; no code.
- **Rationale**: card deliverable ("empty crates … with SPDX headers and `lib.rs` docs"). Verified against `tools/verify.ps1` (read in full): the moment `Cargo.toml` exists, gates `rust-format`, `rust-lint` (`clippy --workspace --all-targets --locked -- -D warnings`), and `rust-test` become active — so the skeleton must be fmt-clean, clippy-clean (including pedantic review posture per docs/02 §2.14), and test-green from the first commit.
- **Alternatives considered**: skeleton with placeholder `todo!()` bodies — rejected: anti-hallucination rule 4 forbids placeholders in merged code.

## D5 — Tauri scaffold and identity

- **Decision**: run `pnpm create tauri-app@latest` (React, TypeScript, pnpm) into `app/` (card Steps 4); set `productName` "AfterLeads-Video-Spirits", `identifier` "com.afterleads.videospirits", window title "AfterLeads-Video-Spirits" (card; docs/00 §0.5 — the exact constants, never shortened); remove template demo code and assets; add SPDX headers to source files; pin dependency versions to the docs/00 §0.8 baseline (Tauri 2.12.x: crate 2.12.5, `@tauri-apps/api`/`cli` 2.12.4).
- **Rationale**: card Steps 4 verbatim; identity constants are canonical (G-20); template removal keeps originality (G-12).
- **Alternatives considered**: hand-written minimal Tauri app — rejected: the card names the scaffold tool; license headers and pinned versions are then adjusted on top (G-01).

## D6 — pnpm workspace and TypeScript pin

- **Decision**: root `package.json` + `pnpm-workspace.yaml` with `app` as the workspace package (docs/02 §2.13); TypeScript pinned to 6.0.x (card; docs/00 §0.8).
- **Rationale**: card deliverable; the workspace file is the canonical mechanism for `pnpm -C app …` commands used throughout the docs.
- **Verified environment**: pnpm 12.16.1 active (baseline 12.x — docs/00 §0.8), Node 26.13.0 LTS active; `Test-DevEnvironment.ps1` all PASS on 2026-11-02.
- **Scaffold pin list (docs/00 §0.8 UI rows, verified)**: TypeScript **6.0.x** (7.x unsupported by `typescript-eslint` peer `<6.1.0` — the docs state the reason); React **19.3.x**, Vite **8.3.x**, `@vitejs/plugin-react` **6.x**; Tauri **2.12.x** (crate 2.12.5, `@tauri-apps/api`/`cli` 2.12.4). The scaffold step pins these at implementation time; deviations need an ADR (G-11).
- **Lockfiles**: `Cargo.lock` and `pnpm-lock.yaml` are generated during the build/scaffold steps and **committed** — docs/00 §0.8 locks exact versions by these files, and the card's acceptance commands run with `--locked` (which fails without `Cargo.lock`).
- **Alternatives considered**: npm workspaces — rejected: canonical baseline is pnpm (docs/00 §0.8; docs/10 §10.2 tool table).

## D7 — verify.ps1 gate activation map (verified by reading the script)

- **Decision**: accept the script's own skip behavior; never edit it.
- **After WP-00 the gates run**: `docs`, `commits`, `rust-format`, `rust-lint`, `rust-test`. **Stay SKIPPED by the script's own rules**: `dependency-policy` (`deny.toml` absent — not a WP-00 deliverable), `crate-rules`, `placeholders` (`tools` created in WP-01), `bindings-drift` (no bindings), `ui-typecheck`/`ui-lint`/`ui-test`/`i18n` (app scripts not defined by the scaffold — they arrive with the real UI tooling).
- **Rationale**: G-08 (never weaken a gate) + the script's documented design ("Gates whose targets do not exist yet are reported as SKIPPED", tools/verify.ps1 header).
- **Alternatives considered**: adding `deny.toml` or UI scripts now to activate more gates — rejected: not in the card (G-02).

## D8 — "Test commit on main rejected" mechanism (verified)

- **Decision**: the acceptance check is demonstrated with `.githooks/pre-commit`, whose branch case `main|master` prints "committing on 'main' is not allowed (G-03)" and exits 1 (read and verified in `.githooks/pre-commit` lines 8–13).
- **Rationale**: card acceptance names the hook; the mechanism is now identified precisely for the PR evidence (G-09).
- **Alternatives considered**: proving via `pre-push` — rejected: `pre-push` guards pushes, the acceptance names a *commit* on main.

## D9 — Template-origin licensing

- **Decision**: the create-tauri-app template is the sanctioned scaffold source (card Steps 4); generated boilerplate is adjusted (identity, demo removal, SPDX headers per docs/02 §2.14). Third-party dependency licenses are recorded by the notices tooling when lockfiles are pinned ("no generated files edited by hand … notices are regenerated by scripts" — docs/02 §2.14; packaging WP-28).
- **Rationale**: G-12 requires allowlisted licenses recorded in the notices manifest; the manifest is script-generated, and WP-00 adds no dependency beyond the 00 §0.8 baseline (G-11).
- **Alternatives considered**: hand-maintained notices now — rejected: docs/02 §2.14 defines notices as regenerated artifacts.

## D10 — Already-satisfied deliverables on this branch

- **Decision**: record as done with evidence, do not redo: workstation all-green (2026-11-02 run, 28/28 PASS incl. Node 26 LTS, pnpm 12.16.1); Spec Kit initialized with constitution preserved (`git diff` empty; commit `311a773`); WP started (branch `chore/000-bootstrap`, STATUS.md `in-progress`, commit `202c975`); spec committed (`0428960`).
- **Rationale**: constitution III (recorded evidence) and G-09; the flow's plan checkpoint follows.
- **Alternatives considered**: none — re-running completed steps would violate G-07 (clean small commits) and waste the checkpoint.
