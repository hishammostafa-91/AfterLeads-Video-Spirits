# 07 — Experience

Canonical for: UX principles, layout, workspaces (WS), design tokens, components, panels (PNL), dialogs (DLG), interaction rules, states and feedback, right-to-left rules, localization guidelines and terminology, accessibility rules, onboarding.
Depends on: [01-product.md](01-product.md) (requirements), [12-catalogs.md](12-catalogs.md) (commands, shortcuts, menus, colors), [04-interfaces.md](04-interfaces.md) (commands/events the UI calls).

---

## 7.1 Principles (UXP)

| ID | Principle | Practical rule |
|---|---|---|
| UXP-01 | The picture and the timeline come first | Viewer + timeline occupy ≥ 60% of the window in every built-in workspace. |
| UXP-02 | Calm by default, powerful on demand | Advanced controls live in collapsible sections, context menus, and the command palette; defaults are sensible. |
| UXP-03 | Never block the user | Long work runs as jobs with progress; modals only when data could be lost or input is required. |
| UXP-04 | Every action is reversible | Undo for all edits; destructive non-edit actions (delete files from cache, discard autosave) confirm first. |
| UXP-05 | Keyboard equivalence | Everything doable with the mouse is doable with the keyboard (REQ-A11Y-001). |
| UXP-06 | Honest status | Proxy mode, offline media, read-only, safe mode, reduced preview, background work are always visible. |
| UXP-07 | One language, all the way | Fully translated UI, correct RTL, consistent terminology (7.12). |
| UXP-08 | Fast feel | Interactions respond within 100 ms; anything slower shows progress within 300 ms. |

---

## 7.2 Window anatomy (Edit workspace, LTR)

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ [logo] File Edit Clip Timeline Playback View Help   Project • Main ▾        ─  ▢  ✕           │ title bar (32 px)
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ [V][B][T][Y][U][H][Z] │ ⟲ ⟳ │ Snap ⊡ Link ⛓ │ Workspace: Edit ▾ │      [ Export… ]  🔔 ⚙ jobs │ toolbar (40 px)
├──────────────────────┬──────────────────────────────┬───────────────────────────────────────┤
│ Media │Effects│Trans.│ Source viewer                │ Program viewer          │ Inspector     │
│ │Titles              │                              │                         │ (PNL-05)      │
│  [search……] ▦ ☰     │       (PNL-07)               │        (PNL-06)         │ ▸ Video       │
│  ▸ Footage           │                              │                         │   Transform   │
│  ▸ Music             │ 00:00:12:04 ⏮ ◀ ▶ ▶▶ ⏭ [{ }]  │ 00:01:02:10 ⏮ ◀ ▶ ▶▶ ⏭  │   Crop …      │
├──────────────────────┴──────────────────────────────┴─────────────────────────┴───────────────┤
│ Main ×  Vertical ×   │ tools… zoom ━━●━━  │ ruler 00:00 ─── 00:10 ─── 00:20 ─── 00:30       │
│ C1 [CC] 🔒 👁          │ ▭▭ captions                                                   │meters│
│ V2 ◎ 🔒 👁            │      [Title  ]                                                │ ▮▮   │
│ V1 ◎ 🔒 👁            │ [Interview A ▣▣▣▣][Broll▣▣]  ⟋⟍ [Interview B ▣▣▣▣▣▣]          │ ▮▮   │
│ A1 ◎ 🔒 M S           │ [~~~~~~~~~~~~~~~~][~~~~~~]       [~~~~~~~~~~~~~~~~~~]          │ ▮▮   │
│ A2 ◎ 🔒 M S           │      [music ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~]        │      │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ ● Saved 12:30 │ Proxies ON │ 29.97 fps • 0 dropped │ GPU: NVIDIA RTX 3060 │ 2 jobs running  │ status bar (24 px)
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Minimum window 1280×720 (REQ-APP-001). Below 1600 px width the Source and Program viewers merge into one tabbed viewer (REQ-PLY-001).
- Region defaults (Edit workspace at 1920×1080): left dock 22%, viewers 56% (split equally), Inspector 22%; upper area 55% height, timeline 45%.
- The title bar hosts the menu bar; the system caption buttons area keeps native Snap Layouts behavior (REQ-APP-001).

---

## 7.3 Workspaces (WS)

| ID | Name | Left | Center | Right | Bottom |
|---|---|---|---|---|---|
| WS-01 | Edit | Media · Effects · Transitions · Titles (tabs) | Source + Program viewers | Inspector · History (tabs) | Timeline |
| WS-02 | Color | Scopes (2×2) | Program viewer (large) | Inspector (color controls expanded, larger wheels) | Timeline (reduced height 30%) |
| WS-03 | Audio | Media · Effects (audio category) | Program viewer (small) | Inspector (audio section) | Mixer (35%) above Timeline (waveforms enlarged) |
| WS-04 | Captions | Captions panel (wide) · Transcript | Program viewer | Inspector (caption style) | Timeline (caption lanes expanded) |

