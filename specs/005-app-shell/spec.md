# Feature Specification: WP-05 - Application shell

**Feature Branch**: `feat/005-app-shell`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "Implement WP-05 exactly as specified by its card in docs/10-execution.md §10.7. Requirements: REQ-APP-001, REQ-APP-002, REQ-APP-003 (built-in workspaces), REQ-APP-004, REQ-APP-005, REQ-APP-006, REQ-APP-007 (framework), REQ-APP-008, REQ-APP-010 (window DPI), REQ-APP-011, REQ-APP-012, REQ-PRJ-001 (DLG-01), REQ-I18N-005, REQ-A11Y-001, REQ-A11Y-002 (shell), REQ-HLP-003. Acceptance tests: T-APP-001..007, T-A11Y-001, T-I18N-005; CSP/capabilities part of T-SEC-005. Read only the sections in the card's Read line. Derive every requirement from the canonical documents and cite IDs; add nothing beyond them."

**Canonical basis**: WP-05 card (docs/10 §10.7). Requirements quoted from docs/01-product.md §1.6.1/§1.6.2/§1.6.16-18 rows where binding. Design authorities from the card's Read line: 07 §7.2-7.3 (window anatomy, workspaces), §7.7 (shell parts of the panel catalog — panel *contents* belong to their own work packages), §7.8-7.10 (context menus interaction rules, states), §7.14 (DLG-01, DLG-20); 04 §4.3.1-4.3.2 (app + project commands), §4.10 (events); 12 §12.10-12.11 (command catalog, menus); 05 §5.4 (webview hardening); 02 §2.3 (process model). Acceptance test IDs from the card's Acceptance row and docs/09 §9.10. Nothing is added beyond those documents (G-01, G-02).

**Scope boundaries** (from the card and requirement rows): panel *contents* (media grid, inspector sections, timeline, viewers, mixer, scopes) belong to WP-08/WP-09/WP-12+; custom workspace save/rename/delete is WP-24's share of REQ-APP-003; status-bar *values* for jobs/playback/proxies arrive with WP-07/WP-11 (REQ-APP-007 lists all three WPs); the shortcut *editor* (DLG-06, REQ-SET-002) is the Settings work package. Commands not yet implemented by any shipped work package register disabled with the tooltip from the card: "Available in a later version".

## User Scenarios & Testing *(mandatory)*

Actors: the editor's end user driving the shell (window, menus, workspaces, project lifecycle), and every later UI work package that mounts its panels into the dock and its commands into the registry. This package turns the WP-04 design system into the actual application frame.

### User Story 1 - The application frame: window, title bar, menus, toolbar, status bar (Priority: P1)

The user launches the app and gets a professional Windows 11 application window: a custom 32 px title bar hosting the menu bar with the native caption buttons (minimize/maximize/close) that keep Snap Layouts on the maximize button, a 40 px toolbar, dockable panel area, and a 24 px status bar. The window enforces the 1280×720 minimum and stays crisp on per-monitor DPI at every documented scaling step. The menu bar follows 12 §12.11 exactly — File, Edit, Clip, Timeline, Playback, View, Help, each listing its commands in KB order with the documented group separators and checkmarks (Snapping, Linked Selection, Loop, Play Every Frame, List View). (REQ-APP-001, REQ-APP-010 window part; 07 §7.2; 12 §12.11.)

**Why this priority**: the frame is what every later panel mounts into; nothing else in WP-05 is demonstrable without it.

**Independent Test**: T-APP-001 — window chrome, menus match 12 §12.11, toolbar, status bar fields, min size (harness + e2e).

**Acceptance Scenarios**:

1. **Given** a clean launch, **When** the window opens, **Then** the custom title bar (32 px) hosts the seven menus in order, the caption buttons work, and maximizing from the caption button offers Windows 11 Snap Layouts.
2. **Given** any window state, **When** the user tries to resize below 1280×720, **Then** the window refuses (minimum enforced).
3. **Given** the menu bar, **When** each menu opens, **Then** its items appear in KB order with the §12.11 group separators, localized labels, and shortcut hints in the current keyboard layout's glyphs.
4. **Given** a 150% DPI monitor, **When** the window renders, **Then** text and icons are crisp (per-monitor DPI awareness; REQ-APP-010 window part).

