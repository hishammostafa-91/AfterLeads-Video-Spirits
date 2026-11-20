# WP-04 Evidence — commands, results, and test IDs (G-09)

Environment: Windows 11 x64, PowerShell 7, pnpm 12.x, Node 26.x; dates 2026-11-14..16. Branch `feat/004-design-system`.

## Gates

| Gate | Command | Result |
|---|---|---|
| All local gates | `pwsh -File tools/verify.ps1` | PASS — docs, commits, rust-format, rust-lint, dependency-policy, crate-rules, placeholders, rust-test, bindings-drift, ui-typecheck, ui-lint, ui-test, i18n ("All available gates passed") |
| CI (T042) | PR #11 checks | PASS — all 9 green on `feat/004-design-system`: `ui` [job 132935456005](https://github.com/hishammostafa-91/AfterLeads-Video-Spirits/actions/runs/43754482248/job/132935456005), `ui-harness` [job 132935456199](https://github.com/hishammostafa-91/AfterLeads-Video-Spirits/actions/runs/43754482248/job/132935456199), plus build/rust-lint/rust-test/docs/commits/dco/e2e-smoke (run [43754482248](https://github.com/hishammostafa-91/AfterLeads-Video-Spirits/actions/runs/43754482248)) - the golden matrix is threshold-stable in CI (Playwright's per-pixel comparator) |
| Documentation checks | `python tools/docs/check_docs.py` | PASS |
| Commit checks | `python tools/git/check_commits.py origin/main..HEAD` | PASS (Conventional Commits + DCO on every commit) |
| UI typecheck | `pnpm -C app typecheck` | PASS |
| UI lint | `pnpm -C app lint` (eslint + stylelint + prettier) | PASS |
| UI unit tests | `pnpm -C app test` | PASS (158 tests over 19 files; includes the round-2 token-hygiene scanner and the real used-id i18n scan) |
| i18n | `pnpm -C app i18n:check` | PASS (130 keys identical across 6 locales; meta complete; no unused keys; no namespaced key used in code but missing from en.json) |
| Production build | `pnpm -C app exec vite build` | PASS (bundles InterVariable.woff2 352 KB, NotoSansArabic-Variable.ttf 845 KB, JetBrainsMono-Variable.ttf 303 KB) |

## Acceptance suites (quickstart rows)

| # | Test | Command | Result |
|---|---|---|---|
| 1 | T-UI-001 | `pnpm -C app test -- -t t_ui_001` | PASS — 7 tests: themes/accent/forced-colors flip live without reload |
| 2 | T-UI-002 | `pnpm -C app test -- -t t_ui_002` | PASS — 2 unit tests (clamp + `--app-zoom`); the harness `zoom-extremes.spec.ts` covers zoom 80/200 (LOC-90 clipping) and every monitor-DPI step 100/125/150/175/200 with no document overflow |
| 3 | T-I18N-003 | `pnpm -C app test -- -t t_i18n_003` | PASS — 5 tests: Arabic plurals, `latn` numbers/dates, LTR-isolated timecode |
| 4 | T-I18N-004 | `pnpm -C app i18n:check` then `pnpm -C app test -- -t t_i18n_004` | PASS — key sets equal (130 keys × 6 locales); 16 tests: LOC-90/LOC-91 pseudo-locales render unclipped |
| 5 | T-A11Y-004 | `pnpm -C app harness -- --grep a11y` | PASS — 4 passed (one per project): per-story axe zero violations, visible 2 px `--focus-ring` focus rings (asserted by width + token color), reduced-motion zeroes transitions while progress indicators keep animating, badge text+icon |
| 6 | Snapshots | `pnpm -C app harness` | PASS - 72 tests, 1,224 goldens threshold-stable; repeat-run determinism asserted in-test through Playwright's per-pixel threshold comparator (content-sized text boundaries jitter ±1 between rasterizations - byte equality would false-fail on that noise); axe clean per story |

## Snapshot matrix counts (T033 + convergence)

