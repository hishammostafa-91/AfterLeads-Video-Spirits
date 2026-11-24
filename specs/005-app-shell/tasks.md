---
description: "Task list for WP-05 - Application shell"
---

# Tasks: WP-05 - Application shell

**Input**: Design documents from `/specs/005-app-shell/` (plan.md, spec.md, research.md)

**Prerequisites**: plan.md (owner-approved 2026-11-21), spec.md (7 stories P1-P3, FR-001..FR-018)

**Tests**: Included — the card's Acceptance row (T-APP-001..007, T-A11Y-001, T-I18N-005, T-SEC-005 part) and FR ACs require them; the WP-04 harness/jsdom patterns carry over (deterministic goldens, per-project themes, reduced motion).

**Organization**: Phase 0-1 build the command foundation every story consumes; Phases 2-6 are the stories in priority order; Phase 7 hardens and activates e2e; Phase 8 closes out. Every new file carries the SPDX header (02 §2.14); strings ship in all six locales + meta.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US7 per spec.md; foundation/polish carry no label

---

## Phase 0 — Command catalog foundation (plan Phase 0)

- [X] T001 Create `app/src/commands/catalog.ts`: the 168-entry typed table from 12 §12.10 (commandId, labelKey, menu path, default shortcut as physical `KeyboardEvent.code` sequence, context, checkable, implementedIn WP) and `app/src/commands/catalog.fixture.ts` with the raw KB table; both files' headers cite 12 §12.10
- [X] T002 Unit tests `app/src/commands/catalog.test.ts`: registry ≡ fixture (168/168 exact fields), zero conflicts in the default map within any context, menu paths cover exactly the 12 §12.11 groups, KB-168 is palette-only (US2)
- [X] T003 Shortcut display helper `app/src/commands/keyLabel.ts` (US label fallback, `navigator.keyboard.getLayoutMap()` when available) with unit tests

## Phase 1 — Command runtime (plan Phase 1)

- [X] T004 `app/src/commands/registry.ts`: runtime registry — enable state (implementedIn + focused-panel context per 07 §7.9.1 + project flags), handler map, checked state, the localized later-version tooltip marker (US2)
- [X] T005 `app/src/commands/dispatcher.ts`: single keydown listener dispatching by physical code; text-field exception list (Ctrl+S/Ctrl+Shift+S/Ctrl+E/Ctrl+Shift+P/F1/F6/Shift+F6/F11), IME suppression, Esc field-then-panel rule per 07 §7.9.2 (US2)
- [X] T006 Unit tests for the dispatcher: code-sequence dispatch table, exceptions, IME, Esc chain (T-I18N-005 share)
- [X] T007 `app/src/commands/palette/CommandPalette.tsx`: Ctrl+Shift+P overlay, fuzzy search over labels+ids in six locales, keyboard-navigable, shortcut display, disabled entries non-runnable with tooltip (US2)
- [X] T008 Harness story for the palette; interaction goldens + a11y assertions (T-APP-003 share)

## Phase 2 — Shell chrome (plan Phase 2) — US1

- [X] T009 Window config: min size 1280×720, per-monitor DPI (FR-002); Tauri window settings + verification notes in evidence
- [X] T010 `app/src/shell/TitleBar.tsx`: 32 px custom bar hosting the menu bar, project name + dirty marker slot, caption-button area preserving Snap Layouts (FR-002)
- [X] T011 `app/src/shell/MenuBar.tsx` rendered from registry data with 12 §12.11 group separators and ✓ items (FR-003); story + goldens
- [X] T012 `app/src/shell/Toolbar.tsx` per the 07 §7.2 anatomy (tools KB-091..097, undo/redo, Snap/Link, workspace switcher, Export…, bell, settings, jobs) with tooltips + accessible names (FR-004)
- [X] T013 `app/src/shell/StatusBar.tsx`: five documented fields, "—" placeholders until data (FR-005); story + goldens
- [X] T014 Locale keys for all chrome in six catalogs + meta; a11y pass (focus rings, names, contrast inherited from tokens)

## Phase 3 — Docking, workspaces, focus (plan Phase 3) — US4/US6

