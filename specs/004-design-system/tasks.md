---
description: "Task list for WP-04 — Design system and UI foundation"
---

# Tasks: WP-04 — Design system and UI foundation

**Input**: Design documents from `/specs/004-design-system/` (plan.md, spec.md, research.md D1–D16, data-model.md, contracts/design-system-apis.md, quickstart.md)

**Prerequisites**: plan.md (owner-approved 2026-11-05 incl. the F03/F05/F08 rulings), spec.md (5 stories P1–P5)

**Tests**: Included — the spec's acceptance (T-UI-001/002, T-I18N-003/004, T-A11Y-004 + the snapshot matrix) and FR-007/FR-017 require them; Vitest suites are deterministic (jsdom, no wall-clock), Playwright snapshots deterministic per research D7 (fixed viewport 1280×720, `reducedMotion: 'reduce'`, `document.fonts.ready` awaited).

**Organization**: Phases 1–2 are shared infrastructure (dependencies/configs, then the token sheet, locale seed, and harness shell every story renders through); Phases 3–7 are the user stories in priority order; Phase 8 is polish. Every new file carries the SPDX header (02 §2.14).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US5 per spec.md; setup/foundational/polish carry no label
- File paths are repository-relative; `app/…` is the existing Tauri app skeleton

## Path Conventions

- UI sources: `app/src/{design,components,i18n,locales,harness}/`
- Harness/tests: `app/src/harness/`, `tests/ui/` (Playwright), component tests co-located `<Name>.test.tsx`
- Assets: `assets/fonts/`, `NOTICES.md` (repo root)
- Workspace: root `pnpm-lock.yaml` (no per-package lockfile — research D1/F04)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The dependency set, enforcement stack, and test pipelines — including the four `package.json` scripts that activate the `verify.ps1` UI gates.

- [X] T001 Update `app/package.json` with the D1 dependency set (runtime: `react-aria-components` 1.22.x, `react-intl` 12.x, `zustand` 5.x, `lucide-react` 1.x; dev: `vitest` 5.x, `@testing-library/react` 16.x, `@testing-library/dom` 10.x, `jsdom` 30.x, `@playwright/test` 1.64.x, `@axe-core/playwright` 4.x, `eslint` 9.39.x, `typescript-eslint` 8.x, `eslint-plugin-jsx-a11y` 6.x, `eslint-plugin-react-hooks` 7.x, `eslint-plugin-i18next` 6.x, `eslint-plugin-formatjs` 8.x, `stylelint` 17.x, `stylelint-config-standard` 40.x, `stylelint-use-logical` 2.x, `prettier` 3.x, `@formatjs/cli` 6.x) and the scripts `typecheck` (`tsc --noEmit`), `lint` (ESLint + Stylelint + `prettier --check`), `test` (Vitest run), and `harness` (Playwright) — the `i18n:check` script is added in T007 together with its checker, so no gate activates before its implementation is green (analysis H1); install through the root pnpm workspace and commit the updated root `pnpm-lock.yaml`; add the Playwright browser build-scripts approval to the root `package.json` `pnpm.onlyBuiltDependencies` (research D15); `pnpm -C app typecheck` green on the skeleton
- [X] T002 [P] Create `app/eslint.config.js` (flat config: `typescript-eslint` strict-type-checked, `react-hooks`, `eslint-plugin-jsx-a11y` errors, `eslint-plugin-i18next/no-literal-string` errors, `eslint-plugin-formatjs`), `app/.stylelintrc.json` (`stylelint-config-standard` + `stylelint-use-logical` — physical properties are errors), and `app/.prettierrc.json`; `pnpm -C app lint` green on the skeleton (02 §2.14)
- [X] T003 [P] Wire Vitest in `app/vite.config.ts` (jsdom environment, setup file `app/src/test/setup.ts` with Testing Library cleanup) plus a smoke test `app/src/test/smoke.test.tsx`; `pnpm -C app test` green
- [X] T004 [P] Scaffold the Playwright harness: `tests/ui/playwright.config.ts` (Chromium, viewport 1280×720, `reducedMotion: 'reduce'`, four projects = dark/light × en/ar via URL params), a trivial `tests/ui/harness.spec.ts` snapshot proving the pipeline, and the `pnpm -C app harness` script wiring (vite dev server → Playwright); `pnpm -C app harness` green
- [X] T005 Extend `.github/workflows/ci.yml` (research D15): the `ui` job gains `pnpm -C app typecheck && pnpm -C app lint && pnpm -C app test && pnpm -C app i18n:check` after the existing build + license check, plus a `ui-harness` job running the Playwright suite with browser caching (G-08)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The three substrates every story renders through: the verbatim token sheet, the seeded locale catalogs, and the harness shell.