Every panel can be opened from View › Panels in any workspace. Layout changes persist per workspace (REQ-APP-002); "Reset workspace" restores the table above. Saved workspaces store the dockview JSON (03 §3.6 `alvs.workspaces.saved`).

---

## 7.4 Design tokens

Tokens are CSS custom properties in `app/src/design/tokens.css`; components reference tokens only (no raw colors). Contrast ratios below are against the token's usual background and satisfy REQ-A11Y-003.

### 7.4.1 Color — Dark theme (default)

| Token | Value | Use / contrast |
|---|---|---|
| `--bg-app` | `#111214` | Window background |
| `--bg-panel` | `#17181B` | Panel body |
| `--bg-panel-header` | `#1C1D21` | Panel tabs/headers |
| `--bg-raised` | `#222328` | Inputs, cards, menus |
| `--bg-hover` | `#2A2B31` | Hover |
| `--bg-active` | `#33343B` | Pressed / selected row |
| `--border-subtle` | `#2A2B30` | Separators |
| `--border-strong` | `#3A3B42` | Input borders (3.3:1 vs panel) |
| `--text-primary` | `#ECEDEE` | Body text (≈ 15:1) |
| `--text-secondary` | `#A9ABB3` | Labels (≈ 7.6:1) |
| `--text-tertiary` | `#83858E` | Hints (≈ 4.7:1) |
| `--text-disabled` | `#5C5E66` | Disabled (exempt) |
| `--accent-solid` | accent base (default `#3E63DD`) | Primary buttons (white text ≈ 5.3:1) |
| `--accent-text` | `#8DA4EF` | Links, selected text (≈ 7.4:1) |
| `--accent-subtle` | `#1E2A55` | Selection backgrounds |
| `--focus-ring` | `#8DA4EF` | 2 px outline + 1 px offset (≥ 3:1) |
| `--danger` / `--danger-text` | `#E5484D` / `#FF9592` | Errors, record |
| `--warning` / `--warning-text` | `#FFC53D` / `#FFD885` | Warnings |
| `--success` / `--success-text` | `#30A46C` / `#3DD68C` | Success |
| `--info` | `#05A2C2` | Info |

Accent alternatives (REQ-APP-009) swap `--accent-solid`/`--accent-text`/`--accent-subtle` with: purple `#8E4EC6`/`#C29FEB`/`#2F1E45`, green `#30A46C`/`#5BD39B`/`#16301F`, orange `#F76B15`/`#FFA066`/`#3A2010`, pink `#D6409F`/`#F08DC6`/`#3A1730`.

### 7.4.2 Color — Light theme

| Token | Value | Token | Value |
|---|---|---|---|
| `--bg-app` | `#F3F4F6` | `--text-primary` | `#1A1B1E` |
| `--bg-panel` | `#FFFFFF` | `--text-secondary` | `#4B4E57` |
| `--bg-panel-header` | `#F7F8FA` | `--text-tertiary` | `#6A6D76` (≈ 5.3:1) |
| `--bg-raised` | `#FFFFFF` | `--text-disabled` | `#A3A6AE` |
| `--bg-hover` | `#EEF0F3` | `--accent-solid` | `#3E63DD` |
| `--bg-active` | `#E3E6EB` | `--accent-text` | `#3451C4` (≈ 6.5:1) |
| `--border-subtle` | `#E4E6EA` | `--accent-subtle` | `#E6ECFD` |
| `--border-strong` | `#B9BCC4` | `--focus-ring` | `#3E63DD` |

Status colors are the same hues with darker text variants: danger text `#C62A2F`, warning text `#8A5A00`, success text `#18794E`.

### 7.4.3 High Contrast

When `forced-colors: active` (Windows contrast themes) or the user selects High Contrast, tokens map to system colors: backgrounds `Canvas`, text `CanvasText`, accents and selection `Highlight`/`HighlightText`, controls `ButtonFace`/`ButtonText`, disabled `GrayText`, links `LinkText`. Clip colors become outlined shapes with text labels; meters use `CanvasText` fills with patterns for clip and peak states.

### 7.4.4 Timeline and media colors

| Token | Dark | Light | Item |
|---|---|---|---|
| `--tl-bg` | `#0E0F11` | `#E9EBEF` | Track area |
| `--tl-track-alt` | `#131417` | `#E2E5EA` | Alternating track background |
| `--tl-video` / border | `#2F4E8C` / `#4A6FBF` | `#9DB4EC` / `#3E63DD` | Video clips |
| `--tl-audio` / border | `#2D6B4F` / `#3E8E69` | `#9ED9BC` / `#2C8C5E` | Audio clips |
| `--tl-title` / border | `#5B3E8C` / `#7A58B5` | `#CDB6EE` / `#7A58B5` | Titles |
| `--tl-generator` / border | `#8C5A2E` / `#B57A45` | `#F2C9A5` / `#B57A45` | Generators, adjustment layers |
| `--tl-nested` / border | `#2E7480` / `#46A2B0` | `#A6DCE4` / `#2E8A99` | Nested sequences |
| `--tl-caption` / border | `#7A2E5E` / `#A84A85` | `#EBB3D5` / `#A84A85` | Captions |
| `--tl-playhead` | `#F76B15` | `#E5560A` | Playhead |
| `--tl-inout` | `rgba(62, 99, 221, 0.18)` | `rgba(62, 99, 221, 0.15)` | In–out range |
| `--tl-selected` | `#FFFFFF` 2 px outline | `#1A1B1E` 2 px outline | Selected clip |
| `--tl-offline` | `#E5484D` diagonal stripes | same | Offline media |

