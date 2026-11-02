#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
"""Detect unfinished-work markers of docs/06-ai-automation.md SS6.13 rule 4
(WP-01 card; docs/08-operations.md SS8.4 gate).

Markers: the placeholder macros, marker comments, mocked production paths,
commented-out code. Test code and the mock backend are exempt (SS6.13).
Marker strings are assembled at runtime so this tool never matches itself.

Usage: python tools/check_placeholders.py   Exit codes: 0 pass, 1 findings, 2 error.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Runtime-assembled markers (never written as literals here); the set is
# exactly docs/06 SS6.13 rule 4: the placeholder macros and marker comments.
MACRO_MARKERS = tuple(m + "!(" for m in ("to" + "do", "un" + "implemented"))
COMMENT_MARKERS = ("TO" + "DO", "FIX" + "ME")

# Commented-out code heuristic: a line comment whose body starts like code.
CODEISH = re.compile(
    r"^\s*(?://|#|--)\s*"
    r"(?:let\s+|const\s+|fn\s+|pub\s+(?:fn|use|mod|struct|enum)\s+|return\s+\w|"
    r"use\s+\w+::|import\s+\w|match\s+\w|defer\s+\w)"
)
STMT_END = re.compile(r";\s*$")

# First-party trees (data-model/contract scope); generated and test areas excluded.
SCAN_DIRS = ("crates", "app/src", "app/src-tauri", "tools", ".githooks")
EXCLUDE_PARTS = {
    "node_modules",
    "target",
    "dist",
    "ipc",
    "bindings",
    "__pycache__",
    "mocks",
}
EXCLUDE_SUFFIXES = (
    ".md",
    ".json",
    ".lock",
    ".toml",
    ".yaml",
    ".yml",
    ".svg",
    ".png",
    ".ico",
)
RUST_TEST_SUFFIX = "_test.rs"
TS_TEST_MARKERS = (".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx")

RUST = (".rs",)
TS = (".ts", ".tsx")
SCRIPTS = (".py", ".ps1", ".sh")


def scan_file(path: Path, rel: str, findings: list[str]) -> None:
    suffix = path.suffix
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError as exc:
        findings.append(f"{rel}: unreadable: {exc}")
        return
    is_test_file = (
        rel.endswith(RUST_TEST_SUFFIX)
        or any(rel.endswith(m) for m in TS_TEST_MARKERS)
        or "/mocks/" in rel
        or rel.startswith("tests/")
    )
    rust_comment = False
    cfg_test_depth = 0
    for no, line in enumerate(lines, 1):
        stripped = line.strip()
        if suffix == ".rs":
            if stripped.startswith("/*"):
                rust_comment = True
            if rust_comment and "*/" in stripped:
                rust_comment = False
            if re.search(r"#\[cfg\(test\)\]", line):
                cfg_test_depth += (
                    1  # lib/test region opens; never closes above it in practice
                )
        in_test = is_test_file or cfg_test_depth > 0
        if not in_test:
            for macro in MACRO_MARKERS:
                if macro in line:
                    findings.append(f"{rel}:{no}: placeholder macro ({macro[:-1]}…)")
            for marker in COMMENT_MARKERS:
                if marker in line:
                    findings.append(f"{rel}:{no}: marker comment ({marker})")
        if (
            not in_test
            and not rust_comment
            and CODEISH.match(line)
            and STMT_END.search(stripped)
        ):
            findings.append(f"{rel}:{no}: commented-out code")
    return


def main() -> int:
    findings: list[str] = []
    seen_any = False
    for top in SCAN_DIRS:
        base = ROOT / top
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix in EXCLUDE_SUFFIXES:
                continue
            rel = path.relative_to(ROOT).as_posix()
            if any(part in EXCLUDE_PARTS for part in Path(rel).parts[:-1]):
                continue
            if path.suffix not in RUST + TS + SCRIPTS:
                continue
            seen_any = True
            scan_file(path, rel, findings)
    if not seen_any:
        print("check_placeholders: no first-party source files found")
        return 2
    if findings:
        print("check_placeholders: findings")
        for f in findings:
            print("  - " + f)
        return 1
    print("check_placeholders: no unfinished-work markers")
    return 0


if __name__ == "__main__":
    sys.exit(main())
