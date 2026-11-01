## Summary

<!-- What changes and why, in two or three sentences. -->

## Work package or issue

<!-- Example: WP-12 (docs/10-execution.md §10.7) or #154 -->

## Requirements implemented

<!-- REQ/NFR IDs from docs/01-product.md -->

## Tests and evidence (G-09)

<!-- Test IDs from docs/09-quality.md and the commands you ran with their results, e.g.:
- T-EDT-002 — cargo nextest run -p alvs-model — 214 passed
- pwsh -File tools/verify.ps1 — all gates PASS -->

## Screenshots (UI changes)

<!-- Dark and Light, English and Arabic -->

## Checklist (docs/08-operations.md §8.14)

- [ ] Branch created from `main`, up to date with `main`, merge-commit method
- [ ] Conventional Commits with DCO sign-off
- [ ] Only the requirements of this work package or issue; docs updated with behavior changes
- [ ] Tests added/updated; no tests skipped or weakened
- [ ] Inputs validated; no new network access or runtime code loading
- [ ] SPDX headers; no new dependency without an approved ADR
- [ ] Strings in all six locales; RTL checked; keyboard access and visible focus
- [ ] Performance budgets kept; no placeholders

## Notes for the reviewer