A user label color (CLR-01..08) replaces the clip body color while keeping the type icon.

### 7.4.5 Typography

| Token | Value |
|---|---|
| `--font-ui` | `"Inter Variable", "Noto Sans Arabic Variable", "Segoe UI Variable", "Segoe UI", system-ui, sans-serif` |
| `--font-mono` | `"JetBrains Mono", "Cascadia Mono", Consolas, monospace` (timecode, numbers; `font-variant-numeric: tabular-nums`) |
| Sizes | `--fs-xs 11px` badges · `--fs-sm 12px` dense labels · `--fs-md 13px` default · `--fs-lg 15px` panel titles · `--fs-xl 18px` dialog titles · `--fs-2xl 24px` welcome |
| Line height | 1.4 (`:lang(ar)` 1.6 and +1 px size) |
| Weights | 400 regular, 500 medium (labels), 600 semibold (headers) |

### 7.4.6 Space, shape, elevation, motion

- Spacing scale: `--sp-1 2px`, `--sp-2 4px`, `--sp-3 8px`, `--sp-4 12px`, `--sp-5 16px`, `--sp-6 24px`, `--sp-7 32px`.
- Control height: 28 px comfortable / 24 px compact; minimum target 24×24 px (WCAG 2.2 target size).
- Radius: `--radius-sm 3px` (inputs), `--radius-md 6px` (buttons, cards), `--radius-lg 10px` (dialogs).
- Elevation: menus/popovers `0 8px 24px rgba(0,0,0,0.35)`; dialogs `0 16px 48px rgba(0,0,0,0.45)`; light theme halves the alpha.
- Motion: `--dur-fast 120ms`, `--dur-base 180ms`, `--dur-slow 260ms`, easing `cubic-bezier(0.2, 0, 0, 1)`; with reduced motion all durations are 0 except progress indicators (REQ-A11Y-004).
- Z-order: panels < floating panels < popovers/menus < toasts < dialogs < drag ghost.

---

## 7.5 Iconography

- Base set: Lucide (ISC license), 16 px in dense UI, 20 px in toolbars, stroke 1.75.
- Custom icons drawn in the same grid and stroke (stored in `assets/icons/`, original works): blade, trim-ripple, trim-roll, slip, slide, link, unlink, keyframe, keyframe-add, stopwatch, snapping, proxy, safe-areas, mask-rect, mask-ellipse, mask-pen, caption, transcribe, voice-over, ducking, loudness, adjustment-layer, nested-sequence.
- Every icon-only button has a tooltip with its label and shortcut (REQ-HLP-003) and an accessible name.
- Mirroring in RTL follows 7.11.

---

## 7.6 Components

All interactive primitives are built on React Aria Components (keyboard, focus, ARIA, RTL handled). Each component has stories in the UI harness (Playwright) for: default, hover, focus-visible, pressed, disabled, error, RTL, compact density, and High Contrast.

| Component | Notes |
|---|---|
| Button (primary, secondary, ghost, danger), IconButton, ToggleButton, SegmentedControl | Primary used once per dialog; danger requires confirmation text for irreversible actions |
| MenuBar, Menu, ContextMenu, Submenu | Shows shortcuts right-aligned (left in RTL); checkable items for toggles |
| Tabs (panel tabs) | Drag to rearrange (dockview); middle-click closes floating tabs |
| Dialog, AlertDialog, Popover, Tooltip (500 ms delay), Toast | Dialogs trap focus, Esc closes (except unsaved-changes prompts which need an explicit choice) |
| NumberField (scrubbable) | Drag horizontally to change (Shift ×10, Ctrl ×0.1), wheel with focus, arrow keys, typed math (`+10`, `*2`), unit suffix, mixed state "—" |
| Slider, AngleDial | Paired with a NumberField; double-click resets to default |
| ColorPicker | HSV square + hue slider + hex/RGB fields + eyedropper (samples the viewer, REQ-FX-012) + recent colors |
| ColorWheel | Chroma puck + luma slider; double-click resets; Alt-drag fine control |
| CurveEditor | Add point (click), delete (double-click or Delete), drag; channel tabs; reset |
| PointControl | Numeric x/y in pixels + on-viewer crosshair handle |
| TimecodeField | Segmented `HH:MM:SS:FF` with typed entry, `+`/`−` relative entry, always LTR (7.11) |
| KeyframeToggle (stopwatch), KeyframeNavigator (◀ ◆ ▶) | Stopwatch on = animated; diamond filled when a keyframe exists at the playhead |
| SearchField, TextField, TextArea | `dir="auto"` for user content (7.11) |
| Checkbox, Switch, RadioGroup, Select, ComboBox | — |
| Tree (folders), GridList (media grid), Table (lists), ListBox | Virtualized beyond 200 rows |
| ProgressBar, Meter (audio), Spinner, Skeleton | Indeterminate only before the first progress value |
| Badge (state chips: Proxy, Offline, 10-bit, HDR, VFR) | Text + icon, never color alone |
| EmptyState | Illustration-free: icon, one sentence, primary action, shortcut hint |

