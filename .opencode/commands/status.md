---
description: Show the repository and work-package status.
---
Report, in this order:
1. `git status --short --branch` and the current branch's upstream.
2. `git config --get core.hooksPath` — it must be `.githooks`; if not, tell the owner to run `pwsh -File tools/git/Install-Hooks.ps1`.
3. From STATUS.md: work packages `in-progress`, `in-review`, and `blocked` (with their questions).
4. The next candidate work packages: state `todo` and every dependency (docs/10-execution.md §10.8) `done`.
5. A warning if the current branch is `main` and there are uncommitted changes.
