# WP-05 Implementation Plan — Application shell

Branch `feat/005-app-shell` (from merged main `4c6e5c7` + STATUS chore). Spec: `specs/005-app-shell/spec.md` (FR-001..FR-018). All decisions cite canonical docs; research facts in `research.md`.

## Guiding constraints

- WP-04's design system is the only UI vocabulary: tokens, components (`Menu`, `Button`, `Dialog`, `Toast`, `Tabs`…), i18n runtime, six locales, harness. No raw colors, no literal strings (G-13), logical properties only.
- Everything UI-visible ships with locale keys in all six catalogs + meta; harness stories/goldens for the shell; keyboard/a11y assertions per WP-04 patterns.
- Unimplemented commands/panels are visibly disabled/labeled — never fake (card rule; G-02).
- Rust changes stay inside `app/src-tauri` (app commands, plugins, config) — no new workspace crates.

## Phase 0 — Catalog foundation (FR-008 ground work)

- `app/src/commands/catalog.ts`: the 168-entry typed table (commandId, labelKey, menu path, default shortcut as `KeyboardEvent.code` sequence, context, checkable, implementedIn WP) transcribed from 12 §12.10; `app/src/commands/catalog.fixture.ts` with the raw KB table for the equality test.
- Unit tests: registry completeness (168/168, ids/labels/menus/contexts/shortcuts exactly per fixture); default-map zero conflicts within any context (the 12 §12.10 closing guarantee).
- Shortcut → display label helper (US label fallback; `navigator.keyboard.getLayoutMap()` when available).

**Files**: `app/src/commands/*`, unit tests. **Tests**: catalog completeness part of T-APP-003.

## Phase 1 — Command runtime (FR-008, FR-009, FR-010)

- `app/src/commands/registry.ts`: runtime registry over the catalog — enable state per command (implementedIn + focused-panel context per 07 §7.9.1 + project state flags), handler map, checked state, the localized "Available in a later version" tooltip marker.
- `app/src/commands/dispatcher.ts`: single keydown listener dispatching by physical `code`; text-field exception list (Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11), IME suppression (`isComposing`), Esc field-then-panel rule (07 §7.9.2); conflict-free by construction (registry data).
- `app/src/commands/palette/` (CommandPalette): Ctrl+Shift+P overlay, fuzzy search (label + id, six locales), keyboard-navigable, shortcut display, disabled entries with tooltip.
- Unit/harness tests: dispatch table per layout (T-I18N-005 with emulated layouts where possible + unit tests over code sequences), exception list, palette behavior (T-APP-003 share).

**Files**: `app/src/commands/**`, harness stories. **Tests**: T-APP-003 (registry/palette/enablement), T-I18N-005.

## Phase 2 — Shell chrome (FR-002..FR-005)

- `app/src/shell/TitleBar.tsx` (custom 32 px bar, project name + dirty dot, caption-button area preserving native Snap Layouts), `MenuBar.tsx` rendered from the registry (menu path data; group separators per 12 §12.11; ✓ items), `Toolbar.tsx` (anatomy contents: tools KB-091..097 wired as disabled-until-WP-12 toggles except focusable ones, undo/redo, Snap/Link toggles, workspace switcher, Export…, bell, settings, jobs), `StatusBar.tsx` (five fields; "—" until data arrives).
- Window config: min size 1280×720, per-monitor DPI (Tauri window settings) — FR-002.
- All chrome localized (six catalogs + meta); icon-only controls via IconButton + tooltips with shortcut (REQ-HLP-003).
- Harness stories for title/menus/toolbar/status; goldens dark/light × en/ar; a11y (focus rings, names) per WP-04 suites.

**Files**: `app/src/shell/**`, `app/src/App.tsx`, harness. **Tests**: T-APP-001 parts.

## Phase 3 — Docking, workspaces, focus model (FR-006, FR-007, FR-015)