---

## 7.7 Panels (PNL)

| ID | Panel | Contents and behavior |
|---|---|---|
| PNL-01 | Media | Toolbar: Import (Ctrl+I), New Folder, view (grid/list), size slider, sort (name, date added, duration, type), search, kind filter. Breadcrumb for folders. Grid tiles: poster thumbnail with hover scrub, name, duration, badges (offline, proxy building/ready, optimized, stabilized, VFR, HDR, 10-bit), usage dot. List columns: name, duration, resolution, fps, codec, date added, label. Double-click opens in Source viewer; drag to timeline or folders; drop files from Explorer anywhere on the panel (highlighted drop zone). Empty state: "Import media" button + "or drop files here". |
| PNL-02 | Effects | Search; sections Video (Color, Blur & Sharpen, Keying, Stylize, Distort) and Audio (EQ, Dynamics, Restoration, Space, Utility); favorites (star); recent. Rows: icon, name, tooltip with description. Drag onto a clip or a track header (audio track effects); Enter or double-click applies to selected clips. |
| PNL-03 | Transitions | Video and Audio groups; each item shows an icon and name; drag onto an edit point (the edit point highlights and shows alignment start/center/end by drop position); double-click applies to selected edit points; the default transition is marked with a star (right-click › Set as default). |
| PNL-04 | Titles | "Basic title" first, then TPL-01..12 with rendered previews (generated once by the engine and cached), then user templates; drag to timeline or double-click to insert at the playhead on the highest targeted video track. |
| PNL-05 | Inspector | Header: clip name (editable), type icon, enable toggle, keyframe strip (clip duration with keyframes of all parameters). Sections (collapsible, remembered): Video › Transform, Crop, Opacity & Blend; Effects (stack with drag handles, enable eye, preset menu, reset, remove); Masks; Title (layers list + text editor, `dir="auto"`); Audio › Volume (gain + keyframable volume), Pan, Fades, Channels, Effects; Speed (summary + "Speed/Duration…"); Info (path, codec, resolution, frame rate). Each parameter row: label, stopwatch, control, reset (↺), keyframe navigator when animated. Multi-selection shows shared parameters with mixed values; edits apply to all (one undo step). Transition, caption, marker, or track selections show their own properties. With nothing selected: sequence settings summary and tips. |
| PNL-06 | Program viewer | Top bar: sequence switcher, zoom (Fit, 25–400%), preview resolution, overlays menu (safe areas, grid, center, checkerboard), compare split, full screen, external preview. Canvas with on-viewer handles for the selected clip (REQ-PLY-009) and title text editing (REQ-TTL-006). Bottom: editable timecode, duration of in–out, transport (go to in, previous frame, play reverse, play/pause, next frame, go to out), loop, mark in/out, dropped-frame indicator. Shows "Offline", "Reduced preview", and "Proxy" badges when relevant. |
| PNL-07 | Source viewer | Same as Program plus: source name and switcher (recent sources), insert (,) and overwrite (.) buttons, drag handles for video-only and audio-only, "Open in Media panel". Audio-only sources show a large waveform instead of video. |
| PNL-08 | Timeline | Sequence tabs; toolbar (tools KB-091..097, snapping, linked selection, add marker, zoom slider, track height presets S/M/L, thumbnails mode, list view toggle). Ruler (adaptive ticks, timecode labels, in–out range, sequence markers). Caption lanes on top, then video tracks (top = highest), then audio tracks, then the master meter column. Track header: name, source-patch target toggle, lock, visibility (video) / mute + solo (audio), height drag handle, context menu. Details in 7.7.1. |
| PNL-09 | Mixer | One strip per audio track: name, effects slots (add, bypass, open), pan knob, fader (−∞ to +12 dB, double-click = 0 dB), mute, solo, meter (peak + RMS, 1.5 s peak hold, clip LED). Master strip: fader, meter, loudness readouts (M, S, I, TP) with target selector (−14/−16/−23 LUFS) and color status. |
| PNL-10 | Scopes | Scope selector per cell (luma waveform, RGB parade, vectorscope, histogram), layout 1/2/4, intensity, graticule in % (0–100) with 10-bit code values on hover, source = Program (default) or Source viewer. |
| PNL-11 | Captions | Track selector with language; table (index, start, end, duration, text, warning icon for readability rules); inline text editing (`dir="auto"`); toolbar: add at playhead, split at cursor, merge, delete, import, export, transcribe, find & replace; clicking a row seeks; warnings explain the rule (REQ-CAP-007). |
| PNL-12 | Markers | Table (time, duration, name, color, kind, note); filters by color and kind; inline editing; export chapters/CSV; double-click seeks. |
| PNL-13 | History | Undo steps with localized labels and times; current step highlighted; click to jump (REQ-PRJ-007). |
| PNL-14 | Jobs | Rows: kind icon, title, state, progress bar, ETA, actions (cancel, retry, open output, show log); pause/resume queue; clear finished; filter (all, running, failed). |
| PNL-15 | Transcript | Media selector, search, paragraph text with word timestamps on hover, click a word to seek (REQ-STT-005), "Create captions from transcript". |

