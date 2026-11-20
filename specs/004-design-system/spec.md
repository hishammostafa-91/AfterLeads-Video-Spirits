# Feature Specification: WP-04 — Design system and UI foundation

**Feature Branch**: `feat/004-design-system`

**Created**: 2026-11-04

**Status**: Draft

**Input**: User description: "Implement WP-04 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: REQ-APP-009, REQ-APP-010, REQ-I18N-001 (infrastructure), REQ-I18N-002 (infrastructure), REQ-I18N-003, REQ-I18N-004 (infrastructure), REQ-I18N-007 (UI fonts), REQ-A11Y-002 (components), REQ-A11Y-003, REQ-A11Y-004, REQ-HLP-003 (components). Acceptance tests: T-UI-001, T-UI-002, T-I18N-003, T-I18N-004, T-A11Y-004, plus harness snapshots for all components in dark/light × en/ar. Read only the sections in the card's Read line. Derive every requirement from docs/01-product.md and cite IDs; add nothing that is not in the canonical documents."

**Canonical basis**: WP-04 card (docs/10 §10.7). Requirements cited from docs/01-product.md rows (quoted where binding; the "(infrastructure)"/"(components)" markers in the card scope WP-04's share — the rest of each row belongs to WP-05/WP-17/WP-25 per the rows' WP columns). Design authorities from the card's Read line: 07 §7.1, §7.4–7.6, §7.11–7.13; 02 ADR-011 and §2.14; 12 §12.14, §12.16, §12.17. Acceptance test IDs from the card's Acceptance row. Nothing is added beyond those documents (G-01, G-02, constitution I).

## User Scenarios & Testing *(mandatory)*

Actors: the component library, tokens, and i18n runtime that every later UI work package (WP-05 shell, WP-08 timeline, WP-24 settings, WP-25 polish) assembles; and through them the end user of REQ-APP-009/010, REQ-I18N-001–004/007, REQ-A11Y-002–004, REQ-HLP-003. This package is the product's visual and linguistic foundation: every pixel, label, direction, and interaction primitive stands on it.

### User Story 1 - Themes, accents, and zoom that apply instantly (Priority: P1)

The user picks Dark (default), Light, or High Contrast and an accent color; switching applies immediately, without restart. High Contrast also activates automatically when a Windows contrast theme is on (`forced-colors`), mapping tokens to system colors. The palette is fully tokenized — dark, light, high-contrast, and the timeline/media colors — so no component ever uses a raw color. Zoom scales the whole UI 80–200% and stays crisp on per-monitor DPI at 100/125/150/175/200% (REQ-APP-009, REQ-APP-010; 07 §7.4; 12 §12.17).

**Why this priority**: every later work package consumes these tokens; REQ-APP-009/010 are P0 requirements, and the contrast, focus, and elevation rules that REQ-A11Y-003/004 verify are properties of the token set itself.

**Independent Test**: T-UI-001 suite — theme and accent switching without restart; T-UI-002 — zoom and DPI crispness (card Acceptance row).

**Acceptance Scenarios**:

1. **Given** the running UI in Dark, **When** the user switches to Light, **Then** every token-driven surface changes without a restart and text/UI-component contrast still meets WCAG 2.2 AA (4.5:1 / 3:1).
2. **Given** a Windows contrast theme active, **When** the UI loads (or the theme turns on), **Then** tokens map to system colors (backgrounds `Canvas`, text `CanvasText`, accents/selection `Highlight`/`HighlightText`, …) and no hardcoded color leaks through.
3. **Given** any theme, **When** the user selects one of the accent alternatives (12 §12.17: CLR-06 default, CLR-07, CLR-04, CLR-02, CLR-08), **Then** the three accent tokens swap per 07 §7.4.1 without restart.
4. **Given** zoom set anywhere in 80–200%, **When** the UI renders at 100/125/150/175/200% monitor scaling, **Then** text and icons (SVG, variable fonts, `tabular-nums`) stay crisp.

---

### User Story 2 - One runtime, six languages, correct direction (Priority: P2)

