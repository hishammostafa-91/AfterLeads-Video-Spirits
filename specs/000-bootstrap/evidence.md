# WP-00 — Pull-Request Evidence (G-09)

Owner: AfterLeads · Branch: `chore/000-bootstrap` · Date: 2026-11-02 · Card: docs/10 §10.7 WP-00

Every acceptance item of the card, with the command run and its summarized result. The card defines no T-xx test IDs; acceptance is gate-based.

## Acceptance 1 — `Test-DevEnvironment.ps1` all green

- Run: `pwsh -File tools/setup/Test-DevEnvironment.ps1` (2026-11-02, twice: at stage 1 and at final validation)
- Result: 28/28 checks **PASS**, zero FAIL, zero WARN — including Node.js 26 LTS line (v26.13.0) and pnpm 12.16.1 (baseline 12.x, docs/00 §0.8). Final line: `Workstation ready.`
- Note: Node was updated to 26.13.0 and pnpm to 12.16.1 during this work package (owner-approved; nvm4w channel, npm prefix pinned to the user scope).

## Acceptance 2 — `cargo build --workspace --locked`

- Run: `cargo build --workspace --locked` on the completed skeleton
- Result: **exit 0** — eleven crates + `afterleads-video-spirits` app crate; `tauri` locked at exactly **2.12.5** (Cargo.lock), `webview2-com` 0.39.1 (baseline 0.39.x).
- Fresh-clone proof (SC-006): `git clone --branch chore/000-bootstrap` into a temp dir → `Install-Hooks.ps1` → `cargo build --workspace --locked` → **exit 0 in 39.84 s**, no manual toolchain steps (1.99.1 auto-installed from `rust-toolchain.toml` on first use).

## Acceptance 3 — `pnpm -C app build`

- Run: `pnpm -C app build`
- Result: **exit 0** — `tsc && vite build`, Vite 8.3.5 (baseline 8.3.x), 16 modules, 594 ms.

## Acceptance 4 — Window opens with the correct title

- Run: `pnpm -C app tauri dev`
- Result: **Owner-confirmed on 2026-11-02**: an application window opens titled exactly `AfterLeads-Video-Spirits`, empty content (skeleton). Screenshot to be attached to the PR by the owner.

## Acceptance 5 — `python tools/docs/check_docs.py` green

- Run at final validation: `check_docs: all checks passed` — **exit 0**.
- Includes the owner-approved scope fix (commit `c05c787`): VER-06/VER-08 now exempt Spec Kit vendor files; scope recorded in docs/11 VER-06/VER-08 rows; docs set version 1.0.2 (§0.11 patch).

## Acceptance 6 — A test commit on `main` is rejected by the hook (manual check)

- Run: `git switch main` → `git commit --allow-empty -m "test: hook rejection evidence"`
- Result: **rejected by `.githooks/pre-commit`** — `pre-commit: committing on 'main' is not allowed (G-03).` — exit code 1; `main` unchanged at `f791141`; returned to the work branch. Nothing was committed.

## Gates — `pwsh -File tools/verify.ps1` (final run)

| Gate | Result |
|---|---|
| docs, commits, rust-format, rust-lint, rust-test | **PASS** |
| dependency-policy, crate-rules, placeholders, bindings-drift, ui-typecheck, ui-lint, ui-test, i18n | SKIPPED — the script's own reasons (targets created by WP-01+ / real UI work packages) |

`All available gates passed.` — exit 0.

## Deliverables inventory (card)

- Verified workstation (above) and active hooks (`core.hooksPath = .githooks`, verified twice).
- Spec Kit initialized with constitution preserved (`git diff .specify/memory/constitution.md` empty; commit `311a773`).
- Cargo workspace: root `Cargo.toml` (`crates/*` + `app/src-tauri`, resolver 3), `rust-toolchain.toml` (1.99.1 + rustfmt + clippy), `.cargo/config.toml` (`crt-static` + `control-flow-guard` for `x86_64-pc-windows-msvc`), `Cargo.lock` committed.
- Eleven empty crates `alvs-core`…`alvs-testkit`, SPDX headers + CMP-cited `lib.rs` docs (commits `790160f`).
- Tauri 2 scaffold in `app/`: identity per docs/00 §0.5 (productName/identifier/window title `AfterLeads-Video-Spirits` / `com.afterleads.videospirits`), demo code and assets removed, `tauri-plugin-opener` dropped (not in the 00 §0.8 baseline, G-11), SPDX headers on source files (commit `2427cc3`).
- Root `package.json` + `pnpm-workspace.yaml` + `pnpm-lock.yaml` committed; TypeScript ~6.0.3 (6.0.x baseline).
- STATUS.md WP-00 → `in-review` at `/wp-finish`; `done` after owner merge (card).

## Exclusions honored (card "Not included")

No CI workflows (WP-01), no FFmpeg (WP-06), no real UI (WP-04/05). G-19 check (T037): the only behavior-affecting change beyond the card — the VER-06/VER-08 scope — was committed together with its canonical documentation in `c05c787`; no other documentation requires an update for this package.

## Reviewer findings and dispositions (2026-11-02, reviewer session on `origin/main...HEAD`)

Verdict was APPROVE with six LOW findings; the owner directed all of them to be resolved.

| ID | Finding | Disposition |
|---|---|---|
| F1 | docs/11 recorded only VER-06/VER-08 exemptions while the shared file collectors also scope VER-01, VER-07, VER-10 away from vendor files | **Fixed**: shared note added under the checks table in docs/11 naming exactly VER-01, VER-06, VER-07, VER-08, VER-10; `check_docs.py` `is_vendored` docstring updated to match; docs set version bumped to 1.0.3 (§0.11 patch) |
| F2 | Transient red docs gate between Spec Kit init (`311a773`) and the scope fix (`c05c787`) — "tests pass at every commit" | **Recorded** (per reviewer: no action): inherent to the baseline conflict, branch never pushed, no CI yet. Lesson: run the docs gate immediately after committing vendor-generated files |
| F3 | Gate-tool fix `c05c787` mixed with whole-file formatter churn | **Recorded** (per reviewer: process note): the branch is local and the reformat is semantics-neutral (py_compile + green run verified); splitting it now would require history surgery for a cosmetic gain. Rule adopted: formatting-only changes get their own `style`/`chore` commit |
| F4 | Template `serde`/`serde_json` retained but unused in `app/src-tauri` | **Fixed**: both dependencies removed; workspace rebuilt and lockfile re-verified (`cargo build --workspace --locked` exit 0) |
| F5 | docs/02 §2.4 rule 4 requires `#![deny(missing_docs)]` on `alvs-core`, `alvs-model`, `alvs-formats` | **Fixed**: attribute added to all three crates; fmt/clippy/test green |
| F6 | Default create-tauri-app icons kept (third-party template assets) | **Tracked**: required for the bundler to build; to be replaced by the product icon (assets per 02 §2.13) or recorded in the notices manifest by the tooling that lands in WP-01/WP-28 |

Re-verification after the fixes: `python tools/docs/check_docs.py` exit 0; `cargo build --workspace --locked` exit 0; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` all exit 0; `pwsh -File tools/verify.ps1` exit 0.
