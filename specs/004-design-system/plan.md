# Implementation Plan: WP-04 — Design system and UI foundation

**Branch**: `feat/004-design-system` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-design-system/spec.md`; constraints from the owner's `/speckit.specify` invocation (card docs/10 §10.7; REQ-APP-009/010, REQ-I18N-001/002/003/004/007, REQ-A11Y-002/003/004, REQ-HLP-003 with the card's infrastructure/components shares; acceptance T-UI-001/002, T-I18N-003/004, T-A11Y-004 + harness snapshots dark/light × en/ar; read only 07 §7.1, §7.4–7.6, §7.11–7.13, 02 ADR-011 + §2.14, 12 §12.14/§12.16/§12.17).

## Summary

Deliver the design system exactly as the WP-04 card specifies, inside the existing Tauri app shell (`app/`, WP-00 skeleton: React 19.3 + TS 6.0.3 + Vite 8.3, `src/ipc/bindings` from WP-02 already present): `app/src/design/tokens.css` with the complete 07 §7.4 token set and a theme/accent mechanism that switches without restart (dark/light attribute swap + `forced-colors` High Contrast), the 07 §7.6 component library on React Aria Components styled only with tokens and logical properties, the react-intl runtime with six shipped locales + identical key sets + `en.meta.json` + LOC-90/91 pseudo-locales for tests, FNT-01..03 bundled with license files and a hashed `fonts.json`, the ESLint/Stylelint/Prettier enforcement stack (i18n literal rule, jsx-a11y, logical properties), and the Vite UI harness (Playwright snapshots dark/light × en/ar + axe, mock-backend skeleton) whose `package.json` scripts activate the four currently-SKIPPED `verify.ps1` UI gates. No workspace composition, no panels, no backend wiring, no dockview (all WP-05), no title fonts (WP-17), no native translation review (WP-25).

## Technical Context

**Language/Version**: TypeScript 6.0.x (`strict`; typescript-eslint peer `<6.1` — 00 §0.8 row), React 19.3, Vite 8.3, pnpm 12.x (12.16.1 installed). The `app/` crate skeleton, `tsconfig.json`, `vite.config.ts`, and `src/ipc/bindings` already exist from WP-00/WP-02; no Rust changes in this package (backend untouched).

**Primary Dependencies** (all 00 §0.8 baseline rows — versions below are the baseline's approved majors/minors; exact pins resolve from the registry at implementation and lock into `app/pnpm-lock.yaml`, G-10/G-11; no ADRs needed — ADR-011 already decides the stack): `react-aria-components` 1.22.x; `react-intl` 12.x + `@formatjs/cli` 6.x; `zustand` 5.x; tests: `vitest` 5.x, `@testing-library/react` 16.x, `jsdom` 30.x, `@playwright/test` 1.64.x, `@axe-core/playwright` 4.x; lint: `eslint` 9.39.x, `typescript-eslint` 8.x, `eslint-plugin-jsx-a11y` 6.x, `eslint-plugin-react-hooks` 7.x, `eslint-plugin-i18next` 6.x, `eslint-plugin-formatjs` 8.x, `stylelint` 17.x + `stylelint-config-standard` 40.x + `stylelint-use-logical` 2.x, `prettier` 3.x. Explicitly NOT added now: `dockview` (WP-05 workspace composition), any virtualization library (none in the baseline — custom windowing hook, research D10), any HTTP client (SEC-10).

**Storage**: none persistent — theme/accent/locale live in Zustand stores with a swappable persistence seam (URL params in the harness now; `alvs.appearance.*` settings keys bind in WP-05/WP-24). Locale catalogs are static JSON in `app/src/locales/`; fonts are static assets in `assets/fonts/` with an integrity manifest.

**Testing**: Vitest + Testing Library for component behavior and i18n units (T-UI-001/002 logic, T-I18N-003/004 — deterministic, jsdom); the Playwright harness for snapshots (every component × 9 story states × dark/light × en/ar) and per-story axe checks (T-A11Y-004); snapshot determinism by fixed viewport, `prefers-reduced-motion: reduce` emulation, awaited `document.fonts.ready`, and stubbed dates (constitution III). `pnpm -C app test` backs the `ui-test` gate; the Playwright suite runs via `pnpm -C app harness` (and in CI after this package).

**Target Platform**: Windows 11 x64, WebView2 (Chromium) — CSS `forced-colors` and the standardized CSS `zoom` property are the platform hooks for High Contrast and the 80–200% application zoom (research D3).

**Project Type**: additive UI layer inside `app/` — new `src/design/`, `src/components/`, `src/i18n/`, `src/harness/`, `src/locales/`, `tests/` (harness), plus root configs (`eslint.config.js`, `.stylelintrc.json`, `.prettierrc.json`) and `assets/fonts/`. The four `verify.ps1` UI gates activate by defining the scripts `typecheck`, `lint`, `test`, `i18n:check` in `app/package.json` (verified in `tools/verify.ps1`: `Invoke-Gate 'ui-*' (Test-AppScript '<name>') ...`).

**Performance Goals**: none in the card; obligations are UI-quality budgets — WCAG 2.2 AA contrast (4.5:1/3:1), interactions within UXP-08's 100 ms (component-level: no layout thrash in stories), virtualized lists beyond 200 rows.

**Constraints**: every behavior cites 07 §7.1/§7.4–7.6/§7.11–7.13, 02 ADR-011 + §2.14, 12 §12.14/§12.16/§12.17; no JSX text literals; tokens and logical properties only (Stylelint-enforced); `eslint-plugin-jsx-a11y` errors fail CI; AGPL headers on every new file; no new dependencies outside the baseline rows; pseudo-locales exist only for tests.

**Scale/Scope**: ~35 components (07 §7.6) × 9 story states, 1 tokens file + theme layer, 7 locale files (+2 generated pseudo-locales), 3 font families, 4 lint configs, 1 Playwright harness. The largest surfaces are the component matrix and the token set.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| I. Canonical documentation first | PASS | Every FR/decision cites 01 REQ rows, 07 §7.1/§7.4–7.6/§7.11–7.13, 02 ADR-011/§2.14, 12 §12.14/§12.16/§12.17; versions verified against 00 §0.8 rows (G-10); no conflicts to escalate. |
| II. Scope discipline | PASS | FR-018 fences workspace/panels/menus/backend (WP-05), settings pages (WP-24), translation review (WP-25), title fonts (WP-17), custom feature icons (ship with features). |
| III. Test-first evidence | PASS | Acceptance = T-UI-001/002, T-I18N-003/004, T-A11Y-004 + the snapshot matrix (card); harness snapshots deterministic (D7). |
| IV. Protected main and clean history | PASS | Work on `feat/004-design-system` from main `1dc6b02`; gates + review before the owner's merge. |
| V. Quality gates non-negotiable | PASS | This package *activates* the four SKIPPED UI gates by defining their scripts; all gates green before merge; no gate edits. |
| VI. Global and accessible by default | PASS | The package IS this principle's infrastructure: six locales from day one (G-13), RTL rules, WCAG 2.2 AA, keyboard/ARIA via React Aria (REQ-A11Y-002..004). |
| VII. Private, secure, local-first | PASS | No network (font files bundled with recorded hashes — no runtime download), no telemetry; fonts from official upstream repositories with OFL licenses recorded in the notices manifest (G-12). |
| VIII. Simplicity and performance budgets | PASS | Baseline stack per ADR-011; custom windowing instead of a new dependency (D10); no speculative abstractions — stories double as documentation. |
| IX. Open-source integrity | PASS | SPDX headers on all new files; Lucide (ISC) base icons; OFL fonts with license files in the bundle + notices; nothing copied from other products (G-20). |
| Technology constraints | PASS | Baseline rows only (00 §0.8 UI/lint/test tables); toolchain unchanged. |

## Project Structure

### Documentation (this feature)

```text
specs/004-design-system/
├── plan.md              # This file
├── research.md          # Phase 0 — D1..D14
├── data-model.md        # Phase 1 — entities, validation rules, story matrix
├── quickstart.md        # Phase 1 — per-acceptance validation guide
├── contracts/
│   └── design-system-apis.md  # Phase 1 — public TS/CSS surface + guarantees G1..G8
└── tasks.md             # Phase 2 (/speckit.tasks — not created by /speckit.plan)
```

### Source Code (repository root — what WP-04 writes)

```text
app/
├── package.json                   # + react-aria-components, react-intl, zustand, test/lint stack,
│                                  #   scripts: typecheck/lint/test/i18n:check/harness (activates UI gates)
├── pnpm-lock.yaml                 # created and committed
├── eslint.config.js  .stylelintrc.json  .prettierrc.json
├── index.html                     # lang/dir bootstrap
└── src/
    ├── design/
    │   ├── tokens.css             # 07 §7.4 verbatim: dark/light/HC + timeline colors, type, space/motion
    │   ├── fonts.css              # @font-face for FNT-01..03 (fallback chain per §7.4.5)
    │   ├── theme.ts               # theme/accent state + persistence seam (Zustand)
    │   └── icons.ts               # Lucide base-set loader + icon conventions
    ├── components/                # every 07 §7.6 component: <Name>.tsx + <Name>.module.css + <Name>.test.tsx
    │   ├── Button.tsx  IconButton.tsx  Menu.tsx  Tabs.tsx  Dialog.tsx  Tooltip.tsx  Toast.tsx
    │   ├── NumberField.tsx  Slider.tsx  TimecodeField.tsx  KeyframeToggle.tsx
    │   ├── ColorPicker.tsx  ColorWheel.tsx  CurveEditor.tsx  PointControl.tsx
    │   ├── Fields.tsx (Text/Search/TextArea)  Selection.tsx (Checkbox/Switch/Radio/Select/ComboBox)
    │   ├── Collections.tsx (Tree/GridList/Table/ListBox + windowing)
    │   ├── Feedback.tsx (ProgressBar/Meter/Spinner/Skeleton)  Badge.tsx  EmptyState.tsx
    │   └── a11y.ts                # live regions (polite/assertive), focus/landmark helpers
    ├── i18n/
    │   ├── runtime.tsx            # IntlProvider wrapper, locale registry, runtime switch
    │   ├── direction.ts           # dir/lang management (07 §7.11)
    │   ├── format.ts              # Intl numbers/dates (latn), LTR isolate helpers
    │   └── pseudo.ts              # LOC-90/91 generation (tests only)
    ├── locales/                   # en.json (source) + ar/fr/es/de/pt-BR.json + en.meta.json
    ├── harness/
    │   ├── HarnessApp.tsx         # story grid shell (?theme=&locale=&density=)
    │   ├── stories/               # one file per component: 9 states
    │   ├── mock/                  # mock backend skeleton (typed fixtures)
    │   └── snapshots/             # Playwright projects: dark/light × en/ar
    └── App.tsx                    # mounts the harness entry (replaced by WP-05's shell)