### 7.7.1 Timeline details

- **Geometry:** track header width 180 px (resizable 140–320); default video track 64 px, audio 56 px, caption lane 28 px; ruler 28 px; clips have 3 px corner radius and 1 px border; names truncate with ellipsis; minimum drawn clip width 2 px.
- **Clip rendering:** body color by type (7.4.4); thumbnails per `alvs.timeline.thumbnails`; waveform (peak min/max) with the clip-gain line; badges (fx, speed %, reverse ⟲, proxy, offline); fade handles at top corners (audio and video); selected clip shows a 2 px outline and keyframe diamonds of the selected parameter row.
- **Hit zones:** clip edges 6 px = trim; edit point (between adjacent clips) 8 px = roll; transition body = select transition; fade handle 8 px square; clip body = move.
- **Cursors:** move, trim-in, trim-out, roll, slip, slide, blade, hand, zoom, not-allowed.
- **Tools:** Selection (V) moves/trims; Blade (B) splits at the hovered frame (Shift = all tracks); Trim (T) ripple at edges, roll at edit points; Slip (Y) changes source in/out; Slide (U) moves between neighbors; Hand (H) pans; Zoom (Z) click to zoom in, Alt-click out.
- **Feedback during gestures:** ghost of the moved clips, live delta tooltip (`+00:00:01:12`, frames), snap line (accent) with the snapped target name, invalid drop shown with a red outline and not-allowed cursor. During trims the Program viewer shows the two frames on each side of the edit (outgoing/incoming) side by side.
- **Drag modifiers:** default overwrite; Ctrl = insert (ripple); Alt = duplicate; Shift = constrain to the same time (vertical move only).
- **Zoom and scroll:** wheel scrolls tracks vertically, Shift+wheel horizontally, Alt+wheel zooms around the mouse; zoom range 1 frame per 64 px … whole sequence in view.
- **Transitions:** drawn as a lighter overlay spanning the transition range with a bow-tie icon; drag its edges to change duration.

---

## 7.8 Context menus

| Target | Items (command IDs from 12 §12.10 unless noted) |
|---|---|
| Media item | Open in Source · Insert/Overwrite at playhead · Rename · Label Color ▸ · Create Proxies · Optimize Media · Basic Stabilize · Transcribe · Relink · Show in Explorer · Properties · Remove from project |
| Folder (Media) | New Folder · Rename · Label Color ▸ · Remove |
| Clip | Cut · Copy · Paste Attributes · Duplicate · Delete · Ripple Delete · Split · Enable · Link/Unlink · Group/Ungroup · Speed/Duration · Reverse · Freeze Frame · Nest · Match Frame · Mask ▸ · Label Color ▸ · Normalize Loudness · Copy/Paste Grade · Reveal in Media · Properties |
| Edit point | Add Default Transition · Add Transition ▸ · Ripple Trim Previous/Next |
| Transition | Duration… · Alignment ▸ (start, center, end) · Replace with ▸ · Delete |
| Gap | Close Gap · Delete All Gaps on Track |
| Track header | Rename · Add Track Above/Below · Delete Track · Lock · Hide/Mute · Solo · Height ▸ · Add Track Effect (audio) |
| Caption (timeline or panel) | Edit Text · Split at Playhead · Merge with Next · Delete · Style… |
| Marker | Edit… · Color ▸ · Convert to Chapter/Standard · Delete |
| Viewer | Zoom ▸ · Preview Resolution ▸ · Overlays ▸ · Export Frame · Copy Timecode |

---

## 7.9 Interaction rules

### 7.9.1 Selection and focus

- One focused panel at a time, indicated by a 1 px accent line on its tab. Commands with context `timeline`, `viewer`, `media` act on the focused panel; `global` commands act everywhere.
- Selection is per panel (media items vs clips vs captions). The Inspector shows the selection of the most recently focused selection-bearing panel.

### 7.9.2 Keyboard handling

- Shortcuts dispatch by `KeyboardEvent.code` (REQ-I18N-005). The displayed key label uses `navigator.keyboard.getLayoutMap()` when available, else the US label.
- While an editable text field has focus, it receives all keys except Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11; Esc first cancels the field edit, a second Esc returns focus to the panel.
- Single-key shortcuts (J, K, L, I, O, M, V, B…) are suppressed during IME composition.

### 7.9.3 Timeline keyboard model (REQ-EDT-013)

