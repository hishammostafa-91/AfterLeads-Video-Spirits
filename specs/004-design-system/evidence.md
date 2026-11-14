# WP-04 Evidence — commands, results, and test IDs (G-09)

Environment: Windows 11 x64, PowerShell 7, pnpm 12.x, Node 22.x; date 2026-11-14. Branch `feat/004-design-system` (8 commits at evidence time: `a246d98`…`0e43fea`, Phase 7–8 work committed on top).

## Gates

| Gate | Command | Result |
|---|---|---|
| All local gates | `pwsh -File tools/verify.ps1` | PASS — docs, commits, rust-format, rust-lint, dependency-policy, crate-rules, placeholders, rust-test, bindings-drift, ui-typecheck, ui-lint, ui-test, i18n ("All available gates passed") |
| Documentation checks | `python tools/docs/check_docs.py` | PASS |
| Commit checks | `python tools/git/check_commits.py origin/main..HEAD` | PASS (Conventional Commits + DCO on every commit) |
| UI typecheck | `pnpm -C app typecheck` | PASS |
| UI lint | `pnpm -C app lint` (eslint + stylelint + prettier) | PASS |
| UI unit tests | `pnpm -C app test` | PASS (137 tests; includes `tokens.test.ts`, `contrast.test.ts` (7), `fonts.test.ts` (4), SegmentedControl (2), MenuItemCheckbox (2)) |
| i18n | `pnpm -C app i18n:check` | PASS (115 keys identical across 6 locales; meta complete; no unused keys; no namespaced key used in code but missing from en.json) |
| Production build | `pnpm -C app exec vite build` | PASS (bundles InterVariable.woff2 352 KB, NotoSansArabic-Variable.ttf 845 KB, JetBrainsMono-Variable.ttf 303 KB) |

## Acceptance suites (quickstart rows)

| # | Test | Command | Result |
|---|---|---|---|
| 1 | T-UI-001 | `pnpm -C app test -- -t t_ui_001` | PASS — 7 tests: themes/accent/forced-colors flip live without reload |
| 2 | T-UI-002 | `pnpm -C app test -- -t t_ui_002` | PASS — 2 tests: zoom clamps 80–200, applies as root CSS zoom; harness verified at 100/125/150/175/200% (`zoom-extremes.spec.ts`) |
| 3 | T-I18N-003 | `pnpm -C app test -- -t t_i18n_003` | PASS — 5 tests: Arabic plurals, `latn` numbers/dates, LTR-isolated timecode |
| 4 | T-I18N-004 | `pnpm -C app i18n:check` then `pnpm -C app test -- -t t_i18n_004` | PASS — key sets equal (115 keys × 6 locales); 15 tests: LOC-90/LOC-91 pseudo-locales render unclipped |
| 5 | T-A11Y-004 | `pnpm -C app harness -- --grep a11y` | PASS — 4 passed (one per project): per-story axe zero violations, focus-visible rings, reduced-motion 0s, badge text+icon |
| 6 | Snapshots | `pnpm -C app harness` | PASS — 1,128 goldens byte-stable; repeat-run determinism asserted in-test; axe clean per story |

## Snapshot matrix counts (T033 + convergence)

