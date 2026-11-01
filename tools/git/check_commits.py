#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
"""Check commit messages in a range: Conventional Commits first line and DCO sign-off.

Usage: python tools/git/check_commits.py origin/main..HEAD
Rules: docs/10-execution.md §10.4.4–§10.4.5. Merge commits created by GitHub
("Merge pull request …") are exempt from both checks; other merge commits need a sign-off.
"""
from __future__ import annotations

import re
import subprocess
import sys

TYPES = "feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert|i18n"
CONVENTIONAL = re.compile(rf"^({TYPES})(\([a-z0-9-]+\))?!?: .{{1,100}}$")
MERGE_OR_REVERT = re.compile(r'^(Merge (branch|pull request|remote-tracking branch) |Revert ")')
GITHUB_MERGE = re.compile(r"^Merge pull request #\d+ ")
SIGNOFF = re.compile(r"^Signed-off-by: .+ <([^>]+@[^>]+)>\s*$", re.M)


def commits(rev_range: str) -> list[tuple[str, str, str, str]]:
    fmt = "%H%x1f%ae%x1f%P%x1f%B%x1e"
    out = subprocess.run(
        ["git", "log", "--format=" + fmt, rev_range],
        check=True, capture_output=True, text=True, encoding="utf-8",
    ).stdout
    result = []
    for record in out.split("\x1e"):
        record = record.strip("\n")
        if not record:
            continue
        sha, email, parents, body = record.split("\x1f", 3)
        result.append((sha, email, parents, body))
    return result


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    problems: list[str] = []
    try:
        entries = commits(sys.argv[1])
    except subprocess.CalledProcessError as exc:
        print(f"check_commits: git log failed for range {sys.argv[1]!r}: {exc.stderr.strip()}")
        return 2
    for sha, email, parents, body in entries:
        lines = [l for l in body.splitlines() if not l.startswith("#")]
        first = next((l for l in lines if l.strip()), "").rstrip("\r")
        short = sha[:9]
        is_merge = len(parents.split()) > 1
        if GITHUB_MERGE.match(first):
            continue
        if not (MERGE_OR_REVERT.match(first) or CONVENTIONAL.match(first)):
            problems.append(f"{short}: first line is not a Conventional Commit: {first!r}")
        signers = [m.group(1).lower() for m in SIGNOFF.finditer("\n".join(lines))]
        if not signers:
            problems.append(f"{short}: missing 'Signed-off-by' (use git commit -s)")
        elif not is_merge and email.lower() not in signers:
            problems.append(f"{short}: sign-off does not match the author email {email}")
    if problems:
        print("check_commits: problems found")
        for p in problems:
            print("  - " + p)
        return 1
    print("check_commits: all commits OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