| Default keys (timeline focused) | Command and action |
|---|---|
| Ctrl+Left / Ctrl+Right | KB-160 / KB-161 — select the previous/next clip on the current track |
| Ctrl+Up / Ctrl+Down | KB-162 / KB-163 — move track focus up/down and select the clip under the playhead |
| Enter | KB-168 — move focus to the Inspector for the selected clip |
| Alt+Left/Right, Alt+Shift+Left/Right | KB-074..KB-077 — nudge selected clips by 1 or 10 frames |
| `[` / `]`, Shift+`[` / Shift+`]` | KB-164..KB-167 — trim the selected edge by 1 or 10 frames. An edge is selected by clicking near it with the Selection tool; with no selected edge the out point of each selected clip is used. Ripple applies when the Trim tool is active. |
| Ctrl+Alt+L | KB-087 — toggle the accessible list view (REQ-EDT-015) |

After each keyboard edit, a polite live region announces the result (for example "Interview A trimmed, new duration 00:00:07:12").

### 7.9.4 Editing values

- Changing a parameter of an animated property at a time without a keyframe creates a keyframe there (03 §3.4.7 semantics via `setParam` with `at`).
- Scrubbing a NumberField produces one undo step per gesture (committed on release).
- Invalid typed values revert and show an inline message; values are clamped to the range with a subtle flash.

### 7.9.5 Confirmations

Confirm only: removing media used in sequences, deleting a sequence, discarding unsaved changes, clearing caches, deleting models, overwriting files, taking over a project lock. Everything else is undoable or harmless.

### 7.9.6 Drag and drop

Accepted drops: files/folders from Explorer (Media panel, timeline, viewers); media items (timeline, Source viewer, folders); effects/transitions/titles (timeline targets, Inspector stack); panels (docking targets). Every drop target shows its highlight and an accessible alternative (menu command) exists.

---

## 7.10 States and feedback

| State | Treatment |
|---|---|
| Empty project | Media panel empty state + Timeline empty state ("Drag media here or press , to insert from the Source viewer") |
| Loading | Skeleton rows/tiles after 150 ms; panels never show a blank area |
| Background work | Status bar job summary; Jobs panel; per-item badges; completion toast (export: "Open folder" action) |
| Error | Toast with code and recovery button (REQ-APP-006); details expander; logged |
| Offline media | Red striped clips, "Offline" viewer card with Relink button, Media badge |
| Read-only project | Yellow banner under the toolbar: "Read-only — Save As to keep changes" with Save As button |
| Safe mode | Orange banner: "Safe mode — reduced performance" with "Restart normally" |
| Reduced preview / WARP | Status bar chip "Software rendering" and viewer badge |
| Proxy mode | Status bar chip "Proxies ON" and viewer badge "Proxy" |
| Unsaved | Title bar dot; autosave chip shows the last autosave time |

Message style: sentence case, verb-first actions ("Relink media"), no blame, no jargon without explanation, one recovery action when possible.

---

## 7.11 Right-to-left rules (REQ-I18N-002)

| Element | In Arabic (RTL) |
|---|---|
| Overall layout, docks, panel order, menus, dialogs, form labels, tab order | Mirrored (`dir="rtl"` on `<html>`; logical CSS properties only) |
| Directional icons (back/forward, chevrons for navigation, list indent, undo/redo arrows) | Mirrored |
| Timeline (time axis), ruler, clips, trims, keyframe strips, scopes, meters, waveforms | **Not mirrored** — time always flows left to right |
| Transport buttons (play, rewind, fast-forward, step) | **Not mirrored** |
| Timecode, frame numbers, durations, numeric fields | Western digits, wrapped in an LTR isolate (`<bdi dir="ltr">`) |
| User content (clip names, captions, notes, titles) | `dir="auto"`, bidi isolation in lists |
| Sliders | Value increases toward the inline end (mirrored), except audio faders (vertical) and timeline zoom (left = out, not mirrored) |
| Video in viewers | Never mirrored |
| Keyboard shortcuts | Same physical keys as LTR; `[`/`]` keep their physical meaning (earlier/later in time) |

Implementation: `app/src/i18n/direction.ts` sets `dir` and `lang`; components needing non-mirrored behavior set `dir="ltr"` explicitly with a comment referencing this table. Visual tests capture every panel in `ar` (T-I18N-002).

---

## 7.12 Localization guidelines

1. Message IDs: `area.component.purpose` (for example `timeline.toolbar.snapping`, `error.E2001`, `fx.color.basic.exposure`). IDs never change once released; text can.
2. English is the source (`app/src/locales/en.json`). Every other locale file has exactly the same key set (REQ-I18N-004).
3. Use ICU MessageFormat for plurals (`{count, plural, one {# clip} other {# clips}}`) and select; Arabic supports `zero/one/two/few/many/other`. Never concatenate translated fragments.
4. Every key has a translator description in `app/src/locales/en.meta.json` (context and maximum length when space-limited).
5. Layout budget: labels in toolbars and buttons must fit +40% text expansion (tested with LOC-90); long strings wrap or truncate with a tooltip.
6. Dates and numbers via `Intl` with the active locale and `numberingSystem: "latn"`.
7. Brand and identity strings (00 §0.5) are never translated.
8. Preferred terminology (native-speaker review in WP-25; then binding for all translations):

