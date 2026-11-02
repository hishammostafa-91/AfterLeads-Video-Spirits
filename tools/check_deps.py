#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
"""Enforce the dependency rules of docs/02-architecture.md SS2.4 (VER-09, WP-01).

Rules (checked by this tool in CI, per docs/02 SS2.4):
  1. Crates depend only on the crates listed in their CMP row. No cycles.
  2. Exclusive crate usage: ffmpeg-next only alvs-media; wgpu only alvs-render;
     cpal only alvs-audio; rusqlite only alvs-store; whisper-rs only alvs-stt;
     tauri and webview2-com only the app crate (CMP-12).
  3. UI code never imports Rust types except via app/src/ipc/bindings/.
  4. #![deny(missing_docs)] on alvs-core, alvs-model, alvs-formats.

Also enforces the CMP-11 note: alvs-testkit is a dev-dependency only.

Usage: python tools/check_deps.py    Exit codes: 0 pass, 1 findings, 2 error.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# CMP rows (docs/02 SS2.4): crate name -> allowed internal crate dependencies.
INTERNAL_ALLOWED: dict[str, set[str]] = {
    "alvs-core": set(),
    "alvs-model": {"alvs-core"},
    "alvs-formats": {"alvs-core", "alvs-model"},
    "alvs-media": {"alvs-core"},
    "alvs-render": {"alvs-core", "alvs-model", "alvs-media"},
    "alvs-audio": {"alvs-core", "alvs-model", "alvs-media"},
    "alvs-engine": {
        "alvs-core",
        "alvs-model",
        "alvs-formats",
        "alvs-media",
        "alvs-render",
        "alvs-audio",
    },
    "alvs-store": {"alvs-core"},
    "alvs-jobs": {"alvs-core", "alvs-model", "alvs-store"},
    "alvs-stt": {"alvs-core", "alvs-model", "alvs-media", "alvs-store"},
    "alvs-testkit": {"alvs-core", "alvs-media", "alvs-render"},
}

APP_CRATE = "afterleads-video-spirits"
APP_DIR = ROOT / "app" / "src-tauri"
CRATES_DIR = ROOT / "crates"

# Rule 2: exclusive crate -> the single crate allowed to depend on it.
EXCLUSIVE: dict[str, set[str]] = {
    "ffmpeg-next": {"alvs-media"},
    "wgpu": {"alvs-render"},
    "cpal": {"alvs-audio"},
    "rusqlite": {"alvs-store"},
    "whisper-rs": {"alvs-stt"},
    "tauri": {APP_CRATE},
    "webview2-com": {APP_CRATE},
}

# Rule 4: crates whose lib.rs must carry #![deny(missing_docs)].
MISSING_DOCS_CRATES = ("alvs-core", "alvs-model", "alvs-formats")

UI_SRC = ROOT / "app" / "src"
UI_IMPORT = re.compile(r"""(?:from\s+|import\s+)(?:type\s+)?['"]([^'"]+)['"]""")


def manifest(path: Path) -> dict:
    with path.open("rb") as fh:
        return tomllib.load(fh)


def dep_names(section: object) -> list[str]:
    if not isinstance(section, dict):
        return []
    return [name for name, _ in section.items()]


def check_manifests(problems: list[str]) -> dict[str, set[str]]:
    """Rules 1 and 2 + CMP-11; returns the internal dependency graph."""
    graph: dict[str, set[str]] = {}
    crates = sorted(p.parent.name for p in CRATES_DIR.glob("*/Cargo.toml"))
    expected = set(INTERNAL_ALLOWED) | {"alvs-testkit"}
    for name in crates:
        if name not in expected:
            problems.append(
                f"rule 1: unexpected crate {name!r} under crates/ (not in docs/02 SS2.4)"
            )
    for name in expected:
        mf = CRATES_DIR / name / "Cargo.toml"
        if not mf.exists():
            problems.append(f"rule 1: crate {name!r} from docs/02 SS2.4 is missing")
            continue
        data = manifest(mf)
        pkg = data.get("package", {}).get("name", name)
        deps = dep_names(data.get("dependencies"))
        internal = {d for d in deps if d in INTERNAL_ALLOWED}
        graph[pkg] = internal
        for d in sorted(internal - INTERNAL_ALLOWED[name]):
            problems.append(f"rule 1: {name} depends on {d} (not in its CMP row)")
        if "alvs-testkit" in deps:
            problems.append(
                f"CMP-11: alvs-testkit is a dev-dependency only, found in [dependencies] of {name}"
            )
        for excl, allowed in EXCLUSIVE.items():
            if excl in deps and name not in allowed:
                problems.append(
                    f"rule 2: {excl} may only be used by {'/'.join(sorted(allowed))}, found in {name}"
                )
    if (APP_DIR / "Cargo.toml").exists():
        data = manifest(APP_DIR / "Cargo.toml")
        pkg = data.get("package", {}).get("name", APP_CRATE)
        deps = dep_names(data.get("dependencies"))
        graph[pkg] = {d for d in deps if d in INTERNAL_ALLOWED}
        for excl, allowed in EXCLUSIVE.items():
            if excl in deps and pkg not in allowed:
                problems.append(
                    f"rule 2: {excl} may only be used by {'/'.join(sorted(allowed))}, found in {pkg}"
                )
    return graph


def check_cycles(graph: dict[str, set[str]], problems: list[str]) -> None:
    state: dict[str, int] = {}

    def visit(node: str, stack: list[str]) -> None:
        state[node] = 1
        for dep in sorted(graph.get(node, ())):
            if state.get(dep) == 1:
                problems.append(
                    f"rule 1: dependency cycle: {' -> '.join(stack + [node, dep])}"
                )
            elif state.get(dep) is None:
                visit(dep, stack + [node])
        state[node] = 2

    for node in sorted(graph):
        if state.get(node) is None:
            visit(node, [])


def check_missing_docs(problems: list[str]) -> None:
    for name in MISSING_DOCS_CRATES:
        lib = CRATES_DIR / name / "src" / "lib.rs"
        if not lib.exists():
            problems.append(f"rule 4: {name}/src/lib.rs is missing")
            continue
        text = lib.read_text(encoding="utf-8")
        if "#![deny(missing_docs)]" not in text:
            problems.append(
                f"rule 4: #![deny(missing_docs)] not found in {name}/src/lib.rs"
            )


def check_ui_imports(problems: list[str]) -> None:
    if not UI_SRC.exists():
        return
    for path in sorted(UI_SRC.rglob("*")):
        if path.suffix not in (".ts", ".tsx") or not path.is_file():
            continue
        rel = path.relative_to(ROOT).as_posix()
        if "ipc/bindings" in rel or rel.rsplit("/", 1)[-1] in ("vite-env.d.ts",):
            continue
        for match in UI_IMPORT.finditer(
            path.read_text(encoding="utf-8", errors="replace")
        ):
            target = match.group(1)
            if "src-tauri" in target and "ipc/bindings" not in target:
                problems.append(
                    f"rule 3: {rel} imports Rust outside the generated bindings: {target!r}"
                )


def main() -> int:
    if not (CRATES_DIR.exists() and APP_DIR.exists()):
        print(
            "check_deps: workspace layout not found (crates/ and app/src-tauri expected)"
        )
        return 2
    problems: list[str] = []
    try:
        graph = check_manifests(problems)
    except tomllib.TOMLDecodeError as exc:
        print(f"check_deps: could not parse a manifest: {exc}")
        return 2
    check_cycles(graph, problems)
    check_missing_docs(problems)
    check_ui_imports(problems)
    if problems:
        print("check_deps: findings")
        for p in problems:
            print("  - " + p)
        return 1
    print("check_deps: all dependency rules OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