---

### User Story 2 - Every action is a command; physical-key shortcuts everywhere (Priority: P1)

Every user action is a command with a stable ID, localized label, enabled state, and optional shortcut. The registry contains **all 168 command IDs from 12 §12.10**; the ones whose feature ships in a later work package are disabled and show "Available in a later version" until their work package removes the marker. Commands are reachable from the menus, the command palette (Ctrl+Shift+P, fuzzy search, shows shortcuts), and the keyboard. The dispatcher binds to **physical keys** (`KeyboardEvent.code`) so shortcuts work identically on Arabic, AZERTY, and QWERTZ layouts, suppresses single-key shortcuts during IME composition, and follows the text-field exception list (Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11 still fire while a field has focus). (REQ-APP-004, REQ-I18N-005; 12 §12.10 preamble; 07 §7.9.2.)

**Why this priority**: the command system is the spine of the whole UI — menus, palette, shortcuts, and enablement all read from it, and T-APP-003 verifies completeness against the catalog.

**Independent Test**: T-APP-003 — registry completeness vs 12 §12.10, palette fuzzy search, enablement; T-I18N-005 — physical-key shortcuts on AZERTY, QWERTZ, Arabic layouts.

**Acceptance Scenarios**:

1. **Given** the registry, **When** it is enumerated, **Then** all KB-001..KB-168 IDs exist with labels, menu placement, contexts, and default shortcuts exactly per 12 §12.10, and the default map contains zero conflicts within any context.
2. **Given** an unimplemented command (e.g. `file.export` before WP-14), **When** its menu item renders, **Then** it is disabled with the localized "Available in a later version" tooltip and never appears enabled.
3. **Given** an Arabic keyboard layout, **When** the user presses the physical key positions of Ctrl+K, **Then** `clip.split` fires (dispatch by `KeyboardEvent.code`), and the menu shows the label in Arabic layout glyphs when `navigator.keyboard.getLayoutMap()` is available.
4. **Given** an editable text field focused, **When** the user presses J or Ctrl+K, **Then** the field receives the key (no command fires); Ctrl+S/Ctrl+Shift+S/Ctrl+E/Ctrl+Shift+P/F1/F6/F11 still fire their commands.
5. **Given** the command palette open (Ctrl+Shift+P), **When** the user types a fuzzy query, **Then** matching commands list with their shortcuts and Enter runs the first enabled match (disabled ones show their tooltip).

---

### User Story 3 - Projects from first launch: Welcome, New Project, Open, safe close (Priority: P1)

On startup with no restored project the user sees the Welcome screen: New Project, Open Project, recent projects with thumbnails and a "missing" indicator, and a Getting Started link. New Project opens DLG-01 — name (1–200 chars, Windows-safe), folder picker, sequence preset (12 §12.8, default "Auto — match first clip" with 1080p30 fallback), frame rate (12 §12.9), sample rate (48 kHz default) — and creates `<location>\<name>\<name>.alvs` through the WP-03 `project_new` command. Open reads `.alvs` via `project_open` (with the read-only and lock take-over paths surfaced per 07 §7.10). The title bar shows the project name and a dirty marker (•); closing the project or the app (or opening another) with unsaved changes asks via DLG-20: Save / Don't save / Cancel. (REQ-APP-005, REQ-PRJ-001, REQ-APP-012; 07 §7.14 DLG-01/DLG-20; 04 §4.3.2.)

**Why this priority**: this is the first real user value — creating and reopening a project through the real storage services, and the data-safety dialog REQ-APP-012 (P0).

**Independent Test**: T-APP-004 (welcome states), T-APP-007 (unsaved prompt on close/exit/open), T-PRJ-001 share (DLG-01).

**Acceptance Scenarios**:

1. **Given** first launch (no recents), **When** the Welcome screen shows, **Then** New Project, Open Project, and Getting Started are offered and the recents area shows an empty state.
2. **Given** recents exist, **When** the Welcome screen shows, **Then** each recent lists with a thumbnail; a project whose file is gone shows the missing indicator.
3. **Given** DLG-01, **When** the user submits a valid name/folder/preset, **Then** `project_new` runs and the project opens (title bar shows the name); invalid names are rejected inline; Enter activates Create, Esc cancels.
4. **Given** a dirty project, **When** the user closes the project/app or opens another, **Then** DLG-20 asks Save / Don't save / Cancel; Save runs `project_save` then proceeds, Cancel aborts, and the title-bar dot clears only when clean.
5. **Given** the app relaunches, **When** the previously open project is restorable, **Then** it reopens (per settings) instead of Welcome; a crash-offered autosave follows the DLG-15 flow owned by WP-03 services.

---

### User Story 4 - Panels dock; four built-in workspaces (Priority: P2)

Panels dock, tab, resize, float inside the main window, maximize (`), close, and reopen from View › Panels; the layout persists per workspace across restarts. The four built-in workspaces (WS-01 Edit, WS-02 Color, WS-03 Audio, WS-04 Captions per 07 §7.3) switch from View › Workspaces and Alt+1..4, and "Reset workspace" restores the documented defaults. In WP-05 the docks host the shell's own panels and placeholder surfaces for later work packages' panels (a panel placeholder is labeled and accessible; it never fakes content). Saved-workspace storage (`alvs.workspaces.saved`/`alvs.workspaces.active`, 03 §3.6) is wired; creating custom workspaces is WP-24. (REQ-APP-002, REQ-APP-003 built-ins; 07 §7.3.)

**Why this priority**: docking is the structural skeleton for every future panel; it must exist before WP-08+ mount anything.

**Independent Test**: T-APP-002 — docking operations, workspace switching, reset, persistence.

**Acceptance Scenarios**:

1. **Given** any workspace, **When** the user docks/tabs/resizes/floats/maximizes/closes a panel, **Then** the operation applies to the focused panel and View › Panels reopens it.
2. **Given** Alt+1..4 (or the View menu), **When** the user switches workspaces, **Then** the layout changes to the WS-01..04 arrangement in 07 §7.3 and the active id persists (`alvs.workspaces.active`).
3. **Given** a modified layout, **When** the app restarts, **Then** the per-workspace layout persists; "Reset workspace" restores the documented default.
4. **Given** the Edit workspace at 1920×1080, **When** it renders, **Then** region defaults follow 07 §7.2 (left 22%, viewers 56%, Inspector 22%; upper 55%, timeline 45%).

---

### User Story 5 - Notifications that respect the user's flow (Priority: P2)

Non-modal toast notifications plus a notification center. Core events (`app:notify`, `jobs:update`, `autosave:status`, `model:dirty` from 04 §4.10) drive the UI: error toasts show the error code (12 §12.13) and a recovery action button when the envelope carries one; the notification center keeps the history; the status bar shows the autosave chip and job summary framework. Message style follows 07 §7.10 (sentence case, verb-first, no blame, one recovery action). (REQ-APP-006; 04 §4.10; 07 §7.10.)

**Why this priority**: the notification fabric carries every later work package's feedback (jobs, exports, autosave) — and the round-1 review's T048 deferral (assertive errors, dismiss, pause) is tracked there, not silently dropped.

**Independent Test**: T-APP-005 — toasts, notification center, error envelope rendering with recovery.

**Acceptance Scenarios**:

1. **Given** an `app:notify` error event, **When** the toast shows, **Then** the localized message, the E-code, and — when the envelope has a recovery action — its button render; activating recovery runs it.
2. **Given** any toast, **When** the user opens the notification center, **Then** the history lists entries with severity and time.
3. **Given** `autosave:status` transitions, **When** the status bar updates, **Then** the chip reflects idle/saving/saved/failed with the last save time (07 §7.10 "Unsaved" row).
4. **Given** the known WP-04 deferral T048 (assertive routing, dismiss button, pause on hover/focus), **When** WP-05 ships its toast, **Then** it inherits the WP-04 Toast component behavior; improving it stays tracked as T048.

---

### User Story 6 - Keyboard-first shell accessibility (Priority: P2)

Every shell function is keyboard-operable with visible focus and no traps: F6 / Shift+F6 cycle panels (KB-133/134), ` maximizes the focused panel (KB-132), Shift+1..9 focus the nine catalog panels (KB-135..143), and screen readers get names/roles/values/states plus live-region announcements for shell feedback (one focused panel shows the 1 px accent line on its tab per 07 §7.9.1). (REQ-APP-011, REQ-A11Y-001, REQ-A11Y-002 shell share; 07 §7.9.1-7.9.2.)

**Why this priority**: the shell owns the focus model every panel inherits; retrofitting traps later is costlier than building the cycle now.

**Independent Test**: T-A11Y-001 — keyboard-only operation, F6, maximize panel.

**Acceptance Scenarios**:

1. **Given** the shell, **When** the user presses F6/Shift+F6, **Then** panel focus cycles in logical order with a visible focus indication and the focused panel's tab shows the accent line.
2. **Given** a focused panel, **When** the user presses `, **Then** it maximizes; pressing again restores.
3. **Given** keyboard-only operation, **When** the user completes Story 1-3 flows (menu → command → dialog), **Then** every step is reachable and visible-focus; no trap exists (Esc returns as documented in 07 §7.9.2).

---

### User Story 7 - One instance, hardened shell (Priority: P3)

Launching with a `.alvs` path while the app runs forwards the path to the running instance (`app:open-file`), which opens it — prompting DLG-20 first if the current project is dirty. One project is open at a time. The webview runs the 05 §5.4 CSP, capabilities grant exactly the documented permission set (dialogs, updater, core events/window; never fs/shell/http/process), the asset protocol scope is limited to the documented folders, and the `e2e-smoke` CI job activates with the T-E2E-001 steps that exist so far (launch → first-run → new project). (REQ-APP-008; 05 §5.4; T-SEC-005 CSP/capabilities part; card deliverable.)

**Why this priority**: single-instance and hardening close the shell's contract with the OS and the security doc; they gate the smoke suite but not the daily editing flows.

**Independent Test**: T-APP-006 — second launch forwards path, dirty prompt; T-SEC-005 (CSP/capabilities part).

**Acceptance Scenarios**:

1. **Given** a running instance, **When** a second launch occurs with a `.alvs` path, **Then** the second process exits and the first receives `app:open-file` and opens the project (dirty → DLG-20 first).
2. **Given** the built app, **When** the webview loads, **Then** the 05 §5.4 CSP applies verbatim, navigation off-origin is cancelled, DevTools are disabled in release, and capabilities match the documented list exactly.
3. **Given** CI, **When** `e2e-smoke` runs, **Then** it launches the app, completes the existing T-E2E-001 steps (first-run → new project), and fails if they break.

---

## Functional Requirements *(mandatory)*

Each FR cites its canonical source. "Shell panel placeholders" mean labeled, accessible, empty surfaces owned by the shell until their work package mounts real content (no fake data — G-02/§8.4).

### FR-001: Application modes and single instance — REQ-APP-008; 02 §2.3; 04 §4.10
`main.rs` parses the mode (`--crash-server` / `--worker` / `cli` / GUI) before any Tauri initialization exactly per 02 §2.3; WP-05 implements the GUI branch and the single-instance plugin: a second launch forwards its `.alvs` argument and exits; the running instance receives `app:open-file { path }` and opens it (dirty → DLG-20). One project open at a time.
- AC1: mode dispatch order and behavior compile-time match §2.3 (worker/crash/cli branches remain the stubs their WPs own — no behavior invented).
- AC2: second launch with a path exits 0 and the first instance opens that project (T-APP-006, e2e).
- AC3: dirty current project + forwarded open → DLG-20 resolves before the open proceeds.

### FR-002: Main window, custom title bar, DPI — REQ-APP-001; REQ-APP-010 (window); 07 §7.2
Main window with a custom Windows 11 style title bar (32 px) hosting the menu bar, native caption buttons area keeping Snap Layouts behavior, minimum size 1280×720, per-monitor DPI awareness. The title bar shows the project name and the dirty marker (•) per REQ-APP-012.
- AC1: caption buttons minimize/maximize/close work; maximize exposes Snap Layouts (verified manually, noted in evidence).
- AC2: resize below 1280×720 is refused.
- AC3: crisp at 100/125/150/175/200% DPI (T-APP-001 harness capture at the five steps).
- AC4: title shows "Project • Main ▾" pattern per 07 §7.2 (project + active sequence; sequence switcher ships with WP-12 — its absence must not fake state).

### FR-003: Menu bar — REQ-APP-001; 12 §12.11; 07 §7.9.1
The seven menus in order with items in KB order and the documented group separators and ✓ marks; items are commands from FR-008 (labels, shortcuts, enablement from the registry); context menus for shell surfaces (07 §7.8) mount where the card's surface owns them.
- AC1: each menu's items == the §12.11 groups in order, cross-checked against the registry (T-APP-003 completeness).
- AC2: checkable items (Snapping, Linked Selection, Loop, Play Every Frame, List View) show state from the command's checked state; unimplemented ones stay disabled with the later-version tooltip.

### FR-004: Toolbar — REQ-APP-001; 07 §7.2
40 px toolbar per the anatomy: tools (KB-091..097), undo/redo (KB-017/018), Snap (KB-070) and Link (KB-071) toggles, workspace switcher (WS-01..04), Export… (KB-008), notifications bell (REQ-APP-006), settings (KB-015), jobs (PNL-14 access). Icon-only buttons carry tooltips with label + shortcut (REQ-HLP-003).
- AC1: every toolbar control maps to a KB id; none is unregistered chrome.
- AC2: icon-only controls expose accessible names + tooltips with shortcuts (T-HLP-003 share).

### FR-005: Status bar framework — REQ-APP-007 (framework); 07 §7.2/§7.10; 04 §4.10
24 px status bar with the documented fields: background job summary, autosave state chip (from `autosave:status`), playback statistics (fps, dropped frames — from `playback:state` when it exists), proxy mode chip, renderer (GPU name or "WARP" from `app_info.renderer`). Fields whose data sources ship later (WP-07/WP-11) render in their defined place with a neutral "—" state, never fake values.
- AC1: each field exists, is localized, and updates from its event when present.
- AC2: `app_info` renderer and `autosave:status` drive their chips in the e2e/harness run.

### FR-006: Docking and panel operations — REQ-APP-002
dockview-based docking: dock, tab, resize, float inside the main window, maximize (`), close, reopen from View › Panels; layout persists per workspace across restarts (03 §3.6 storage).
- AC1: all six operations work on shell panels (T-APP-002).
- AC2: layout round-trips through restart per workspace.