The i18n runtime loads `en` plus `ar`, `fr`, `es`, `de`, `pt-BR` (LOC-01..06); the language changes at runtime without restart; the default follows the Windows display language when supported, else English. Arabic gets a mirrored RTL layout implemented with logical CSS properties, while the timeline, ruler, transport, and time never mirror; timecodes and frame numbers stay Western digits in an LTR isolate. Messages use ICU MessageFormat (including Arabic's zero/one/two/few/many/other plurals); numbers and dates use locale formatting with `numberingSystem: "latn"` (REQ-I18N-001/002/004 infrastructure, REQ-I18N-003; 07 §7.11–7.12; 12 §12.14).

**Why this priority**: direction and message handling shape component markup itself — the library must be bilingual (logical properties, `dir="auto"` content, LTR isolates) from the first component, not retrofitted.

**Independent Test**: T-I18N-003 — ICU formatting, locale numbers/dates, LTR-isolated timecodes; T-I18N-004 — key-set equality across all six locales and pseudo-locale behavior (card Acceptance row).

**Acceptance Scenarios**:

1. **Given** the runtime in English, **When** the user switches to Arabic, **Then** `dir="rtl"` and `lang` apply without restart, the layout mirrors, and directional icons mirror — while the timeline axis, transport, timecode, and numeric fields stay LTR (07 §7.11).
2. **Given** a message with a count, **When** it renders in Arabic, **Then** ICU plural categories (zero/one/two/few/many/other) select correctly; dates and numbers follow the active locale with Western digits.
3. **Given** any locale file with a missing or extra key relative to `en.json`, **When** the key-set check runs, **Then** it fails (REQ-I18N-004).
4. **Given** the pseudo-locales LOC-90 (`en-XA`, +40% length) and LOC-91 (`ar-XB`, RTL bidi), **When** tests render components, **Then** layouts survive expansion and bidi without clipping or overflow errors (07 §7.12 rule 5).

---

### User Story 3 - An accessible component library, proven in the harness (Priority: P3)

Every interactive primitive of 07 §7.6 is built on React Aria Components (keyboard, focus, ARIA, RTL handled), styled exclusively with tokens and logical properties: buttons/icon buttons/toggles/segmented controls, menus and context menus, tabs, dialogs/popovers/tooltips/toasts, the scrubbable NumberField, sliders/dials, color picker/wheel, curve editor, point control, the segmented LTR TimecodeField, keyframe toggle/navigator, text fields, selection controls, tree/grid/table/list (virtualized beyond 200 rows), progress/meter/spinner/skeleton, badges (text + icon, never color alone), and EmptyState. Screen-reader semantics — names, roles, values, states — plus live-region infrastructure (polite for edits/jobs, assertive for errors) come with the primitives. Icon-only buttons carry a tooltip with label and shortcut. Each component ships harness stories for default, hover, focus-visible, pressed, disabled, error, RTL, compact density, and High Contrast, with snapshots in dark/light × en/ar and axe checks (REQ-A11Y-002 components share, REQ-HLP-003 components share; 07 §7.5–7.6, §7.13).

**Why this priority**: this is the library itself — the largest deliverable; it depends on Story 1's tokens and Story 2's direction rules, and everything after WP-04 is assembled from it.

**Independent Test**: harness snapshots for all components in dark/light × en/ar (card Acceptance row) plus T-A11Y-004 — contrast, forced-colors, reduced-motion, and never-color-alone rules.

**Acceptance Scenarios**:

1. **Given** any component in the harness, **When** it is driven by keyboard only, **Then** every action is reachable, focus is visible via `--focus-ring` on `:focus-visible`, and screen readers get name/role/value/state.
2. **Given** the component stories, **When** snapshots run in dark and light × `en` and `ar`, **Then** every story renders correctly and axe reports no violations.
3. **Given** an icon-only button, **When** it is hovered/focused, **Then** a tooltip (500 ms delay) shows its label and shortcut, and the button has an accessible name.
4. **Given** Windows "Animation effects" off (`prefers-reduced-motion`), **When** components animate, **Then** non-essential motion is disabled (durations 0, progress indicators excepted) and no meaning is carried by color alone (REQ-A11Y-004).

---

### User Story 4 - Fonts that cover the UI in every shipped language (Priority: P4)

The UI fonts — Inter Variable (Latin UI), Noto Sans Arabic Variable (Arabic UI), JetBrains Mono (timecode/numeric readouts, `tabular-nums`) — are bundled under `assets/fonts/` as separate files with their SIL OFL 1.1 license files and recorded in `assets/fonts/fonts.json` (source URL + SHA-256); other scripts fall back to installed Windows fonts. Arabic typography adjustments (line-height 1.6, +1 px size via `:lang(ar)`) are part of the type scale (REQ-I18N-007 UI fonts; 07 §7.4.5; 12 §12.16).

**Why this priority**: fonts ride the token/type foundation and are verified visually through the same harness snapshots; no interaction blocks on them, but text rendering everywhere depends on the fallback chain being right.

**Independent Test**: font bundle checks (files + licenses + `fonts.json` hashes) and the harness snapshots rendering Latin/Arabic/mono text (card deliverables; T-A11Y-004/T-I18N-003 runs exercise the rendered text).

**Acceptance Scenarios**:

1. **Given** a fresh install on a clean machine, **When** the UI and harness render English and Arabic text, **Then** bundled fonts are used (no download, no system-font dependency) with the documented fallback chain for other scripts.
2. **Given** the font bundle, **When** its integrity is checked, **Then** each file matches its recorded SHA-256 and each license file is present (12 §12.16).

---

### User Story 5 - The UI harness and the rules that keep it clean (Priority: P5)

The UI harness (Vite + mock backend skeleton + Playwright + axe) renders every component story, and the lint stack enforces the canonical rules: strict TypeScript with no `any`; no JSX text literals (i18n keys only); `jsx-a11y` errors; CSS Modules with tokens and logical properties only (`stylelint-use-logical`); Prettier formatting. These are the project's UI gates — the previously skipped verify.ps1 UI/i18n checks activate with this package (ADR-011; 02 §2.14; card Deliverables).

**Why this priority**: the harness and configs are the enforcement surface for Stories 1–4 and every later UI package; they land with the first stories and close the package by turning the gates on.

**Independent Test**: `pnpm -C app typecheck`, `lint`, `test`, `i18n:check` and `pnpm -C app harness` — all green in verify.ps1 (card Deliverables; 02 §2.14).

**Acceptance Scenarios**:

1. **Given** a component using a raw color or a physical CSS property, **When** lint runs, **Then** it fails (tokens and logical properties only).
2. **Given** a JSX text literal or an `any`, **When** lint/typecheck runs, **Then** it fails (G-13; 02 §2.14).
3. **Given** the full gate suite, **When** `tools/verify.ps1` runs, **Then** the UI typecheck/lint/test and i18n gates execute (no longer SKIPPED) and pass.

---

### Edge Cases

- Windows contrast theme toggled while running: tokens re-map to system colors live; clip colors become outlined shapes with labels, meters use patterns (07 §7.4.3).
- Windows display language not among LOC-01..06: default to English (REQ-I18N-001).
- Zoom extremes (80%, 200%) with long localized labels: wrap or truncate with a tooltip — never clipped silently; the +40% expansion budget is tested with LOC-90 (07 §7.12 rule 5).
- Bidi user content (clip names, captions, notes) in lists of the opposite direction: `dir="auto"` with bidi isolation (07 §7.11).
- A locale file with missing/extra keys, or a pseudo-locale used in a production build: CI fails the former; pseudo-locales LOC-90/91 exist for tests only (REQ-I18N-004; 12 §12.14).
- A font file mismatching its recorded SHA-256, or a missing license file: the bundle check fails (12 §12.16).
- Reduced motion on: all motion durations become 0 except progress indicators (07 §7.4.6).
- High Contrast + accent selection: accents follow the forced-colors system mapping (Highlight/HighlightText), never raw brand colors (07 §7.4.3).
- Lists beyond 200 rows: virtualized rendering (07 §7.6).
- Timecode inside an RTL paragraph: Western digits in an LTR isolate (`<bdi dir="ltr">`), always (07 §7.11).

## Requirements *(mandatory)*

All requirements are scoped to the WP-04 card and the "(infrastructure)"/"(components)" shares of its rows. IDs are feature-local (FR-xxx) and cite their canonical source.

### Functional Requirements

- **FR-001**: The design tokens MUST exist as CSS custom properties in `app/src/design/tokens.css` carrying the complete 07 §7.4 set — Dark and Light color tables, the High Contrast system-color mapping, timeline/media colors (§7.4.4), typography stacks and sizes (§7.4.5), and space/shape/elevation/motion (§7.4.6) — and components MUST reference tokens only, never raw colors (07 §7.4).
- **FR-002**: Themes MUST include Dark (default), Light, and High Contrast; High Contrast activates automatically under `forced-colors` (Windows contrast themes) and maps tokens to system colors per §7.4.3; switching theme or accent applies without restart (REQ-APP-009; 07 §7.4.1–7.4.3).
- **FR-003**: The accent palette MUST offer CLR-06 (default) plus CLR-07, CLR-04, CLR-02, CLR-08, swapping `--accent-solid`/`--accent-text`/`--accent-subtle` with the documented values per theme (12 §12.17; 07 §7.4.1).
- **FR-004**: Application zoom MUST span 80–200% with crisp text and icons at 100/125/150/175/200% monitor scaling (per-monitor DPI), using SVG icons and variable fonts (REQ-APP-010).
- **FR-005**: FNT-01 (Inter variable), FNT-02 (Noto Sans Arabic variable), and FNT-03 (JetBrains Mono) MUST be bundled as separate files under `assets/fonts/` with their SIL OFL 1.1 license files and a `fonts.json` recording each file's source URL and SHA-256; other scripts use installed Windows fonts as fallback; Arabic gets line-height 1.6 and +1 px size via `:lang(ar)` (REQ-I18N-007; 12 §12.16; 07 §7.4.5).
- **FR-006**: The component library MUST implement every 07 §7.6 component on React Aria Components, styled only with tokens and logical properties, including the NumberField (scrub, Shift ×10, Ctrl ×0.1, typed math, mixed state), the segmented always-LTR TimecodeField, virtualized tree/grid/table/list beyond 200 rows, badges that never rely on color alone, and EmptyState (icon, one sentence, primary action, shortcut hint) (07 §7.6; 02 §2.14).
- **FR-007**: Every component MUST have harness stories covering default, hover, focus-visible, pressed, disabled, error, RTL, compact density, and High Contrast, with snapshots in dark/light × en/ar and axe accessibility checks (07 §7.6; card Acceptance).
- **FR-008**: The i18n runtime MUST load LOC-01..06 (en source + ar/fr/es/de/pt-BR), switch language at runtime without restart, default to the Windows display language when supported else English, and set `dir`/`lang` on the document root per locale (REQ-I18N-001 infrastructure; 12 §12.14; 07 §7.11).
- **FR-009**: RTL MUST mirror layout, docks, menus, dialogs, and directional icons via logical properties, while the timeline axis, ruler, transport, scopes, meters, and waveforms do not mirror; timecodes/frame numbers/durations use Western digits in an LTR isolate; user content uses `dir="auto"` with bidi isolation; sliders follow the §7.11 exceptions (REQ-I18N-002 infrastructure; 07 §7.11).
- **FR-010**: Messages MUST use ICU MessageFormat including Arabic's zero/one/two/few/many/other; numbers and dates format via `Intl` with `numberingSystem: "latn"`; translated fragments are never concatenated (REQ-I18N-003; 07 §7.12).
- **FR-011**: `app/src/locales/` MUST contain `en.json` (source), the five other locale files with exactly the same key set, and `en.meta.json` with a translator description per key; the key-set check fails on missing or extra keys; pseudo-locales LOC-90 (`en-XA`, accented, +40%) and LOC-91 (`ar-XB`, RTL/bidi) are available to tests; toolbar/button labels tolerate +40% expansion (REQ-I18N-004 infrastructure; 07 §7.12; 12 §12.14).
- **FR-012**: Components MUST expose names, roles, values, and states to screen readers; live-region infrastructure MUST provide one polite region (edits, job completions) and one assertive region (errors); focus is visible via `--focus-ring` on `:focus-visible` only; landmarks and canvas `role="application"`/`aria-roledescription` patterns follow §7.13 (REQ-A11Y-002 components share; 07 §7.13).
- **FR-013**: Dark and Light themes MUST meet WCAG 2.2 AA — 4.5:1 text, 3:1 UI components and focus indicators — enforced by the token values and verified in the harness; High Contrast follows Windows forced colors (REQ-A11Y-003; 07 §7.4, §7.13).
- **FR-014**: `prefers-reduced-motion` MUST set all motion durations to 0 except progress indicators, and no information may be conveyed by color alone (badges carry text/icons; High Contrast uses patterns) (REQ-A11Y-004; 07 §7.4.6, §7.13).
- **FR-015**: Every icon-only button MUST have a tooltip (500 ms delay) carrying its label and shortcut, and an accessible name; tooltips and EmptyState ship as components (REQ-HLP-003 components share; 07 §7.5–7.6).
- **FR-016**: The lint/build stack MUST enforce: strict TypeScript (no `any`), no JSX text literals (react-intl IDs `area.component.purpose` only), `jsx-a11y` errors, CSS Modules with tokens and logical properties only (`stylelint-use-logical`), Prettier — per ADR-011 and 02 §2.14; all SPDX headers on new files.
- **FR-017**: The UI harness MUST render every story via Vite with a mock backend skeleton, run Playwright snapshots (dark/light × en/ar) and axe checks, and back the `verify.ps1` UI gates (`typecheck`, `lint`, `test`, `i18n:check`, harness) so they execute and pass (card Deliverables; ADR-011).
- **FR-018**: Scope boundary — WP-04 delivers tokens/themes, the component library, the i18n runtime, FNT-01..03, the lint stack, and the harness. NOT included: workspace/panel composition, menus wiring, real backend commands, the settings pages and dialogs (WP-05, WP-24); complete runtime UI translation coverage T-I18N-001/002 beyond the harness (WP-05/WP-25); native-speaker terminology review (WP-25); title fonts FNT-04..08 and `fonts.json` completion for them (WP-17); custom feature icons beyond the base set and IconButton pattern (they ship with their features per 07 §7.5) (card Implements markers; 01 rows' WP columns).

### Key Entities *(this package owns the design system, not the product UI)*

- **Design tokens** (07 §7.4): CSS custom properties in `app/src/design/tokens.css`; the only color/size/motion source components may reference.
- **Theme** (07 §7.4.1–7.4.3): dark | light | high-contrast × accent (CLR-06/07/04/02/08); applied without restart; forced-colors mapping.
- **Component** (07 §7.6): a React Aria Components primitive with token/logical-property styling and its nine harness stories.
- **Locale catalog** (07 §7.12; 12 §12.14): `en.json` + five translations + `en.meta.json`; identical key sets; LOC-90/91 pseudo-locales for tests.
- **Font bundle record** (12 §12.16): `assets/fonts/fonts.json` — family, file, source URL, SHA-256, license file.
- **Harness story/snapshot** (07 §7.6): per-component states in dark/light × en/ar with axe results.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: T-UI-001 passes — Dark/Light/High Contrast and all five accents apply without restart, with High Contrast following Windows forced colors (REQ-APP-009).
- **SC-002**: T-UI-002 passes — zoom 80–200% and per-monitor DPI render crisp text and icons at 100/125/150/175/200% (REQ-APP-010).
- **SC-003**: T-I18N-003 passes — ICU messages (including Arabic plurals), locale numbers/dates with `latn` digits, and LTR-isolated timecodes behave per 07 §7.11–7.12 (REQ-I18N-003).
- **SC-004**: T-I18N-004 passes — all six locale files carry identical key sets, the check fails on any mismatch, and LOC-90/91 exercise expansion and RTL in tests (REQ-I18N-004).
- **SC-005**: T-A11Y-004 passes — WCAG 2.2 AA contrast in Dark and Light, forced-colors mapping, reduced-motion compliance, and no color-only meaning (REQ-A11Y-003/004).
- **SC-006**: Harness snapshots exist for every 07 §7.6 component in dark/light × en/ar, all passing with zero axe violations (card Acceptance).

## Assumptions

- Dependencies come from the 00 §0.8 baseline as approved by ADR-011 (React 19, TypeScript 6.0, Vite 8, React Aria Components, react-intl, Zustand, dockview, Playwright/axe); exact locked versions resolve at planning from the lockfile (G-11; no new ADR needed — ADR-011 already decides the stack).
- `fonts.json` is created here for FNT-01..03; 12 §12.16 assigns FNT-04..08 and their entries to WP-17.
- The initial key set is the vocabulary of this package (tokens' UI-facing strings, component labels/tooltips, harness chrome); later packages add keys under the same CI equality rule from day one.
- Initial translations for the initial key set are authored now; the native-speaker terminology review is WP-25 (07 §7.12 rule 8).
- Lucide (ISC) is the base icon set with the documented grid/stroke conventions (07 §7.5); custom product icons ship with their features, so WP-04 delivers the base set, the IconButton/Tooltip pattern, and the custom-icon slots' conventions.
- The previously SKIPPED `verify.ps1` UI gates activate in this package because their scripts come into existence here (forward-declared by target presence).
- Playwright snapshots are deterministic (fixed viewport, fonts, and animation disabled) so they are re-runnable offline in CI (constitution III).
