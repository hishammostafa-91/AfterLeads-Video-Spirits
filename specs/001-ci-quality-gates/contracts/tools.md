# Contract: repository check tools

CLI contracts of the four first-party checkers this package delivers or extends. All are invoked identically locally and in CI (08 §8.4; G-08). Exit codes: 0 pass, 1 findings, 2 usage/environment error (the `tools/git/check_commits.py` convention, read in full — research.md D4).

## `tools/git/check_commits.py` (exists; extended)

```
python tools/git/check_commits.py <rev-range>                 # both rules (unchanged behavior)
python tools/git/check_commits.py <rev-range> --only conventional
python tools/git/check_commits.py <rev-range> --only dco
```

| Behavior | Contract |
|---|---|
| `--only conventional` | checks only the first-line Conventional Commits format (types/scopes of 10 §10.4.5); GitHub merge commits exempt |
| `--only dco` | checks only the DCO rule: `Signed-off-by:` present and matching the author email |
| no flag | both rules — the existing contract; sole current caller: `tools/verify.ps1` (`.githooks/commit-msg` implements its own bash copy of the same rules and does not call this tool) |
| callers | `commits` job (`--only conventional`), `dco` job (`--only dco`), `verify.ps1` gate (no flag) |

## `tools/check_deps.py` (new)

```
python tools/check_deps.py
```

Enforces docs/02 §2.4 (VER-09) over the workspace manifests, no arguments:

1. Crate dependencies only from the component's CMP row; `app/src-tauri` per CMP-12; no cycles.
2. Exclusive usage: `ffmpeg-next`→`alvs-media`; `wgpu`→`alvs-render`; `cpal`→`alvs-audio`; `rusqlite`→`alvs-store`; `whisper-rs`→`alvs-stt`; `tauri`, `webview2-com`→`app/src-tauri` only.
3. UI (`app/src`) imports Rust types only via `app/src/ipc/bindings/` (structural check now; full import-graph enforcement lands with real UI, WP-04+).
4. `#![deny(missing_docs)]` present in `alvs-core`, `alvs-model`, `alvs-formats`.

Output: `check_deps: <rule violated>: <detail>` per finding; `check_deps: all dependency rules OK` when green.

## `tools/check_placeholders.py` (new)

```
python tools/check_placeholders.py
```

Scans first-party source trees (`crates/`, `app/src`, `app/src-tauri/`, `tools/`, `.githooks/`, `tests/` when present; `app/src/ipc/bindings/` excluded as generated) for the unfinished-work markers of docs/06 §6.13 rule 4: the placeholder macros, `TODO`/`FIXME` marker comments, mocked production paths, commented-out code. Per that rule's explicit exemption, **test code and the mock backend are exempt** — structurally: `tests/` trees, test-pattern files (`*_test.rs`, `*.test.ts(x)`), `#[cfg(test)]` regions, and the mock-backend module (WP-04/05) are out of scope. Self-matching is avoided by construction (markers assembled at runtime).

Output: `check_placeholders: <file>:<line>: <marker kind>` per finding; `check_placeholders: no unfinished-work markers` when green.

## `tools/check_npm_licenses.py` (new)

```
python tools/check_npm_licenses.py
```

Runs `pnpm -C app licenses list --json --prod` (requires an installed store — run after `pnpm install --frozen-lockfile`; in CI: `ui` job; locally: on demand, not a `verify.ps1` gate per 08 §8.4) and verifies every package license against the 05 §5.6 allowlist (shared with `deny.toml`, data-model.md).

Output: `check_npm_licenses: <package>: <license> not allowlisted` per finding; `check_npm_licenses: all npm licenses allowlisted` when green.

## Shared conventions

- SPDX header + copyright line, UTF-8, LF (docs/02 §2.14).
- Python 3 standard library only — no third-party imports (no new dependencies, G-11).
- Findings are deterministic, ordered, and printed to stdout; exit codes per the convention above.
- A failing tool MUST never be silenced in CI (`continue-on-error` is reserved for the informational nightly docs-links job only).