### FR-007: Built-in workspaces — REQ-APP-003 (built-in share); 07 §7.2-7.3
WS-01..04 with the documented left/center/right/bottom arrangements and the Edit-region defaults; switching via View › Workspaces and Alt+1..4 (KB-125..128); Reset workspace (KB-130); persistence keys `alvs.workspaces.active`/`alvs.workspaces.saved` wired for WP-24. Below 1600 px width the viewers' merge behavior is noted as PLY territory (REQ-PLY-001) — the shell must not block it.
- AC1: Alt+1..4 and the menu switch layouts; active id persists.
- AC2: Reset restores the 07 §7.3 table; the dockview JSON round-trips.

### FR-008: Command registry — REQ-APP-004; 12 §12.10
All 168 command ids from 12 §12.10 registered with id, label key, menu path, context (`global`/`timeline`/`viewer`/`media`), default shortcut (physical `code` sequence), checked/enable state. Commands unimplemented in any shipped WP are disabled with the localized "Available in a later version" tooltip; the implementing WP removes that marker (card rule). Enablement also honors the focused-panel context (07 §7.9.1).
- AC1: registry enumeration == KB-001..168 exactly (T-APP-003 unit check against the catalog table).
- AC2: context-scoped commands enable only with a matching focused panel (e.g. `timeline.*` with the timeline placeholder focused).
- AC3: zero conflicts in the default map within any context (grounding T-SET-002 later).