- 4 Playwright projects: dark/light × en/ar (1280×720, reduced motion, fonts settled).
- 44 story cards in the registry (41 components — several cards compose more than one; the convergence phase added `button.segmented` plus nine `*.states` cards).
- `snapshots.spec.ts` "story matrix": 44 cards × 3 page modes (default, compact `?density=compact`, High Contrast `?theme=high-contrast`) × 4 projects = **528 goldens**; plus a repeat-capture byte-equality determinism check per project.
- `snapshots.spec.ts` "interaction states": 35 cards with a primary control × hover/focus/pressed × 4 projects = **420 goldens** (9 static stories — `menu.context`, `collections.grid-list/table/listbox/states`, `feedback.progress`, `badges.all`, `empty.media` — have no primary control; the page-mode matrix covers them, and the loop skips them instead of waiting on a non-existent target).
- `hc.spec.ts`: 44 cards × 4 projects under CDP `forced-colors: active` = **176 goldens** (`Emulation.setEmulatedMedia`; `matchMedia("(forced-colors: active)")` asserted true before capture; the skip guard is lowercase-`chromium` — Playwright reports `"chromium"`).
- `harness.spec.ts`: 4 shell goldens (one per project).
- Total committed goldens: **1,128 PNG** across 28 tests (7 specs × 4 projects); first runs write, verification runs pass. Adding cards reflows the CSS grid, so golden updates go through one `--update-snapshots` pass followed by a full verification run.
- Harness API for focused work: `?story=<id>` renders a single card; each card carries `data-story-id`; interaction captures hover with `force` (RAC overlays the native checkbox/switch/radio inputs with their `<label>`, which is the real press target).

## Contrast rulings implemented (owner, 2026-11-14)

- `docs/12 §12.17` CLR-01/04/08 Text column: white → **black** (white measured 3.91/3.03/4.12:1 vs the 4.5:1 REQ-A11Y-003 threshold; black = 6.0/6.9/5.09:1). `docs/07 §7.4.1` documents `--accent-foreground` (per-accent) and `--danger-foreground`; tokens live in `tokens.css`; `contrast.test.ts` asserts ≥4.5:1 for both themes and all five accents.
- Enforcement (T036): `color-no-hex` + `color-named` stylelint rules outside `src/design/tokens.css` — probed and verified failing on `#ff0000`, `margin-left` (→ `margin-inline-start`), a JSX text literal (`i18next/no-literal-string`), and `any` (`no-explicit-any`); probes removed; lint green.

## Deviations / notes

- **Composed Tree (D6 amendment, owner ruling A 2026-11-06)**: RAC 1.22.1 `Tree` crashes real Chromium through its hidden-`<template>` collection machinery (`getOwnerDocumentFromRootContainer(...).createTextNode is not a function` in React 19 completeWork; reproduced on 19.2.1 and 19.3.1; GridList/Table/ListBox unaffected). `Tree`/`TreeItem` are therefore composed directly over APG tree semantics (role=tree/treeitem, arrow/Home/End keys, roving tabindex) while the rest of the collections stay on RAC.
- Font format deviations from research D9 (upstream publishes no variable WOFF2): Noto Sans Arabic and JetBrains Mono ship as variable TTF; recorded per-entry in `assets/fonts/fonts.json` `sourceUrl`.
- Light-theme values for the non-default accent triples remain the documented dark triples (canonical gap flagged in `tokens.css`; §7.4.1 note added for `--accent-foreground`).
- RAC 1.22.1 gaps worked around in-component: ColorSlider/ColorWheel thumb inputs are named via a RAC `<Label>` child (`aria-label` stays on the group and leaves the input unnamed); CurveEditor renders its graph inside a real `TabPanel` so the selected tab's `aria-controls` resolves; the ContextMenu anchor is a zero-size `<span aria-hidden="true">` (an anchor `<button>` without a name fails `button-name`); WindowedListBox's scroll container is the focusable `role="listbox"` (WCAG 2.1.1 scroll access).
- Convergence additions (Phase 9): `SegmentedControl` composes RAC `RadioField`/`RadioButton` (the bare `Radio` primitive is deprecated in 1.22); checkable menu items are `MenuItemCheckbox` (role=menuitemcheckbox via the render prop, D6-style — RAC ships none); `app/src/design/icons.ts` is the Lucide base-set barrel + the §7.5 conventions (16 px dense / 20 px toolbar / stroke 1.75) and every component imports through it; the nine `*.states` cards cover the FR-007 disabled/error states for the interactive families (Fields, Selection, Menu, Overlays, Tabs, Numeric, Color, Collections) — Feedback/Badge/EmptyState are display components without disabled/error states in their §7.6 contracts, covered by the page-mode matrix instead.
