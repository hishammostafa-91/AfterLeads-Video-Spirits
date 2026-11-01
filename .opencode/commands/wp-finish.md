---
description: Finish the current work package - verify, update STATUS.md, push the branch, and open the pull request. Never merges.
---
Finish the current work package. Stop at the first failure and report it.

1. `git rev-parse --abbrev-ref HEAD` must not be `main`. Identify the work package from the branch number and its card in docs/10-execution.md §10.7.
2. Confirm `/speckit.converge` reported "Converged" for specs/<NNN>-<slug> (tasks.md fully checked). If not, stop.
3. Run `pwsh -File tools/verify.ps1`; all gates must pass.
4. Run `python tools/git/check_commits.py origin/main..HEAD`; fix messages only by asking the owner (no history rewriting of pushed commits).
5. Update STATUS.md for this work package: state `in-review`, today's date, next step `owner review`. Commit with `git commit -s -m "chore(docs): ready for review"`.
6. `git push -u origin <branch>`.
7. Create the pull request: `gh pr create --base main --title "<type>(<scope>): <work package title>" --body-file <temp file>` where the body follows .github/pull_request_template.md and includes: the card link, implemented REQ/NFR IDs, passed test IDs with the commands and summarized output, screenshots for UI changes (dark/light, en/ar), and known limitations.
8. Print the pull request URL. Do not merge.