- 4 Playwright projects: dark/light × en/ar (1280×720, reduced motion, fonts settled).
- 44 story cards in the registry (41 components — several cards compose more than one; the convergence phase added `button.segmented` plus nine `*.states` cards).
- `snapshots.spec.ts` "story matrix": 44 cards × 3 page modes (default, compact `?density=compact`, High Contrast `?theme=high-contrast`) × 4 projects = **528 goldens**; plus a repeat-capture determinism check through the threshold comparator per project.
- `snapshots.spec.ts` "interaction states": 42 cards with a primary control × hover/focus/pressed × 4 projects = **504 goldens** (the M10 selector widening added the Tree/GridList/Table/ListBox rows/options; cards whose only content matches nothing — `menu.context`'s area, progress, badges, empty states — have no interaction state to capture; the page-mode matrix covers them).
- `hc.spec.ts`: 44 cards × 4 projects under CDP `forced-colors: active` = **176 goldens** (`Emulation.setEmulatedMedia`; `matchMedia("(forced-colors: active)")` asserted true before capture; the skip guard is lowercase-`chromium` — Playwright reports `"chromium"`).
- `harness.spec.ts`: 4 shell goldens (one per project).
- Total committed goldens: **1,224 PNG** across 72 tests (the suite grew with the reviewer fixes: zoom 80/200 + the five DPI steps, the theme/accent computed-value tests, the HC component-pairing test); first runs write, verification runs pass. Adding cards reflows the CSS grid, so golden updates go through one `--update-snapshots` pass followed by a full verification run.
- Harness API for focused work: `?story=<id>` renders a single card; each card carries `data-story-id`; interaction captures hover with `force` (RAC overlays the native checkbox/switch/radio inputs with their `<label>`, which is the real press target).

## Contrast rulings implemented (owner, 2026-11-14)

- `docs/12 §12.17` CLR-01/04/08 Text column: white → **black** (white measured 3.91/3.03/4.12:1 vs the 4.5:1 REQ-A11Y-003 threshold; black = 6.0/6.9/5.09:1). `docs/07 §7.4.1` documents `--accent-foreground` (per-accent) and `--danger-foreground`; tokens live in `tokens.css`; `contrast.test.ts` asserts ≥4.5:1 for both themes and all five accents.
- Enforcement (T036): `color-no-hex` + `color-named` stylelint rules outside `src/design/tokens.css` — probed and verified failing on `#ff0000`, `margin-left` (→ `margin-inline-start`), a JSX text literal (`i18next/no-literal-string`), and `any` (`no-explicit-any`); probes removed; lint green.

## Reviewer fixes (owner-agreed 2026-11-15)

Owner agreed to fix the CRITICAL/HIGH set; the standing "all findings including low ones" policy covered the mechanical MEDIUM/LOW set in the same pass:

- **H1** `Fields.tsx` recomposed on RAC `Label`+`Input` (label association; `isDisabled`/`isInvalid`/`isRequired` propagate); `TextArea` gains a controlled `value`/`onChange` and `useId` label wiring. Tests assert `getByLabelText`, `disabled`, `aria-invalid`.
- **H2** the spinner keeps animating under reduced motion (07 §7.4.6 exception); the a11y assertion now requires `animationDuration != 0s`.
- **H3** the HC page mode replaces `?theme=` instead of appending a second value; the 176 HC goldens were regenerated under the real `[data-theme="high-contrast"]`.
- **H4** the High-Contrast mapping (both triggers) now defines `--accent-foreground: HighlightText`, `--danger*`/`--warning*`/`--success*`/`--info` → `CanvasText` (+ `--danger-foreground: Canvas`), and `--swatch-ring`/`--swatch-halo`.
- **H5 (ruling A)** Light-theme `--accent-text`+`--accent-subtle` overrides for CLR-07/04/02/08 (purple `#6A3A9A`/`#F0E6FA`, green `#1E5A3A`/`#D9F2E4`, orange `#9A3D00`/`#FDE8D9`, pink `#B02878`/`#FADCEC`), documented in 07 §7.4.1's alternatives paragraph; `contrast.test.ts` now asserts accent-text on bg-app/bg-panel/accent-subtle for both themes × all five accents (19 tests total, incl. focus-ring ≥3:1 and the `*-text` status pairs).
- **M5** ContextMenu opens via the ContextMenu key / Shift+F10 (focusable `role="button" aria-haspopup="menu"` target; keyboard test).
- **M6** the IconButton tooltip is the ICU message `components.iconButton.tooltip` (`{label} ({shortcut})`) — no concatenation.
- **M7** `i18n:check` validates ICU syntax itself, resolving the formatjs parser through `eslint-plugin-formatjs`'s pnpm scope (no new dependency). *(Superseded in round 2, L-04: the parser now resolves through `react-intl`'s scope — the exact version the runtime uses.)*
- **M8** stylelint bans color functional notation (`rgb()/hsl()/oklch()/…`) on color-bearing properties outside tokens.css (probed failing on `rgb(255 0 0)`); the eslint attribute-literal half stays at text-only mode (jsx-only also flags test fixtures and no per-file override applies) — product attribute strings verified clean and covered by the i18n used-but-missing scan. *(Superseded in round 2, M-04: jsx-only attribute mode landed via flat-config layering - a later config block turns the rule off for test fixtures; the plugin itself has no file-ignore option in 6.1.5.)*
- **M9** every icon usage takes `ICON_SIZE_DENSE`/`ICON_SIZE_TOOLBAR` + `ICON_STROKE_WIDTH` (16/20/1.75 per 07 §7.5; no 12/24 leftovers). *(The round-2 review found the claim premature — `KeyframeNavigator` still had `size={12}`; fixed as L-02 and now true.)*
- **M10** focus checks assert `outline-width: 2px` and the `--focus-ring` token color; the interaction selector covers `[role=treeitem|row|option]` so Tree/GridList/Table/ListBox get interaction goldens.
- **M11** zoom-extremes covers all five DPI steps (100/125/150/175/200) plus 80/200 extremes, and asserts computed token flips per theme/accent (SC-001/SC-002); High Contrast asserts `--bg-app: Canvas`.
- **M12** see H5 — full pair matrix in contrast.test.ts.
- **L1** fr `harness.stories.*` translated ("Scénarios"); **L2** the stories landmark's aria-label localizes; **L7** tasks.md a11y.ts→a11y.tsx; **L9** fonts.css uses `format("woff2") tech(variations)` notation; **L10** the dual `@playwright/test` declaration documented in the root package.json; **L11** `--clr-01..08-foreground` tokens added per the amended 12 §12.17.
- **CSS-module convention defect (found during the fix batch)**: Vite exports CSS-module keys as written, but components accessed kebab-case classes camelized (`styles.menuBarTrigger` from `.menu-bar-trigger`) — undefined in the served build while vitest's pipeline masked it, silently dropping every kebab-named class (menubar triggers, tree chevrons/groups, the spinner animation, focus hooks). `vite.config.ts` now sets `css.modules.localsConvention: "camelCaseOnly"`; all goldens regenerated against the correctly-styled UI. This also made the H2 spinner exception real (the animation class actually applies now; captures freeze frames with `animations: "disabled"`), and the stricter focus checks (M10) drove real focus-ring fixes across Selection (indicator `:has(input:focus-visible)` rings), SegmentedControl, Slider, ColorPicker/Wheel, and the RAC collection rows.
- **M5 rework**: the ContextMenu keyboard path is a visible kebab trigger inside the area (a `role="button"` wrapper is structurally nested-interactive because RAC renders the closed menu inline; axe `nested-interactive`); right-click still opens at the pointer, the kebab opens at the button, both through the same anchored MenuTrigger.