assets/fonts/                      # FNT-01..03 files + licenses + fonts.json
tests/ui/                          # Playwright config + harness/axe specs (repo tests/ dir)
```

**Structure Decision**: module-per-concern under `app/src` mirroring 07's own decomposition (design / components / i18n / harness); tokens as one verbatim CSS file because 07 §7.4 is specified as one token sheet; the harness lives inside the app (same Vite build) with its own route, so snapshots test the real component CSS, not a copy.

## Complexity Tracking

No constitution violations; no entries.

---

## Phase 0 — Research

See [research.md](research.md) — D1..D14 with citations and evidence; **zero unresolved clarifications** (the spec carries none). The decisions the reviewer should scrutinize hardest: **D3** (CSS `zoom` for the 80–200% application zoom), **D7** (snapshot determinism strategy), and **D10** (custom windowing instead of a new dependency).

## Phase 1 — Design

- [data-model.md](data-model.md): the design-system entities — token groups, theme/accent state, locale catalog and key equality, the component × story-state matrix, the font bundle manifest — with validation rules from the spec's FRs.
- [contracts/design-system-apis.md](contracts/design-system-apis.md): the public surface (theme/i18n runtime APIs, per-component prop contracts, harness URL contract, scripts and manifest schemas) and the test-enforced guarantees G1..G8.
- [quickstart.md](quickstart.md): validation commands for each acceptance test plus the gates.

## Constitution Check — re-evaluation after Phase 1 design

Re-checked after the design artifacts: no change. The design introduces no entities beyond 07 §7.4–7.6/§7.11–7.13 and 12 §12.14/§12.16/§12.17, no interfaces beyond the canonical component list and the theme/locale runtime; deferred items are explicitly owned by later packages (FR-018). Principles I–IX and the technology constraints remain PASS.