**⚠ CRITICAL**: No user story work begins until this phase is complete.

- [X] T006 Create `app/src/design/tokens.css` with the complete 07 §7.4 token set verbatim — dark (§7.4.1) and light (§7.4.2) tables under `[data-theme="dark"|"light"]`, the five accent blocks `[data-accent="clr-06|clr-07|clr-04|clr-02|clr-08"]` swapping `--accent-solid/--accent-text/--accent-subtle` with the §7.4.1/§12.17 values, the High Contrast system-color mapping under BOTH `@media (forced-colors: active)` AND `[data-theme="high-contrast"]` (owner ruling F05; identical mapping), timeline/media colors per theme (§7.4.4), typography stacks/sizes/weights with the `:lang(ar)` line-height 1.6 + 1 px adjustment (§7.4.5), and space/shape/elevation/motion incl. reduced-motion zeroing (§7.4.6); unit test `app/src/design/tokens.test.ts` asserting both theme blocks define the identical documented token inventory (no token present in one theme and missing in the other)
- [X] T007 [P] Seed the locale catalogs `app/src/locales/{en,ar,fr,es,de,pt-BR}.json` + `app/src/locales/en.meta.json` with the initial key set (harness chrome, theme/appearance labels; IDs `area.component.purpose` per 07 §7.12 rule 1) — identical key sets in all six files, one meta description per key; and create `app/src/i18n/i18n-check.mts` with the `i18n:check` package.json script implementing its first real slice — deep key-set equality across the six catalogs + one-meta-description-per-key — so the gate activates green in the same commit it is defined (analysis H1); component keys are added per component task in Phase 5, and T014 extends the checker to the full docs/08 §8.4 gate
- [X] T008 Create the harness shell: `app/src/harness/HarnessApp.tsx` (story grid reading `?theme=dark|light|high-contrast&locale=<tag>&density=comfortable|compact`, landmarks per §7.13 rule 2), `app/src/components/a11y.ts` (polite + assertive live regions mounted at the root, announcement helpers — §7.13 rule 3), `app/src/harness/mock/` skeleton (typed fixture factories over `app/src/ipc/bindings` shapes, stub event bus; research D11), and rewire `app/src/App.tsx` to mount the harness with `app/src/main.tsx` importing tokens.css; update `tests/ui/harness.spec.ts` so the T004 smoke snapshot captures the real shell (analysis L1)

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 — Themes, accents, and zoom that apply instantly (Priority: P1) ⭐ MVP

**Goal**: Dark/Light/High-Contrast + five accents switching without restart, zoom 80–200% (07 §7.4; 12 §12.17).

**Independent Test**: `pnpm -C app test -- t_ui_001 t_ui_002` — live token switching, accent swaps, zoom clamp.

### Tests for User Story 1 (write first; must fail before implementation)

- [X] T009 [P] [US1] Create `app/src/design/theme.test.ts` as the `t_ui_001` suite: switching dark↔light↔high-contrast swaps `data-theme` on `<html>` without reload and re-resolves token values; each accent swaps the three accent tokens; `forcedColorsActive()` reflects the media query; and `t_ui_002` assertions: `setZoom` clamps to 80–200 and applies `--app-zoom` (integer) with the root computing `zoom: calc(var(--app-zoom) * 1%)` (research D3 — multiplication, never division)

### Implementation for User Story 1

- [X] T010 [US1] Create `app/src/design/theme.ts` (Zustand `themeStore` per contracts §1: `theme: 'dark'|'light'|'high-contrast'` default dark, `accent` default `clr-06`, `zoom` default 100, setters applying `data-theme`/`data-accent`/`--app-zoom` on `<html>` immediately, `forcedColorsActive()`, and the swappable persistence seam — harness binds URL params, WP-05 binds `alvs.appearance.*`); make T009 green
- [X] T011 [US1] Add the theme/zoom controls to the harness toolbar (`app/src/harness/HarnessApp.tsx`): theme/accent/zoom switchers labeled with locale keys (added to all six catalogs + meta), proving no-restart switching interactively and seeding the T-UI-001 visual path

**Checkpoint**: US1 independently functional (MVP: the token/theme/zoom foundation).

