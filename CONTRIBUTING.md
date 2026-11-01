# Contributing to AfterLeads-Video-Spirits

Thank you for helping. This project grows best when improvements land **here**, in the main repository, so every user benefits. Small, focused pull requests are reviewed quickly.

## Ways to contribute

| Contribution | Where | Notes |
|---|---|---|
| Bug reports | GitHub Issues (Bug report form) | Include steps, expected/actual result, version; attach a diagnostics bundle if you can |
| Features | GitHub Issues (Feature request form) | Explain the problem first; features must fit the scope in `docs/01-product.md` |
| Code | Pull request | Follow the rules below |
| Translations | Pull request to `app/src/locales/` | Use the terminology table in `docs/07-experience.md` §7.12; new languages are welcome |
| Title templates, LUT looks | Pull request to `assets/` | Original work only, under the project license |
| Documentation | Pull request to `docs/` | Canonical-document rules in `docs/00-index.md` §0.3 |
| Testing | Beta builds, reports | Especially Arabic/RTL, accessibility, and different GPUs |

## Ground rules

The global rules in [docs/00-index.md §0.4](docs/00-index.md#04-global-rules-g-xx) apply to every contributor. The most important ones:

1. **Branch from `main`, never commit to `main`.** Name branches `<type>/<NNN>-<slug>` (for example `fix/154-relink-crash`), where `NNN` is the issue number.
2. **One change per pull request.** Keep it small and focused.
3. **Conventional Commits** for messages (`feat(timeline): add ripple trim`) and a **DCO sign-off** on every commit (`git commit -s`).
4. **Tests and evidence.** New behavior comes with tests; the pull request says which checks you ran.
5. **Localization and accessibility by default.** No hard-coded user-facing text; keyboard access and visible focus for every control.
6. **No copied code or assets** from other products. Third-party dependencies need an allowed license (`docs/05-security.md` §5.6).

## Workflow

1. Fork (external contributors) or create a branch (maintainers) from up-to-date `main`.
2. Install the repository hooks: `pwsh -File tools/git/Install-Hooks.ps1`.
3. Make your change, then run `pwsh -File tools/verify.ps1`.
4. Bring in the latest `main` with a merge (`pwsh -File tools/git/Sync-Main.ps1`) — do not rebase pushed commits.
5. Open a pull request against `main` and complete the template.
6. The maintainer reviews with the checklist in `docs/08-operations.md` §8.14 and merges with a merge commit.

## Developer Certificate of Origin (DCO)

By signing off a commit you certify the Developer Certificate of Origin 1.1 (https://developercertificate.org/): you wrote the contribution or have the right to submit it under the project's license. Sign off with `git commit -s`, which adds:

```text
Signed-off-by: Your Name <your.email@example.com>
```

There is no contributor license agreement. Contributions are accepted under the project license, **AGPL-3.0-or-later** (inbound = outbound).

## Forks and the project name

You may fork the code under the AGPL. If you distribute a modified version, you must use a different name and logo (see [TRADEMARKS.md](TRADEMARKS.md)). We would much rather merge your improvement here — open a pull request or an issue first.

## Code of conduct

Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