### FR-009: Command palette — REQ-APP-004
Ctrl+Shift+P (KB-131) opens the palette: fuzzy search over labels + ids, shortcut display, keyboard-navigable list, Enter runs the enabled selection; disabled commands show the tooltip and cannot run.
- AC1: fuzzy ranking finds by label substring/initials in all six locales (T-APP-003).
- AC2: palette is fully keyboard operable with visible focus.

### FR-010: Keyboard dispatcher — REQ-I18N-005; 12 §12.10 preamble; 07 §7.9.2
Dispatch by `KeyboardEvent.code` with the documented exception model: editable fields receive all keys except Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11; single-key shortcuts suppressed during IME composition; displayed labels use `navigator.keyboard.getLayoutMap()` when available; Esc cancels the field edit, second Esc returns focus to the panel.
- AC1: AZERTY/QWERTY/Arabic layout presses of the same physical positions fire identical commands (T-I18N-005).
- AC2: the text-field exception list and IME suppression behave exactly as §7.9.2 (unit + harness).

### FR-011: Welcome screen — REQ-APP-005
Startup with no restored project shows Welcome: New Project, Open Project, recents with thumbnails and missing indicator, Getting Started link (opens help — HLP territory, link target per Help menu id).
- AC1: the three recents states (none / list / missing) render correctly (T-APP-004).
- AC2: recents come from `project_recent`; a stale path shows the missing indicator, never an error toast.