---

## Phase 4: User Story 2 — One runtime, six languages, correct direction (Priority: P2)

**Goal**: react-intl runtime, runtime language switch, RTL per 07 §7.11, ICU + `latn` formatting per §7.12, key-set CI.

**Independent Test**: `pnpm -C app i18n:check` + `pnpm -C app test -- t_i18n_003 t_i18n_004`.

### Tests for User Story 2 (write first)

- [X] T012 [P] [US2] Create `app/src/i18n/runtime.test.tsx` as the `t_i18n_003` suite: Arabic ICU plurals (zero/one/two/few/many/other) render correctly; `Intl` numbers/dates use the active locale with `numberingSystem: "latn"`; timecode/frame text is wrapped in an LTR isolate inside RTL prose; language switch re-renders without reload and defaults to the Windows display language when supported else `en`
- [X] T013 [P] [US2] Create `app/src/i18n/i18n-check.test.ts` as the `t_i18n_004` suite: the check fails on a locale file with a key missing from (or extra to) `en.json`, fails on invalid ICU syntax, fails on an unused key, and passes on the real six catalogs; plus the LOC-90 expansion (+40%, brackets) and LOC-91 (RTL, mirrored, bidi marks — all three, 12 §12.14) behavior assertions

### Implementation for User Story 2

- [X] T014 [US2] Extend `app/src/i18n/i18n-check.mts` (created in T007) to the full docs/08 §8.4 localization gate: ICU syntax validation of every message via the `@formatjs/cli` parser + unused-key detection (message IDs extracted from `app/src/**` vs catalogs) on top of T007's key-set equality and meta coverage (research D5; analysis H1)
- [X] T015 [US2] Create `app/src/i18n/runtime.tsx` (static six-locale registry, `<I18n>` IntlProvider wrapper, `localeStore`, `resolveDefaultLocale`) and `app/src/i18n/direction.ts` (sets `dir`/`lang` on `<html>`; helpers for the §7.11 non-mirrored surfaces setting `dir="ltr"` with a citing comment) and `app/src/i18n/format.ts` (`latn` numbers/dates, `ltrIsolate`); wire the harness locale switcher; make T012 green
- [X] T016 [US2] Create `app/src/i18n/pseudo.ts` (generate `en-XA` accented +40% bracketed and `ar-XB` RTL+mirrored+bidi-marks from the real catalogs, test/harness builds only — research D14); make T013 green including the deliberate-drift failure cases and the zoom-extremes edge case: LOC-90 labels rendered at zoom 80 and 200 wrap or truncate with a tooltip — never clip silently (spec Edge Cases; analysis M2)

**Checkpoint**: US1+US2 independently functional.

---

## Phase 5: User Story 3 — An accessible component library, proven in the harness (Priority: P3)

**Goal**: All 41 components of 07 §7.6 on React Aria Components (composed where RAC ships no primitive — MenuBar per D6), tokens/logical-properties styling only, nine harness story states each, locale keys ×6.

**Independent Test**: `pnpm -C app test` (component suites) + snapshots rendering for every story; axe clean per story (full matrix run lands in Phase 7).

### Implementation for User Story 3 (each task: `<Name>.tsx` + `<Name>.module.css` + `<Name>.test.tsx` in `app/src/components/`, its story file in `app/src/harness/stories/`, and its keys added to all six catalogs + `en.meta.json`; behavior per contracts §3)

