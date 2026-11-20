# WP-05 Research — verified facts and decisions

All facts verified against the canonical docs (G-10); nothing below is invented.

## Verified facts

### Stack and mechanisms (docs/00 §0.8; docs/02)
- **Tauri 2.12.x** shell (crate 2.12.5, api/cli 2.12.4). Baseline plugins: `tauri-plugin-updater 2.14.x`, `tauri-plugin-dialog 2.8.x`, **`tauri-plugin-single-instance 2.5.x`** (00 §0.8 row: "Shell | tauri-plugin-updater / -dialog / -single-instance").
- **Single instance implementation is canonical**: 02 §2.5 runtime lifecycle step 4 — "Enforce single instance (tauri-plugin-single-instance): forward a project path to the running instance" → the forwarded path arrives as the `app:open-file { path }` event (04 §4.10).
- **dockview 8.8.x** is the documented docking engine (00 §0.8 "UI | dockview | Docking layout"); workspaces persist as dockview JSON in `alvs.workspaces.saved` (array of `{ id, name, layout }`, version 1) and `alvs.workspaces.active` (`WS-01..04` or a saved id, default `WS-01`) — 03 §3.6 rows.
- **Process model** (02 §2.3): `main()` parses `--crash-server` / `--worker` / `cli` / GUI **before** any Tauri init; release sets `#![windows_subsystem = "windows"]`; CLI attaches the parent console. WP-05 implements the GUI branch; the other branches stay the property of their WPs.
- **tauri-driver 2.1.0** is the installed e2e driver (00 §0.8 dev tools) — the e2e-smoke job can drive the real app (WebView2) via WebDriver without new dependencies.

### Commands, menus, keyboard (docs/12 §12.10-12.11; docs/07 §7.9.2)
- The catalog has **KB-001..KB-168** with columns: id, command id, label, menu, default shortcut, context (`global`/`timeline`/`viewer`/`media`). The default map has **zero conflicts within any context** (12 §12.10 closing note, grounding T-SET-002).
- Menus (12 §12.11): order File, Edit, Clip, Timeline, Playback, View, Help; items in KB order; group separators per the §12.11 table; ✓ marks on Snapping, Linked Selection, Loop, Play Every Frame, List View; `view.inspectSelection` (KB-168) is palette-only.
- Keyboard rules (12 §12.10 preamble + 07 §7.9.2): dispatch by `KeyboardEvent.code` (physical); text-field exceptions exactly Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11; single-key shortcuts suppressed during IME composition; labels via `navigator.keyboard.getLayoutMap()` when available, else US label; Esc cancels field edit, second Esc returns to panel.
- Context enablement (07 §7.9.1): one focused panel; non-global commands act on the focused panel.

### IPC surface for this WP (docs/04 §4.2, §4.3.1-4.3.2, §4.10)
- App commands: `app_info` (→ `AppInfo { version, commit, buildDate, identifier, renderer { name, isWarp }, safeMode, portable, locale }`), `app_ready`, `app_quit { force }` (shutdown per 02 §2.5).
- Project commands (WP-03 services, consumed here): `project_new { name, folder, sequencePreset, frameRate, sampleRate } → ProjectSnapshot` (E1008, E8004); `project_open { path }` (E1001–E1005, E1007); `project_open_readonly`; `project_take_over_lock`; `project_save`/`project_save_as → SaveResult { path, backupPath }`; `project_close { discardChanges }`; `project_snapshot`; `project_recent → RecentProject[]`; `project_recovery_info`/`project_recover`; `project_backups`/`project_restore_backup`; `project_set_proxy_mode`.
- Events consumed by the shell: `app:notify`, `app:open-file`, `model:dirty`, `autosave:status { state, timeUtc }`, `jobs:update` (summary framework), `playback:state` (fps/dropped when present), `renderer:changed`.
- Conventions: typed wrappers only (`app/src/ipc/commands.ts`), ts-rs bindings (ADR-010), validation + `Result<T, ErrorEnvelope>`, no main-thread blocking.

### Shell anatomy (docs/07 §7.2-7.3, §7.10, §7.14)
- Title bar 32 px (hosts menu bar; native caption area keeps Snap Layouts); toolbar 40 px; status bar 24 px; min window 1280×720; Edit-region defaults at 1920×1080: left 22% / viewers 56% / Inspector 22%; upper 55% / timeline 45%.
- Toolbar contents from the anatomy: tools KB-091..097, ⟲⟳ (undo/redo), Snap ⊡ Link ⛓, workspace dropdown, Export…, 🔔 ⚙ jobs.
- Status bar fields (07 §7.2 bottom + §7.10): job summary, autosave state ("Unsaved" row: title dot + autosave chip with last time), playback stats (fps + dropped), proxy chip, renderer (GPU name or WARP).
- DLG-01 New Project fields and DLG-20 three-way prompt as quoted in the spec (07 §7.14). Dialog sizing: content-driven, min 420 px, max 80% window, size remembered.
- States (07 §7.10): read-only yellow banner with Save As; dirty dot; toasts with code + recovery.
- Welcome (REQ-APP-005): New/Open/recents-with-thumbnails + missing indicator + Getting Started.

### Hardening (docs/05 §5.4)
CSP string, capability list (`dialog:allow-open|save|message|ask`, `updater:default`, `core:event:default`, `core:window:default`; **not** fs/shell/http/process), asset protocol scope (`$APPLOCALDATA/cache/thumbs/**` + bundled `assets/`), navigation lock, `window.open` denied, DevTools off in release — all quoted verbatim in FR-017.

### Tests (docs/09 §9.9-9.10)
T-APP-001..007 definitions; T-I18N-005 (AZERTY/QWERTZ/Arabic physical keys); T-A11Y-001 (keyboard-only, F6, maximize); T-SEC-005 includes CSP/capabilities; T-E2E-001 smoke = launch → first-run → new project → import → edit → export (later steps owned by their WPs; the CI no-op comment says "until WP-05 replaces this step with the real smoke suite").

## Decisions (for plan.md)

- **D1 Catalog fixture**: the 168-command table is transcribed once into a typed TS module (id, labelKey, menu, shortcut `code`s, context, implemented-in) plus a checked fixture derived from 12 §12.10; a unit test asserts registry == fixture, and the fixture file header cites the doc. Prevents silent drift; the completeness check is automatable without shipping docs.
- **D2 Dispatcher**: one window-level keydown listener dispatching on `KeyboardEvent.code`; contexts resolved from the focused-panel registry (07 §7.9.1); text-field exceptions and IME rules implemented exactly per §7.9.2.
- **D3 Placeholders**: dockview panels for not-yet-shipped panels are labeled, accessible, empty surfaces (no fake data) — the card's "unimplemented commands disabled" principle extended to panels; each placeholder names the owning WP.
- **D4 e2e driver**: `e2e-smoke` drives the real app through tauri-driver (already a dev tool); steps = the T-E2E-001 prefix that exists; later steps append with their WPs.
- **D5 Hardening test**: a unit test parses `tauri.conf.json` + `capabilities/*.json` and asserts equality with the 05 §5.4 values (CSP string verbatim, exact permission ids, asset scope) so config drift fails a gate rather than a review.