### FR-012: New Project dialog (DLG-01) — REQ-PRJ-001; 07 §7.14 DLG-01; 04 §4.3.2
Fields: name (required 1-200, Windows-safe), location (folder picker, default from settings), sequence preset (12 §12.8 with the "Auto — match first clip" default and 1080p30 fallback), frame rate (12 §12.9), sample rate 48 kHz default; Create (Enter)/Cancel (Esc); creates `<location>\<name>\<name>.alvs` via `project_new` and handles E1008/E8004 per the envelope.
- AC1: valid submission creates + opens the project; title bar updates.
- AC2: invalid name rejected inline with the localized rule; both error codes surface as DLG-level messages with recovery (07 §7.10 style).

### FR-013: Project lifecycle wiring and DLG-20 — REQ-APP-012; 04 §4.3.2; 07 §7.14 DLG-20
File commands wire to WP-03 services: `file.newProject` (DLG-01), `file.openProject` (picker + `project_open`, surfacing read-only (E1007 path) with the 07 §7.10 yellow banner and lock take-over via `project_take_over_lock`), `file.save`/`file.saveAs` (`project_save`/`project_save_as`), `file.closeProject` and `file.exit` run the DLG-20 gate (Save/Don't save/Cancel → `project_close { discardChanges }` / `app_quit`). Dirty state from `model:dirty` drives the title-bar dot.
- AC1: each wired command calls exactly its §4.3.2 command through the typed wrapper (ADR-010 bindings).
- AC2: DLG-20 three-way outcome behaves per REQ-APP-012 in all three triggers (close, exit, open-other) — T-APP-007.
- AC3: read-only open shows the banner with the Save As button; take-over flow completes.

### FR-014: Notifications — REQ-APP-006; 04 §4.10; 07 §7.10
Toast + notification-center fabric driven by `app:notify` (ErrorEnvelope or info `{ severity, messageKey, params }`), with the error code shown and the envelope's recovery action as a button; job/autosave events feed FR-005. Built on the WP-04 Toast component; the T048 improvements remain tracked (assertive errors, dismiss, pause).
- AC1: error envelope renders code + recovery; action runs the recovery (T-APP-005).
- AC2: center history lists entries; info messages route politely (errors' assertive routing stays T048).

### FR-015: Shell accessibility — REQ-APP-011; REQ-A11Y-001/002 (shell share); 07 §7.9.1-7.9.2
F6/Shift+F6 panel cycle, ` maximize, Shift+1..9 focus commands (KB-135..143), visible focus everywhere, logical tab order, no traps; the focused panel's tab shows the 1 px accent line; shell live regions announce feedback (07 §7.9.3's announcement pattern for shell-level results).
- AC1: keyboard-only walkthrough of Stories 1-3 succeeds (T-A11Y-001).
- AC2: focus cycle order is stable and announced; `.focus-visible` rings per WP-04 tokens.

### FR-016: IPC integration — 04 §4.2/§4.3.1-4.3.2; ADR-010
`app_info`, `app_ready`, `app_quit` and the §4.3.2 project commands used above are exposed as Rust commands with validated inputs and `Result<T, ErrorEnvelope>`, called only through typed wrappers in `app/src/ipc/commands.ts` over regenerated ts-rs bindings; binary/pattern rules per §4.2; events subscribed per §4.10.
- AC1: `cargo test export_bindings` clean; CI bindings-drift green.
- AC2: no command bypasses the typed wrapper (lint/grep check).

### FR-017: Webview hardening — 05 §5.4; T-SEC-005 (CSP/capabilities part)
CSP exactly as 05 §5.4; capabilities exactly the documented set (own commands + `dialog:allow-open|save|message|ask`, `updater:default`, `core:event:default`, `core:window:default`); no fs/shell/http/process grants; asset protocol scoped to the documented folders; navigation lock and `window.open` denial; DevTools disabled in release.
- AC1: capability JSON and tauri.conf CSP match §5.4 verbatim (unit check over the config).
- AC2: T-SEC-005's CSP/capabilities assertions pass.

### FR-018: e2e-smoke activation — card deliverable; 09 §9.9
The `e2e-smoke` CI job replaces its no-op with the T-E2E-001 steps that exist: launch → first-run (DLG-19 territory is Settings/WP-24 — if first-run setup is not shipped, the step asserts the Welcome path) → new project; steps for import/edit/export assert nothing fake and are added by their WPs.
- AC1: job runs the real app, completes the existing steps, and fails on regression.
- AC2: the job documents which T-E2E-001 steps remain for later WPs.

## Requirements Traceability *(mandatory)*

| REQ | FRs | Test IDs |
|---|---|---|
| REQ-APP-001 | FR-002, FR-003, FR-004, FR-005 | T-APP-001 |
| REQ-APP-002 | FR-006 | T-APP-002 |
| REQ-APP-003 (built-ins) | FR-007 | T-APP-002 |
| REQ-APP-004 | FR-008, FR-009 | T-APP-003 |
| REQ-APP-005 | FR-011 | T-APP-004 |
| REQ-APP-006 | FR-014 | T-APP-005 |
| REQ-APP-007 (framework) | FR-005 | T-APP-001 |
| REQ-APP-008 | FR-001 | T-APP-006 |
| REQ-APP-010 (window DPI) | FR-002 | T-UI-002 share |
| REQ-APP-011 | FR-015 | T-A11Y-001 |
| REQ-APP-012 | FR-013 | T-APP-007 |
| REQ-PRJ-001 (DLG-01) | FR-012 | T-PRJ-001 share |
| REQ-I18N-005 | FR-010 | T-I18N-005 |
| REQ-A11Y-001 / REQ-A11Y-002 (shell) | FR-015 | T-A11Y-001 |
| REQ-HLP-003 | FR-004 | T-HLP-003 share |
| 05 §5.4 hardening | FR-017 | T-SEC-005 (part) |
| 02 §2.3 modes | FR-001 | build/e2e |
| 04 §4.10 events | FR-005, FR-013, FR-014 | harness/e2e |

## Success Criteria *(mandatory)*

- SC-001: The shipped app presents the complete 07 §7.2 anatomy with all seven menus correct to 12 §12.11 and all 168 commands registered (T-APP-001/003 green).
- SC-002: A user can create, save, close, reopen, and safely abandon a project entirely through the shell, with dirty-state and unsaved-changes behavior exactly per REQ-APP-012 (T-APP-004/005/006/007 green).
- SC-003: The keyboard works identically across physical layouts, the palette finds commands in all six locales, and the whole shell is operable keyboard-only (T-I18N-005, T-A11Y-001 green).
- SC-004: The webview runs the documented CSP/capabilities with nothing extra granted (T-SEC-005 part green; unit config check).
- SC-005: CI's `e2e-smoke` exercises the real application for the T-E2E-001 steps that exist.

## Out of scope (per card/canonical WP columns)

Panel contents beyond shell placeholders (WP-08/09/12/13/16/17), custom workspace management (WP-24 share of REQ-APP-003), status-bar live values for jobs/playback/proxies (WP-07/WP-11), shortcut editor DLG-06 (Settings WP), first-run setup DLG-19 (Settings WP), crash recovery dialog DLG-15 UI beyond invoking WP-03's recovery service, external preview windows (WP-09), and the T048 toast improvements (tracked).
