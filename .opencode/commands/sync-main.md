---
description: Merge the latest origin/main into the current work branch with a merge commit (never rebase).
---
1. Run `pwsh -File tools/git/Sync-Main.ps1`.
2. If it reports conflicts: list the conflicted files with `git status --short`; resolve only conflicts whose correct result is clear from the docs and both sides' intent; otherwise stop and ask the owner. After resolving, run `pwsh -File tools/verify.ps1`, then `git add -A` and `git commit -s --no-edit`.
3. Report the resulting merge commit and gate results.