| English | Arabic | French | Spanish | German | Portuguese (BR) |
|---|---|---|---|---|---|
| Project | مشروع | Projet | Proyecto | Projekt | Projeto |
| Sequence | تسلسل | Séquence | Secuencia | Sequenz | Sequência |
| Timeline | الخط الزمني | Timeline | Línea de tiempo | Timeline | Linha do tempo |
| Track | مسار | Piste | Pista | Spur | Trilha |
| Clip | مقطع | Clip | Clip | Clip | Clipe |
| Media | الوسائط | Médias | Medios | Medien | Mídia |
| Folder (media bin) | مجلد | Dossier | Carpeta | Ordner | Pasta |
| Effect | تأثير | Effet | Efecto | Effekt | Efeito |
| Transition | انتقال | Transition | Transición | Übergang | Transição |
| Keyframe | إطار مفتاحي | Image clé | Fotograma clave | Keyframe | Quadro-chave |
| Marker | علامة | Marqueur | Marcador | Marker | Marcador |
| Captions | تسميات توضيحية | Sous-titres | Subtítulos | Untertitel | Legendas |
| Title | عنوان | Titre | Título | Titel | Título |
| Export | تصدير | Exporter | Exportar | Exportieren | Exportar |
| Proxy | ملف وكيل | Proxy | Proxy | Proxy | Proxy |
| Playhead | مؤشر التشغيل | Tête de lecture | Cabezal de reproducción | Abspielposition | Indicador de reprodução |
| In point / Out point | نقطة الدخول / نقطة الخروج | Point d'entrée / de sortie | Punto de entrada / de salida | In-Punkt / Out-Punkt | Ponto de entrada / de saída |
| Workspace | مساحة العمل | Espace de travail | Espacio de trabajo | Arbeitsbereich | Espaço de trabalho |
| Inspector | الخصائص | Inspecteur | Inspector | Inspektor | Inspetor |
| Snapping | الالتقاط | Magnétisme | Ajuste | Einrasten | Encaixe |
| Ripple delete | حذف وسد الفراغ | Supprimer et combler | Eliminar y cerrar hueco | Löschen und Lücke schließen | Excluir e fechar lacuna |
| Voice-over | تعليق صوتي | Voix off | Voz en off | Sprachaufnahme | Narração |
| Mixer | مازج الصوت | Mixeur audio | Mezclador | Mischpult | Mixer |
| Scopes | مراقبات الإشارة | Moniteurs de signal | Monitores de señal | Signalmonitore | Monitores de sinal |
| Background tasks (Jobs) | مهام الخلفية | Tâches en arrière-plan | Tareas en segundo plano | Hintergrundaufgaben | Tarefas em segundo plano |

---

## 7.13 Accessibility rules

1. Focus is always visible (`--focus-ring`) on keyboard focus and hidden on mouse focus (`:focus-visible`).
2. Landmarks: title bar (banner), toolbar (toolbar), each panel (region with its title), status bar (status, polite).
3. Live regions: one polite region for edits and job completions; one assertive region for errors; the current timecode is announced on demand with KB-159 or the viewer's "Announce timecode" button.
4. Canvas surfaces (timeline, scopes, waveforms) are focusable with `role="application"` only while focused, have an `aria-roledescription`, and expose state through the list view (REQ-EDT-015) and live announcements.
5. Dialogs: initial focus on the first field or the safest button; Esc cancels; focus returns to the invoking control.
6. No time limits except toasts (which also go to the notification center).
7. Every color-coded meaning has a text or icon equivalent (badges, labels, patterns in High Contrast).

---

## 7.14 Dialogs (DLG)