- `app/src/shell/dock/`: dockview 8.8.x integration; panel registry (id, title key, icon, owning WP); shell-owned panels + labeled placeholders for later-WP panels; operations dock/tab/resize/float/maximize(`)/close/reopen (View › Panels).
- Workspace service: WS-01..04 default layouts (07 §7.3 + Edit-region defaults 07 §7.2); switch via KB-125..128; reset via KB-130; persistence through the WP-03 settings service into `alvs.workspaces.active`/`alvs.workspaces.saved`.
- Focus model: F6/Shift+F6 cycle, Shift+1..9 focus commands (KB-135..143), focused-panel accent line (07 §7.9.1), `.focus-visible` rings; live-region announcements for shell feedback.
- Tests: T-APP-002 flows in the harness + e2e; T-A11Y-001 keyboard-only walkthrough.

**Files**: `app/src/shell/dock/**`, `app/src/panels/**` (placeholders), settings wiring. **Tests**: T-APP-002, T-A11Y-001.

## Phase 4 — Rust shell services (FR-001, FR-016)

- `app/src-tauri`: `app_info` (AppInfo DTO per 04 §4.3.1 incl. renderer from WP-03/WP-00 infra where available — fields the engine WPs own are filled by them, never faked), `app_ready`, `app_quit { force }` (shutdown sequence per 02 §2.5 with the WP-03 services); tauri-plugin-single-instance 2.5.x registered with the documented callback → `app:open-file` emit (02 §2.5 step 4).
- Verify the mode-dispatch skeleton (02 §2.3) unchanged for worker/crash/cli branches; GUI branch activates.
- ts-rs DTOs (AppInfo, SaveResult consumers…) exported; typed wrappers in `app/src/ipc/commands.ts`; bindings-drift green (ADR-010).
- Rust unit tests for command validation + envelope errors.

**Files**: `app/src-tauri/src/**`, `app/src/ipc/**`. **Tests**: cargo units; bindings gate.

## Phase 5 — Project lifecycle UI (FR-011, FR-012, FR-013)

- `app/src/shell/Welcome.tsx` (states: empty/recents/missing; `project_recent`; thumbnails via asset protocol scope; Getting Started link).
- `app/src/dialogs/NewProject.tsx` (DLG-01 fields per 07 §7.14; presets from 12 §12.8, fps from 12 §12.9 — catalog data files mirroring the doc tables with fixture tests, same pattern as D1), `UnsavedChanges.tsx` (DLG-20).
- File-command wiring: new/open (dialog plugin picker + `project_open`, read-only E1007 path with the 07 §7.10 banner + `project_take_over_lock`), save/saveAs, closeProject/exit gated by DLG-20 → `project_close { discardChanges }`/`app_quit`; `model:dirty` → title dot.
- Error envelopes surface per FR-014's toast fabric (E-codes + recovery).
- Tests: T-APP-004/007 flows (harness + e2e), T-PRJ-001 share for DLG-01; mock-backend coverage of every implemented command (card deliverable).

**Files**: `app/src/shell/Welcome.tsx`, `app/src/dialogs/**`, `app/src/commands/handlers/file.ts`. **Tests**: T-APP-004, T-APP-007.

## Phase 6 — Notifications and status wiring (FR-005 events, FR-014)

- Event subscriptions (`app:notify`, `autosave:status`, `jobs:update` summary, `renderer:changed`); notification center (history panel in the dock) + toast fabric over the WP-04 Toast; error code + recovery action rendering.
- Status bar live fields for the events that exist; placeholders stay "—".
- Tests: T-APP-005 (envelope rendering, recovery, center); harness stories with mocked events.

**Files**: `app/src/shell/notifications/**`, StatusBar wiring. **Tests**: T-APP-005.

## Phase 7 — Hardening + e2e activation (FR-017, FR-018)

- Apply 05 §5.4 verbatim in `tauri.conf.json` (CSP) + `capabilities/main.json`; unit test asserting config equality with the doc values (D5); navigation lock + window.open denial verified.
- `e2e-smoke` job: replace the no-op with tauri-driver steps — launch → welcome → new project → project opens (the T-E2E-001 prefix); explicit skip-list comment for the later steps.
- Full gates: `verify.ps1` 13/13, harness goldens, e2e green in CI.

**Files**: `app/src-tauri/tauri.conf.json`, `capabilities/**`, `.github/workflows/ci.yml`, `tests/e2e/**`. **Tests**: T-SEC-005 part, T-E2E-001 prefix.

## Phase 8 — Docs, evidence, convergence

- Evidence file with commands/results; STATUS flip; docs amendments only if a canonical gap is found (escalate first per G-17); `/speckit.analyze`, converge loop, then the review protocol (this repo's reviewer rounds + your approval).

## Risks / open points (none blocking plan approval)

- **Snap Layouts on a custom title bar** (FR-002 AC1): WebView2/Tauri custom caption keeps native hit-testing when the caption area is preserved; verified at implementation, manual check recorded in evidence if not automatable.
- **Thumbnails on Welcome**: recent-project thumbnails need the asset protocol scope (already documented for `$APPLOCALDATA/cache/thumbs/**`); WP-05 renders the documented placeholder until WP-08 generates real thumbs (no fake images).
- **dockview React API details** (8.8.x) verified against the installed package at implementation start (G-10).