- [X] T017 [P] [US3] Buttons: `Button.tsx` (primary/secondary/ghost/danger; primary once per dialog; danger pairs with confirmation semantics), `IconButton.tsx` (icon + 500 ms tooltip carrying label+shortcut + accessible name — REQ-HLP-003), `ToggleButton.tsx`, `SegmentedControl.tsx` (RadioGroup-composed)
- [X] T018 [P] [US3] Menu family in `Menu.tsx`: `Menu`/`Submenu`/`ContextMenu` on RAC MenuTrigger and `MenuBar` as a `role="menubar"` composition with the ARIA menubar keyboard pattern (RAC 1.22 ships no MenuBar — research D6); shortcuts right-aligned (left in RTL)
- [X] T019 [P] [US3] Overlays in `Dialog.tsx`/`Tooltip.tsx`/`Toast.tsx`: Dialog/AlertDialog (focus trap, initial focus on the safest control, Esc except unsaved-changes prompts, focus return), Popover, Tooltip (500 ms), Toast (region + polite announcement)
- [X] T020 [P] [US3] `Tabs.tsx` (panel-tab primitive; dockview integration stays WP-05)
- [X] T021 [P] [US3] `NumberField.tsx` — scrubbable (drag, Shift ×10, Ctrl ×0.1), wheel-when-focused, arrows, typed math (`+10`, `*2`), unit suffix, mixed state "—"
- [X] T022 [P] [US3] `Slider.tsx` with `AngleDial` (paired NumberField, double-click reset; dial per 07 §7.6)
- [X] T023 [P] [US3] `TimecodeField.tsx` — segmented `HH:MM:SS:FF`, typed + `+`/`−` relative entry, always `dir="ltr"` (§7.11)
- [X] T024 [P] [US3] `KeyframeToggle.tsx` + `KeyframeNavigator.tsx` (stopwatch state; ◀ ◆ ▶ enabled states)
- [X] T025 [P] [US3] `ColorPicker.tsx` (HSV square + hue slider + hex/RGB fields + recent colors; eyedropper is a later package's viewer integration) and `ColorWheel.tsx` (chroma puck + luma slider, Alt-drag fine control, double-click reset)
- [X] T026 [P] [US3] `CurveEditor.tsx` (SVG points: click-add, double-click/Delete remove, drag, channel tabs, reset) and `PointControl.tsx` (numeric x/y + handle slot)
- [X] T027 [P] [US3] `Fields.tsx`: SearchField, TextField, TextArea — `dir="auto"` for user content (§7.11)
- [X] T028 [P] [US3] `Selection.tsx`: Checkbox, Switch, RadioGroup, Select, ComboBox
- [X] T029 [P] [US3] `Collections.tsx`: Tree, GridList, Table, ListBox + the `useWindowing` hook (custom, no new dependency — research D10) virtualizing beyond 200 rows with a 200/201-row boundary test
- [X] T030 [P] [US3] `Feedback.tsx` (ProgressBar, Meter, Spinner — indeterminate only before the first progress value — Skeleton), `Badge.tsx` (text + icon state chips: Proxy, Offline, 10-bit, HDR, VFR; never color alone), and `EmptyState.tsx` (icon, one sentence, primary action, shortcut hint)

**Checkpoint**: all seven stories independently functional — every 07 §7.6 component exists with stories in all nine states.

---

## Phase 6: User Story 4 — Fonts that cover the UI in every shipped language (Priority: P4)

**Goal**: FNT-01..03 bundled with licenses and a hashed manifest; Arabic type adjustments live.

**Independent Test**: `pnpm -C app test -- fonts` (integrity) + harness snapshots rendering Latin/Arabic/mono through the bundled faces.

- [X] T031 [P] [US4] Bundle `assets/fonts/` — Inter variable (FNT-01), Noto Sans Arabic variable (FNT-02), JetBrains Mono (FNT-03) as separate files from their official upstream repositories, each family's `OFL.txt` beside them, and `assets/fonts/fonts.json` recording `{id, family, file, sourceUrl, sha256, licenseFile}` per 12 §12.16; verify `NOTICES.md` rows match the bundle
- [X] T032 [US4] Create `app/src/design/fonts.css` (`@font-face` for the three families, `font-display: swap`, wired into `main.tsx` after tokens.css) and the integrity test `app/src/design/fonts.test.ts` re-hashing every file against `fonts.json` and asserting each license file exists (runs in `pnpm -C app test` — research D9/F10); snapshots now render with the real faces (`document.fonts.ready` awaited per D7)

**Checkpoint**: US1–US4 independently functional.

---

## Phase 7: User Story 5 — The UI harness and the rules that keep it clean (Priority: P5)

**Goal**: The full snapshot matrix + axe suite (T-A11Y-004, SC-006) and the gates green locally and in CI.

**Independent Test**: `pnpm -C app harness` green; `pwsh -File tools/verify.ps1` all gates PASS (UI gates now executing).

- [X] T033 [US5] Create `tests/ui/snapshots.spec.ts` — every component × nine states × dark/light × en/ar via the four Playwright projects (URL-driven modes per D7), each capture preceded by `document.fonts.ready` and a repeat-run determinism assertion; and `tests/ui/hc.spec.ts` forcing `forced-colors: active` through a CDP session (`Emulation.setEmulatedMedia` — Playwright `colorScheme` does not cover it, research D7) for the High Contrast stories
- [X] T034 [US5] Create `tests/ui/a11y.spec.ts` as the `t_a11y_004` suite: per-story `@axe-core/playwright` scans (zero violations), focus-visible ring checks, reduced-motion zero-duration assertions, and badge/label text+icon (never-color-alone) checks; contrast ratios in dark/light asserted from the token values in a Vitest companion `app/src/design/contrast.test.ts` (4.5:1 text / 3:1 components — REQ-A11Y-003)
- [X] T035 [US5] Gate closure: `pwsh -File tools/verify.ps1` fully green with `ui-typecheck`, `ui-lint`, `ui-test`, `i18n` executing (no longer SKIPPED), the rust gates untouched and green, and the `ui`/`ui-harness` CI jobs green (T005 wiring verified end-to-end)

**Checkpoint**: all five stories functional; the package's gates exist and pass.

---

## Phase 8: Polish & Cross-Cutting Concerns

- [X] T036 [P] Final enforcement sweep: every new file carries the SPDX header; `pnpm -C app lint` fails a deliberately raw color / physical property / JSX literal / `any` (proving the rules bite — contracts G6); remove the deliberate probe afterwards
- [X] T037 Run the full `specs/004-design-system/quickstart.md` validation (all six acceptance rows + `tools/verify.ps1` + `python tools/git/check_commits.py origin/main..HEAD`) and record commands/results into `specs/004-design-system/evidence.md` (constitution III; G-09), including the snapshot-matrix counts and any environment notes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 first (deps + scripts); T002/T003/T004 parallel after it; T005 after T001. **Phase 1 lands as ONE green commit** (T001–T005): the scripts activate their gates the moment they are defined, so no intermediate commit may be gate-red (analysis M1)
- **Foundational (Phase 2)**: after T001–T004; T006/T007 parallel, T008 after T006 (the shell imports tokens.css); **blocks all stories**
- **Stories (Phases 3–7)**: priority order for a single implementer (US1 → US2 → US3 → US4 → US5); US3 depends on US1 (tokens) and US2 (direction/labels); US4 is independent of US3 (can parallel); US5's full matrix needs US3+US4 complete
- **Polish (Phase 8)**: after all stories

### User Story Dependencies

- **US1 (P1)**: foundation only — MVP
- **US2 (P2)**: foundation; the harness locale switcher (T015) composes with US1's toolbar
- **US3 (P3)**: foundation + US1 tokens + US2 direction/labels; components are file-disjoint [P]
- **US4 (P4)**: foundation only (independent of US2/US3; snapshots sharpen once T032 lands)
- **US5 (P5)**: all stories (the matrix snapshots every component with real fonts)

### Parallel Opportunities

- T002/T003/T004 (configs/pipelines, file-disjoint)
- T006/T007 (tokens.css vs locale seed)
- T012/T013 (US2 test files)
- T017–T030 (the 14 component clusters are file-disjoint — the largest parallel batch)
- T031 parallel with Phase 5 (fonts vs components)

## Parallel Example: User Story 3

```text
Task: "Buttons cluster in app/src/components/Button.tsx…"      (T017)
Task: "Menu family in app/src/components/Menu.tsx"             (T018)
Task: "Overlays in app/src/components/Dialog.tsx…"             (T019)
# …all fourteen clusters are file-disjoint and [P]-marked
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 + Phase 2 → deps, gates, tokens, locale seed, harness shell
2. Phase 3 (US1) → **STOP and VALIDATE**: `pnpm -C app test -- t_ui_001 t_ui_002` green; the harness switches themes live
3. Continue P2→P5 in order, validating each story's filter before the next

### Incremental Delivery

Each story lands with its suites green and a commit (Conventional Commits + DCO); `/wp-verify` runs after each stage per the card protocol.

---

## Notes

- Every user-visible string is a message ID in all six locales + `en.meta.json` at the moment its component lands (no key debt; `i18n:check` green at every commit)
- Styling: CSS Modules with tokens and logical properties only — Stylelint errors on violations (02 §2.14)
- The four gate scripts are what `tools/verify.ps1` probes; do not rename them — and a gate script never appears before its implementation is green in the same commit (`i18n:check` lands with the T007 checker; analysis H1)
- Playwright determinism rules (D7) apply to every snapshot: viewport 1280×720, `reducedMotion: 'reduce'`, fonts awaited, animations off
- No `dockview`, no virtualization library, no HTTP client, no new dependency outside the baseline rows (G-11)
- Commit after each task or logical group; stop at any checkpoint to validate a story independently