Deferred with owner-visible tracking (appended as Phase 10 tasks): M1 (windowing beyond ListBox + operable listbox semantics), M2 (compact density across all families), M3 (NumberField mixed-state editing), M4 (TimecodeField flicks API + spinbutton segments), L3 (harness render side-effects), L4 (toast dismiss/pause + assertive errors), L5 (autofocus eslint-disable), L6 (Tree selection semantics).

## Deviations / notes

- **Composed Tree (D6 amendment, owner ruling A 2026-11-06)**: RAC 1.22.1 `Tree` crashes real Chromium through its hidden-`<template>` collection machinery (`getOwnerDocumentFromRootContainer(...).createTextNode is not a function` in React 19 completeWork; reproduced on 19.2.1 and 19.3.1; GridList/Table/ListBox unaffected). `Tree`/`TreeItem` are therefore composed directly over APG tree semantics (role=tree/treeitem, arrow/Home/End keys, roving tabindex) while the rest of the collections stay on RAC.
- Font format deviations from research D9 (upstream publishes no variable WOFF2): Noto Sans Arabic and JetBrains Mono ship as variable TTF; recorded per-entry in `assets/fonts/fonts.json` `sourceUrl`.
- Light-theme values for the non-default accent triples remain the documented dark triples (canonical gap flagged in `tokens.css`; §7.4.1 note added for `--accent-foreground`).
- RAC 1.22.1 gaps worked around in-component: ColorSlider/ColorWheel thumb inputs are named via a RAC `<Label>` child (`aria-label` stays on the group and leaves the input unnamed); CurveEditor renders its graph inside a real `TabPanel` so the selected tab's `aria-controls` resolves; the ContextMenu anchor is a zero-size `<span aria-hidden="true">` (an anchor `<button>` without a name fails `button-name`); WindowedListBox's scroll container is the focusable `role="listbox"` (WCAG 2.1.1 scroll access).
- Convergence additions (Phase 9): `SegmentedControl` composes RAC `RadioField`/`RadioButton` (the bare `Radio` primitive is deprecated in 1.22); checkable menu items are `MenuItemCheckbox` (role=menuitemcheckbox via the render prop, D6-style — RAC ships none); `app/src/design/icons.ts` is the Lucide base-set barrel + the §7.5 conventions (16 px dense / 20 px toolbar / stroke 1.75) and every component imports through it; the nine `*.states` cards cover the FR-007 disabled/error states for the interactive families (Fields, Selection, Menu, Overlays, Tabs, Numeric, Color, Collections) — Feedback/Badge/EmptyState are display components without disabled/error states in their §7.6 contracts, covered by the page-mode matrix instead.


