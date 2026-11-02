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
import re
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
ALLOWED_UPPER = frozenset(a.upper() for a in ALLOWED)


def expression_ok(expr: str) -> bool:
    """Evaluate an SPDX license expression against the allowlist:
    OR = any, AND = all, WITH joins an id and its exception into one unit,
    parentheses group; identifiers match case-insensitively. Unparseable or
    empty expressions are not satisfied (fail closed)."""
    tokens = _tokenize(expr)
    if not tokens:
        return False
    ok, rest = _parse_or(tokens)
    return ok and not rest


def _tokenize(expr: str) -> list[str]:
    tokens = re.findall(r"\(|\)|[A-Za-z0-9][A-Za-z0-9.+-]*", expr)
    normalized = []
    for tok in tokens:
        upper = tok.upper()
        normalized.append(upper if upper in ("AND", "OR", "WITH") else tok)
    return normalized


def _parse_or(tokens: list[str]) -> tuple[bool, list[str]]:
    ok, rest = _parse_and(tokens)
    while rest and rest[0] == "OR":
        rhs, rest = _parse_and(rest[1:])
        ok = ok or rhs
    return ok, rest


def _parse_and(tokens: list[str]) -> tuple[bool, list[str]]:
    ok, rest = _parse_unit(tokens)
    while rest and rest[0] == "AND":
        rhs, rest = _parse_unit(rest[1:])
        ok = ok and rhs
    return ok, rest


def _parse_unit(tokens: list[str]) -> tuple[bool, list[str]]:
    if not tokens:
        return False, tokens
    if tokens[0] == "(":
        ok, rest = _parse_or(tokens[1:])
        if not rest or rest[0] != ")":
            return False, tokens
        return ok, rest[1:]
    ident = tokens[0]
    rest = tokens[1:]
    if rest and rest[0] == "WITH":
        if len(rest) < 2:
            return False, tokens
        ident = f"{ident} WITH {rest[1]}"
        rest = rest[2:]
    return ident.upper() in ALLOWED_UPPER, rest


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
