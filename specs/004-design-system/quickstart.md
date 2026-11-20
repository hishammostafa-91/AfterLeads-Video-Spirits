# WP-04 Quickstart — validation guide

Run from the repository root (PowerShell 7; `pnpm` 12.x). First run: `pnpm -C app install` creates `app/pnpm-lock.yaml` and `node_modules`; Playwright browsers install with `pnpm -C app exec playwright install chromium`.

## Gates (must all pass once the scripts exist)

```powershell
pwsh -File tools/verify.ps1
# docs, commits, rust-format, rust-lint, dependency-policy, crate-rules,
# placeholders, rust-test, bindings-drift  → PASS
# ui-typecheck, ui-lint, ui-test, i18n      → PASS (no longer SKIPPED)
```

## Acceptance suites

| # | Test | Command | Expected |
|---|---|---|---|
| 1 | T-UI-001 — themes, accents, no restart | `pnpm -C app test -- -t t_ui_001` | all pass: dark↔light switch flips token values live; each accent swaps the three accent tokens; forced-colors maps system colors; switching needs no reload |
| 2 | T-UI-002 — zoom and DPI crispness | `pnpm -C app test -- -t t_ui_002` | zoom clamps to 80–200 and applies as root CSS zoom; SVG/tabular-num renders verified in the harness at 100/125/150/175/200% scaling |
| 3 | T-I18N-003 — ICU and locale formatting | `pnpm -C app test -- -t t_i18n_003` | Arabic plural categories; `Intl` numbers/dates with `latn`; timecode in an LTR isolate inside RTL prose |
| 4 | T-I18N-004 — key sets and pseudo-locales | `pnpm -C app i18n:check` then `pnpm -C app test -- -t t_i18n_004` | equality check green (and fails when a key is deliberately removed); LOC-90 expansion (+40%) and LOC-91 RTL render without clipping |
| 5 | T-A11Y-004 — contrast, forced colors, motion, color-alone | `pnpm -C app harness -- --grep a11y` | AA ratios in dark+light; forced-colors mapping; reduced motion zeroes durations; badges/labels carry text+icon |
| 6 | Snapshots — every component, 9 states, dark/light × en/ar | `pnpm -C app harness` | all screenshots match; repeat run is stable through Playwright's per-pixel threshold comparator (determinism); axe reports zero violations per story |

## Point checks

- Theme runtime: open `/harness?theme=dark&locale=en`, toggle theme/accent in the harness toolbar — instant re-theme.
- RTL: `/harness?locale=ar` — mirrored layout, LTR timecode/transport, `dir="auto"` list content.
- Fonts: `pnpm -C app test -- fonts` re-hashes `assets/fonts/*` against `fonts.json` and asserts each `OFL.txt` exists.
- Enforcement: add a raw color to any component CSS or a JSX literal to any TSX → `pnpm -C app lint` fails.