## Reviewer round 2 (2026-11-16) - verdict FAIL, all findings fixed same day

Report: `plan/tracking/reviews/WP-04-r2.md` (round 1 was delivered in-session before the file convention existed). Round-1 fix set verified present and correct by the reviewer. New findings: 3 HIGH, 7 MEDIUM (+2 already tracked as T046/T048), 7 LOW (+2 tracked). Owner rulings before fixing (2026-11-16):

- **H-03 ruling A**: `--border-strong` darkened to `#6C6D72` (3.44:1 panel / 3.04:1 raised) and light `#8D9098` (3.19:1) - previously 1.59:1/1.90:1 while 07 s7.4.1 annotated 3.3:1 and `contrast.test.ts` exempted borders as decorative. Both doc tables amended; the exemption comment replaced by explicit border-on-panel/raised assertions.
- **H-01 ruling A**: new `--accent-subtle-foreground` (equals `--accent-text` via lazy indirection; `HighlightText` in both HC triggers) - previously both HC tokens mapped to `Highlight`, so selected/hovered surfaces painted Highlight-on-High (1:1, invisible labels in the user-selectable HC theme). Six subtle-fill rules swapped; s7.4.1 gains the token row, s7.4.3 states the pairing; `contrast.test.ts` asserts both HC mappings and the runtime `HighlightText` resolve.

Fixes (atomic commits, verified individually):