| ID | Dialog | Fields and behavior |
|---|---|---|
| DLG-01 | New Project | Name (required, 1–200 chars, Windows-safe), location (folder picker, default from settings), sequence preset (SQP list with icons for aspect), frame rate (FPS list), sample rate (48 kHz default); Create (Enter) / Cancel. Creates `<location>\<name>\<name>.alvs`. |
| DLG-02 | Project Settings | Name, default sequence, proxy mode, project folder (read-only, Show in Explorer). |
| DLG-03 | Sequence Settings | Name, preset, width/height (even), frame rate, sample rate, drop-frame timecode; warning when changing frame rate of a non-empty sequence (E1104 preview count). |
| DLG-04 | Export | Left: presets list (built-in by group, user presets, search). Right: file name + folder, summary line (codec, resolution, fps, bitrate estimate, size estimate), range (sequence / in–out / selection), "Use hardware encoding" (shows the resolved encoder), captions (none / burn in / sidecar / embed + track picker), chapters toggle, Advanced (expander: container, video and audio codec settings, metadata). Buttons: Add to Queue, Export Now (adds and starts), Cancel. Preflight results shown inline before enabling the buttons. |
| DLG-05 | Settings | Left navigation (pages from 03 §3.6) + search; right content generated from `settings_schema`; Reset page; changes apply immediately (no OK button), except language/theme preview which apply instantly anyway. |
| DLG-06 | Keyboard Shortcuts | Search by command or key (press-to-search); table (command, shortcut, context); select row → press keys to assign; conflict banner with "Reassign"/"Cancel"; reset one/all; import/export; print sheet. |
| DLG-07 | Relink Media | List of offline items (name, last path); "Search folder…"; matches with confidence (name/size/duration); Relink selected/all; fingerprint mismatch warning (E2009). |
| DLG-08 | Transcribe | Source (selected clips or sequence), model (with install state), language (auto + list), translate to English, use GPU; Start → job; result opens the Captions workspace. |
| DLG-09 | Model Manager | Table of MDL-01..05: size, installed, guidance; Download/Pause/Resume/Delete; storage folder and free space. |
| DLG-10 | Speed/Duration | Speed % (10–1600) or duration (timecode), reverse, preserve pitch, ripple following clips; Freeze Frame mode: duration. |
| DLG-11 | Marker | Name, color (CLR swatches), kind (standard/chapter), duration, note; Delete; navigation to previous/next marker. |
| DLG-12 | Paste Attributes | Checklist: Transform, Crop, Opacity & Blend, Effects (each listed), Masks, Audio volume/pan/fades, Audio effects. |
| DLG-13 | Collect Files | Target folder, include proxies (off), estimated size, Start → job. |
| DLG-14 | About | Logo, product name, version + commit + build date (copy button), license summary with "View license" (full AGPL text), "Third-party notices", "Source code for this version" link, trademark notice, previous version rollback (REQ-UPD-002). |
| DLG-15 | Crash Recovery | Explanation; two cards (autosave time, last saved time) with "Open autosave" / "Open last saved"; "Discard autosave" link. |
| DLG-16 | Restore Backup | Backups list (date, size); Restore as new file… |
| DLG-17 | Update Available | Version, release notes link, channel; Install now (saves first) / Later / Skip this version. |
| DLG-18 | Diagnostics Bundle | Checklist (logs, system info, settings, recent job errors, crash reports), redact paths (on), preview file list, Save as…. |
| DLG-19 | First-run Setup | Language, theme (preview), privacy statement (05 §5.8), hardware summary (GPU, decoder support), "Start tour" / "Skip". |
| DLG-20 | Unsaved Changes | "Save changes to <project>?" Save / Don't save / Cancel. |
| DLG-21 | Normalize Loudness | Target (−14/−16/−23/custom), ceiling dBTP, apply to selected clips or "on export"; Analyze & Apply → job. |
| DLG-22 | Auto-Duck | Music tracks, dialogue tracks, amount, threshold, attack, release; Apply → job → editable keyframes. |
| DLG-23 | Voice-over Record | Target track, input device, input meter, countdown toggle, punch-in between marks; Record (Ctrl+Alt+R) / Stop (Space). |
| DLG-24 | Basic Stabilize | Strength slider, crop mode; Analyze → job; result toggle "Use stabilized". |
| DLG-25 | Report a Problem | Summary text, generated issue body preview (version, OS, GPU, recent error codes; no paths), "Open in browser", "Create diagnostics bundle" link. |

Dialog sizing: content-driven, min 420 px wide, max 80% of the window; resizable when containing lists; size remembered.

---

## 7.15 Onboarding and help

- **First run (REQ-HLP-001):** DLG-19 → optional 5-step tour (coach marks): 1 Import media (Media panel), 2 Source and Program viewers, 3 Timeline editing basics, 4 Inspector and effects, 5 Export. Each step has Next/Back/Skip and is keyboard-accessible.
- **Help (REQ-HLP-002):** the User Guide is bundled Markdown in `assets/help/<locale>/` rendered by an in-app viewer with a table of contents and search; topics list in 10 WP-26.
- **Tooltips:** label + shortcut + one-line description for complex tools.
- **Command palette:** fuzzy search across all commands with category, shortcut, and disabled reason.

---

## 7.16 Responsive behavior

| Window width | Behavior |
|---|---|
| ≥ 1600 px | Dual viewer (Edit workspace) |
| 1280–1599 px | Single tabbed viewer; left dock tabs collapse labels to icons + tooltips below 1400 px |
| Height < 800 px | Timeline default height 40%; Inspector sections start collapsed except the first |

Panels have minimum sizes (viewer 320×180, timeline height 160, Inspector width 260); dockview prevents smaller splits.

---

## 7.17 Perceived performance rules

1. Show the Welcome screen skeleton within 300 ms of launch.
2. Import shows media tiles immediately with a placeholder and fills thumbnails progressively.
3. Timeline gestures render at 60 fps locally; the engine receives one commit at the end.
4. The viewer shows a Draft frame within 100 ms of a seek, then refines (02 §2.8).
5. Export and transcription dialogs close immediately after queueing; progress lives in Jobs and the status bar.