- [X] T015 `app/src/shell/dock/`: dockview 8.8.x integration (API verified against the installed package first, G-10); panel registry; dock/tab/resize/float/maximize/close/reopen (FR-006)
- [X] T016 Panel placeholders for later-WP panels (labeled, accessible, empty — owning WP named) (FR-006, D3)
- [X] T017 Workspace service: WS-01..04 default layouts per 07 §7.3 + 07 §7.2 region defaults; KB-125..128 switching; KB-130 reset; persistence via WP-03 settings into `alvs.workspaces.active`/`alvs.workspaces.saved` (FR-007)
- [X] T018 Focus model: F6/Shift+F6 cycle, Shift+1..9 (KB-135..143), focused-panel accent line (07 §7.9.1), live-region shell announcements (FR-015)
- [X] T019 Harness stories + tests: docking operations and workspace persistence (T-APP-002), keyboard-only walkthrough (T-A11Y-001)

## Phase 4 — Rust shell services (plan Phase 4) — US7

- [X] T020 `app/src-tauri`: `app_info` (AppInfo DTO per 04 §4.3.1), `app_ready`, `app_quit { force }` per 02 §2.5; validation + ErrorEnvelope; cargo unit tests (FR-016)
- [X] T021 Register `tauri-plugin-single-instance` 2.5.x; forwarded `.alvs` path → `app:open-file` event (02 §2.5 step 4); verify the 02 §2.3 mode dispatch is untouched for worker/crash/cli (FR-001)
- [X] T022 ts-rs DTO exports + typed wrappers `app/src/ipc/commands.ts`; bindings-drift green (ADR-010); mock-backend coverage of every implemented command (FR-016, card)

## Phase 5 — Project lifecycle UI (plan Phase 5) — US3

- [X] T023 `app/src/shell/Welcome.tsx`: New/Open/recents with thumbnails placeholder + missing indicator + Getting Started; three states (FR-011, T-APP-004)
- [X] T024 `app/src/dialogs/NewProject.tsx` (DLG-01) with SQP/FPS catalog data files + fixture tests (D1 pattern); wired to `project_new`; E1008/E8004 surfaces (FR-012)
- [X] T025 `app/src/dialogs/UnsavedChanges.tsx` (DLG-20) three-way gate on close/exit/open (FR-013, T-APP-007)
- [X] T026 File-command wiring: open (picker + `project_open` + read-only banner + take-over), save/saveAs, close/exit; `model:dirty` → title dot (FR-013)
- [X] T027 Harness stories + e2e-lite flows for welcome/dialog/unsaved (mock backend); T-APP-004/007 green

## Phase 6 — Notifications and status wiring (plan Phase 6) — US5

- [X] T028 Event fabric: subscribe `app:notify`, `autosave:status`, `jobs:update`, `renderer:changed`; status-bar live fields (FR-005)
- [X] T029 Notification center (dock panel, history) + toast fabric over the WP-04 Toast; error code + recovery action (FR-014, T-APP-005; T048 stays tracked)
- [X] T030 Harness stories with mocked events; T-APP-005 green

## Phase 7 — Hardening and e2e activation (plan Phase 7) — US7

- [ ] T031 Apply 05 §5.4 verbatim: CSP in `tauri.conf.json`, capabilities `app/src-tauri/capabilities/main.json`, asset scope, navigation lock, DevTools off in release (FR-017)
- [ ] T032 Unit test asserting config equality with the 05 §5.4 values (D5); T-SEC-005 part green
- [ ] T033 Replace the `e2e-smoke` no-op with tauri-driver steps: launch → welcome → new project; skip-list comment for later T-E2E-001 steps (FR-018); CI green

## Phase 8 — Closeout (plan Phase 8)

- [ ] T034 Evidence file with all commands/results; STATUS → in-review at `/wp-finish`
- [ ] T035 Full gates (`verify.ps1` 13/13 incl. new suites), golden regen protocol if shell goldens changed, `/speckit.analyze` (no CRITICAL/HIGH), converge loop, reviewer rounds per protocol