- **H-02** `7c1d288` - `--motion-fast` never existed (120ms literal fallback survived reduced motion on the tree chevron + switch thumb); now `--dur-fast`+`--ease-standard`, and the a11y suite asserts zero non-zero transition durations under the suite's reduce emulation on both cards.
- **H-01** `6bdcecd`, **H-03** `766bbd3` - as ruled above.
- **M-01 + L-01 (+2 found in class)** `0c42d9f` - `--shadow-overlay` and `--bg-overlay` were never defined: every menu/dialog/toast/tooltip/popover shipped transparent with no elevation (a var() to an undefined custom property invalidates the declaration at computed-value time); `--menu-min-width` rode a literal fallback. Overlays now use `--bg-raised` (s7.4.1: menus are raised surfaces) + `--shadow-popover`/`--shadow-dialog` (s7.4.6); `--radius-xs` fallbacks collapse to `--radius-sm`. New `token-hygiene.test.ts` fails on any var() reference to an undefined custom property (0 undefined over 100 defined, with an anti-vacuity sanity assertion).
- **L-02** `7381ba2` - keyframe diamond 12px/default-stroke -> `ICON_SIZE_DENSE`/`ICON_STROKE_WIDTH`.
- **M-02** `d5ea2f9` - 07 s7.4.1: the duplicated light-override sentence (pink `#C03088` at 4.11:1, contradicting the kept `#B02878`/`#FADCEC`) removed; stale light tertiary annotation 5.3:1 -> measured 5.1:1 (the round-2 reviewer's 4.70:1 was itself wrong - the project formula measures 5.17:1); tokens.css stale canonical-gap comment updated.
- **M-04** `a16c337` - jsx-only attribute-literal mode landed: product components (src/components, src/i18n) run `i18next/no-literal-string` in jsx-only with plugin-default + enum-valued API-prop attribute excludes (probe-driven); test fixtures opt out via a later flat-config block. The round-1 "no clean override" claim was wrong - the plugin (6.1.5, schema verified) has no file-ignore option, but flat-config layering provides it.
- **L-06** `7619f7f` (owner approved) - `@types/node` 26.6.6 (published 2026-10-28, matches the Node 26 runtime) added to the docs/00 s0.8 baseline; TypeScript 6 no longer auto-includes node_modules/@types so both tsconfigs set `types:["node"]`; all 16 `@ts-expect-error` suppressions removed and the eslint `disableTypeChecked` exemption block deleted - type-aware linting then surfaced and fixed 12 real issues it had masked.
- **L-04 + L-05** `e1ddb9a` + `fcccf37` - the ICU parser resolves through `react-intl`'s scope (declared dependency; exactly the runtime's `@formatjs/icu-messageformat-parser 3.5.22`); the vitest suite feeds the REAL source scan into `collectIssues` (the unused-key case was vacuous) and `collectUsedIds` gets a fixture test (literal ids found, dynamic ignored; non-namespaced fixture ids stay inert under the checker's prefix filter).
- **M-05/M-06/M-07** `eaecb7c` - the focus-ring check proves ownership (the focused control's own outline, its label visuals - RAC composes inputs inside their label - or immediate wrappers ringed via `:has(input:focus-visible)`, wrappers own-outline only so the scan cannot widen to the card; keyboard modality primed with Tab because rings are :focus-visible-gated). Zoom asserted through the computed `zoom` on #root; accents assert the exact s12.17 fills. axe gains `wcag22a`/`wcag22aa` - zero violations across 44 stories x 4 projects.
- **L-07/L-08/L-09** `29ae9ab` - dead `MODES[].url` dropped; FNT-02 `sourceUrl` points at the exact raw file (percent-encoded brackets); STATUS.md records PR #11; stale "35 stories" comment corrected.
- **Goldens** `0096475` - 76 of 1,224 regenerated (shape unchanged: 528 matrix + 504 interactions + 176 forced-colors) after one `--update-snapshots` pass; the clean run passes over the new set.

Round-2 verification: `pnpm -C app test` 158/158; `pnpm -C app harness` 68 passed over 1,224 goldens (regen then clean run); `pnpm -C app i18n:check` 125 keys; `pwsh -File tools/verify.ps1` all 13 gates PASS; `python tools/docs/check_docs.py` PASS.

Still deferred (owner-tracked Phase 10, unchanged): T043-T050. The reviewer's M-09 warning stands: T046 (TimecodeField flicks API) should land before WP-05 binds the frames-based signature.



## Reviewer round 3 (2026-11-20) - verdict FAIL, all seven findings fixed same day

Report: `plan/tracking/reviews/WP-04-r3.md`. The round-2 batch was verified genuine (H-02/H-03 source-fixed with regression-proof assertions, M-01/M-04..M-07 and every LOW landed); the blocker was a residual of round-2 H-01 plus the harness gap that let it slip:

- **H-01 residual (HIGH)** `c6766fa` - the round-2 fix covered six of nine `--accent-subtle` fills. The tree item hover/focused rule still painted `--accent-text` on the subtle fill (Highlight-on-High in manual HC) and the table focused row / windowed option hover kept inherited CanvasText on a Highlight fill. All three now take `--accent-subtle-foreground` (identical to `--accent-text` in dark/light, so only the two previously-unpaired rules changed appearance there).
- **M-02** `c6766fa` - the harness never rendered an HC *interaction* state (HC was static-only; hover/focus ran in project themes) - exactly how H-01 survived round 2. `zoom-extremes.spec.ts` now hovers the real tree item and windowed option on the HC page and asserts fill and label resolve to the two *distinct* system colors through an in-page Highlight/HighlightText probe (Chromium computes system keywords to rgb; the probe makes the check palette-independent; it must run after navigation - a blank-page probe resolves a different palette). The table's `data-focused` rule cannot be driven until selection mode ships (RAC rows are inert - tracked T043/T049) and stays sheet-covered in contrast.test.ts.
- **M-01** - evidence `:26` still said 115 keys/15 tests (round-2 M-03 half-fixed): now 130 keys x 6 locales, 16 tests, with the real-scan note.
- **L-01** `76260fc` - six hard-coded harness accessible names (Media/Bin/Tracks/Windowed/Inspector x2) localized through `formatMessage` with five new keys in all six locales + meta (anchor-ordered insert; `i18n:check` 130 keys green).
- **L-02** (no code) - the collections have no `[data-selected]` styling yet (latent until selectionMode; the tree item is hard-wired `aria-selected={false}`). It rides with T043/T049, which own collection selection semantics.
- **L-03** - "byte-stable"/"byte-equality" determinism claims corrected to the threshold-comparator reality in evidence.md and quickstart.md.
- **L-04** `eea844d` - tokens.test.ts now parses both HC blocks (forced-colors media + manual theme, the selector anchored with its brace because the header comment mentions it verbatim) and asserts identical name-to-value maps (~30 tokens, previously only 3 were checked).

Round-3 verification: `pnpm -C app test` 159/159; `pnpm -C app i18n:check` 130 keys; `pnpm -C app harness` 72 tests over 1,224 goldens (regen pass then clean run; the test count grew by the HC-pairing test across the four projects); `pwsh -File tools/verify.ps1` all 13 gates PASS.

Deferral list unchanged (T043-T050); with round-3 L-02 the collections `[data-selected]` styling is explicitly recorded to land with T043/T049.

