# Data Model: WP-01 — CI and quality gates

**Scope note**: this work package adds no product data (docs/03 is not in the card's Read line; the spec's canonical basis records this). The "data" it introduces is *configuration*: workflows, jobs, gates, and policy sets. Those configuration entities are documented here because tasks and review verify against them.

## Entities

### CI job (configuration entity)

| Field | Value / rule | Source |
|---|---|---|
| id (= required check name) | one of `docs`, `commits`, `dco`, `rust-lint`, `rust-test`, `ui`, `build`, `e2e-smoke` | 08 §8.2, §8.3.1 |
| runner | `ubuntu-latest` for `docs`/`commits`/`dco`; `windows-2025` for the rest | 08 §8.3.1 |
| trigger | `pull_request`→`main`, `push`→`main` (concurrency per branch, cancel in progress) | 08 §8.3.1 |
| blocking | all jobs are merge-blocking once OPS-02 stage two is enabled (owner action, after merge) | 08 §8.2 |

Validation: a job id not in the OPS-02 list, or a job absent from `ci.yml`, is a defect (the required-check contract breaks).

### Nightly job (configuration entity)

| Field | Value / rule | Source |
|---|---|---|
| id | `dependency-audit` (daily 02:17 UTC), `docs-links` (weekly Sunday 02:27 UTC, informational) | 08 §8.3.2; card |
| trigger | `schedule` + `workflow_dispatch` | 08 §8.3.2 |
| blocking | never (nightly is not a merge gate) | 08 §8.3.2 |

### Action pin (configuration entity)

| Field | Value / rule | Source |
|---|---|---|
| reference | `owner/repo@<40-hex commit SHA>` — version tags never used directly | 05 §5.6 rule 5; card |
| current set | the six actions of research.md D1 | research.md D1 (verified 2026-11-02) |

Validation: any `uses:` without a full SHA is a defect (SC-005).

### License policy set (deny.toml)

| Field | Value / rule | Source |
|---|---|---|
| allow | exactly the 05 §5.6 allowlist (24 SPDX ids incl. `OFL-1.1`, `CC-BY-4.0`) | 05 §5.6; research.md D12 |
| deny | the 05 §5.6 denied set — enforced structurally: anything not in `allow` fails | 05 §5.6; research.md D12 |
| bans | HTTP-client crates denied (`reqwest`, `ureq`, `isahc`, `surf`, `attohttpc`, `minreq`, `hyper`, `curl`, `curl-sys`); the single future allowance is `ureq` wrapped by `alvs-stt` (SEC-10/ADR-023, WP-21) | card; 05 §5.7 SEC-10; research.md D12 |
| sources | registries: crates.io only; unknown registries and git sources denied | 05 §5.6 rule 3; research.md D12 |
| advisories | RustSec advisories deny (PR + nightly audit) | 05 §5.6 rule 3; 08 §8.3.2 |

### npm license policy (tools/check_npm_licenses.py)

| Field | Value / rule | Source |
|---|---|---|
| data source | `pnpm -C app licenses list --json --prod` (after `pnpm install --frozen-lockfile`) | research.md D15 |
| allow | the same 05 §5.6 allowlist (shared constant with the policy set) | 05 §5.6 |
| enforcement point | `ui` job in CI; not a `tools/verify.ps1` gate row | 08 §8.4; research.md D15, D16 |

### Check-tool result (contract shared by all four tools)

| Field | Value / rule | Source |
|---|---|---|
| exit 0 | all checks pass, one summary line printed | 08 §8.4 gate convention |
| exit 1 | findings found, one per line, prefixed with the tool name | 08 §8.4; `check_commits.py` precedent |
| exit 2 | usage/environment error (bad arguments, unreadable input) | `check_commits.py` precedent (read in full) |

## State transitions

No runtime state: workflows are declarative configuration; the only lifecycle is the OPS-02 two-stage protocol — jobs exist (this WP) → owner enables required checks (stage two, after merge) — which is a repository-settings state, never agent-modified (08 §8.2; G-18).

## Relationships

- `ci.yml` job id ⇒ OPS-02 required check name (1:1, both directions — a rename breaks protection).
- `deny.toml` allow set ⇔ npm allowlist constant (same source: 05 §5.6; kept identical by review).
- `rust-lint` steps ⇔ `tools/verify.ps1` gates `rust-format`/`rust-lint`/`dependency-policy`/`crate-rules`/`placeholders` (same commands, local and CI — 08 §8.4; G-08).
