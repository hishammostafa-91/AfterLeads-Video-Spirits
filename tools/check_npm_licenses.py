# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
"""npm license policy check: every production dependency's license expression
must be satisfied by the allowlist of docs/05-security.md §5.6 (WP-01 card).

Usage: python tools/check_npm_licenses.py   (run after pnpm install --frozen-lockfile)
Data source: pnpm -C app licenses list --json --prod.
Exit codes: 0 pass, 1 findings, 2 environment error (contracts/tools.md).
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys

ALLOWED = frozenset(
    [
        "MIT",
        "Apache-2.0",
        "Apache-2.0 WITH LLVM-exception",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "ISC",
        "Zlib",
        "0BSD",
        "Unicode-3.0",
        "Unicode-DFS-2016",
        "BSL-1.0",
        "MPL-2.0",
        "CC0-1.0",
        "Unlicense",
        "WTFPL",
        "CDLA-Permissive-2.0",
        "LGPL-2.1-or-later",
        "LGPL-2.1-only",
        "LGPL-3.0-or-later",
        "GPL-2.0-or-later",
        "GPL-3.0-or-later",
        "AGPL-3.0-or-later",
        "OFL-1.1",
        "CC-BY-4.0",
    ]
)


def expression_ok(expr: str) -> bool:
    """Evaluate a simple SPDX expression against the allowlist (OR = any,
    AND = all; identifiers compared exactly; parentheses tolerated)."""
    text = expr.strip()
    if not text:
        return False
    for part in _split_top(text, " AND "):
        alts = _split_top(part, " OR ")
        if not any(alt.strip("() ") in ALLOWED for alt in alts):
            return False
    return True


def _split_top(text: str, sep: str) -> list[str]:
    """Split on sep at parenthesis depth 0."""
    parts, depth, start = [], 0, 0
    i = 0
    while i <= len(text) - len(sep):
        chunk = text[i : i + len(sep)]
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
        if depth == 0 and chunk == sep:
            parts.append(text[start:i])
            start = i + len(sep)
            i = start
            continue
        i += 1
    parts.append(text[start:])
    return parts


def pnpm_command() -> list[str] | None:
    """Resolve pnpm across platforms (on Windows it is a .cmd shim)."""
    path = shutil.which("pnpm")
    if path is None:
        return None
    if sys.platform == "win32":
        return ["cmd", "/c", path]
    return [path]


def main() -> int:
    pnpm = pnpm_command()
    if pnpm is None:
        print(
            "check_npm_licenses: pnpm not found on PATH (run after pnpm is installed)"
        )
        return 2
    try:
        out = subprocess.run(
            pnpm + ["-C", "app", "licenses", "list", "--json", "--prod"],
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
    except OSError as exc:
        print(f"check_npm_licenses: pnpm could not be executed: {exc}")
        return 2
    if out.returncode != 0:
        print(f"check_npm_licenses: pnpm licenses failed (exit {out.returncode}):")
        print(out.stderr.strip()[:500])
        return 2
    try:
        by_license = json.loads(out.stdout)
    except json.JSONDecodeError as exc:
        print(f"check_npm_licenses: could not parse pnpm output: {exc}")
        return 2
    problems: list[str] = []
    for expr, packages in by_license.items():
        if expression_ok(expr):
            continue
        for pkg in packages:
            problems.append(
                f"{pkg.get('name', '?')}: license {expr!r} not satisfied by the 05 §5.6 allowlist"
            )
    if problems:
        print("check_npm_licenses: findings")
        for p in problems:
            print("  - " + p)
        return 1
    print("check_npm_licenses: all npm licenses allowlisted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
