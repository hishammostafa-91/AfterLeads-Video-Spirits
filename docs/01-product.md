# 01 — Product

Canonical for: vision, users, use cases, scope and releases, functional requirements (REQ), non-functional requirements (NFR).
Depends on: [00-index.md](00-index.md) (rules, IDs), [12-catalogs.md](12-catalogs.md) (enumerations referenced by requirements).

---

## 1.1 Vision

AfterLeads-Video-Spirits gives creators, educators, and small studios a **professional, fast, and fully free** video editor for Windows 11 that respects their privacy and their language. It works entirely offline, is open source under AGPL-3.0-or-later, and is designed from day one for English, Arabic (right-to-left), French, Spanish, German, and Brazilian Portuguese users.

Positioning statements:

1. **Professional core, calm surface.** Real editing tools (ripple/roll/slip/slide, keyframes, scopes, loudness, captions) presented in a clean interface that a first-time user can operate.
2. **Local and private.** No accounts, no telemetry, no cloud dependency. Even speech-to-text runs on the user's machine.
3. **Light and stable.** GPU-accelerated engine, isolated background workers, autosave, and crash recovery. The application stays responsive while exporting or transcribing.
4. **Built in the open.** Contributions (code, translations, title templates, LUT looks) are accepted into the main repository so improvements reach every user.

## 1.2 Goals and non-goals

**Goals for 1.0**

| Goal | Measure |
|---|---|
| Complete edit-to-export workflow for 1080p and 4K projects | GATE-01..GATE-04 pass (09 §9.11) |
| Smooth editing on mainstream hardware | NFR-PERF-01..09 met on the reference machines (09 §9.2) |
| First-class Arabic support | REQ-I18N-002, REQ-CAP-006, REQ-TTL-001 pass in `ar` |
| Never lose more than one autosave interval of work | NFR-REL-02 |
| Approachable for newcomers | NFR-UX-01 |

**Non-goals for 1.x**

- macOS, Linux, mobile, or web versions.
- Cloud features, accounts, collaboration, asset marketplaces.
- 3D compositing, node-based compositing, VFX particle systems.
- Third-party native plugin hosting before 1.2 (REQ-AUD-012).
- Generative AI that creates video or images.
- Reading or writing other editors' proprietary project formats.

## 1.3 Users and personas

| Persona | Profile | Primary needs |
|---|---|---|
| Creator (Nour) | Publishes weekly videos in Arabic and English for YouTube and short-form platforms; edits on a gaming laptop. | Fast cutting, vertical and horizontal versions, auto captions in Arabic, titles, music ducking, quick export presets. |
| Educator (Julien) | Teacher producing lessons in French; records voice-over over screen recordings. | Voice-over recording, captions, zoom/pan on stills, clear audio (denoise, loudness). |
| Small studio editor (Sofía) | Freelance editor delivering corporate videos in Spanish/English. | Precise trimming, color correction with scopes, 10-bit sources, ProRes/DNxHR delivery, render queue. |
| Contributor (Lukas) | Developer or translator who improves the project. | Clear docs, reproducible build on Windows 11, quick review of pull requests. |

## 1.4 Primary use cases

| ID | Use case | Main requirements |
|---|---|---|
| UC-01 | Create a project from a preset, import media, rough-cut, refine trims, add music, export to a platform preset. | REQ-PRJ-001, REQ-MED-001, REQ-EDT-002..004, REQ-AUD-001, REQ-EXP-001 |
| UC-02 | Make a vertical (9:16) version of a horizontal sequence by duplicating the sequence and reframing with transform keyframes. | REQ-PRJ-005, REQ-FX-001, REQ-FX-005 |
| UC-03 | Generate Arabic captions automatically, correct them, style them, burn them in. | REQ-STT-001, REQ-CAP-002, REQ-CAP-004, REQ-CAP-005, REQ-CAP-006 |
| UC-04 | Record a voice-over on a new audio track while watching the sequence, denoise it, normalize loudness. | REQ-AUD-006, REQ-AUD-003, REQ-AUD-004 |
| UC-05 | Color-correct interview footage using wheels and scopes, apply a LUT look, compare before/after. | REQ-COL-001, REQ-COL-002, REQ-FX-008, REQ-COL-004 |
| UC-06 | Edit 4K phone footage smoothly using proxies, then export the final at full quality with a hardware encoder. | REQ-MED-005, REQ-EXP-002, REQ-EXP-011 |
| UC-07 | Queue several exports (YouTube 4K, vertical 1080p, audio-only) and keep editing. | REQ-EXP-003, REQ-JOB-001 |
| UC-08 | Recover work after a crash or power loss. | REQ-PRJ-003, REQ-REL-001 |
| UC-09 | Work entirely with the keyboard and a screen reader. | REQ-A11Y-001, REQ-A11Y-002, REQ-EDT-013, REQ-EDT-015 |
| UC-10 | Add an animated title from a template in Arabic and English with fade and slide animation. | REQ-TTL-001..003 |

## 1.5 Scope by release

| Release | Contents |
|---|---|
| **1.0** | Every REQ and NFR in §1.6–§1.7 whose release column is `1.0`. Platform: Windows 11 x64. Locales: en, ar, fr, es, de, pt-BR. |
| **1.1** (roadmap) | Speed ramps (REQ-EDT-018), silence detection and jump cuts (REQ-EDT-019), motion tracking (REQ-FX-013), high-quality stabilization (REQ-MED-012), screen and webcam capture (REQ-MED-014), HSL qualifier and hue curves (REQ-COL-006), HDR timelines and export (REQ-COL-007), text-to-speech including Arabic voices (REQ-AUD-011), OpenTimelineIO interchange (REQ-INT-004), Windows on ARM64 (REQ-APP-013). |
| **1.2** (roadmap) | Audio plugin hosting (VST3, CLAP) in an isolated process (REQ-AUD-012), community shader effect packs (REQ-FX-014), multicam editing (REQ-EDT-020). |
| **Out of scope** | See §1.2 non-goals. |

Roadmap requirements (P2) are written so the 1.0 architecture keeps room for them (02 §2.12). They have no work package and no test in 1.0.

---

## 1.6 Functional requirements

Columns: **Pri** (P0/P1/P2, 00 §0.7), **Rel** (release), **WP** (work packages in 10 §10.7 that implement it), **Tests** (09 §9.10). Catalog references point to [12-catalogs.md](12-catalogs.md).

### 1.6.1 Application shell (APP)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-APP-001 | Main window with a custom Windows 11 style title bar (supports Snap Layouts on the maximize button), a menu bar (File, Edit, Clip, Timeline, Playback, View, Help — 12 §12.11), a toolbar, dockable panels, and a status bar. Minimum window size 1280×720. | P0 | 1.0 | WP-05 | T-APP-001 |
| REQ-APP-002 | Panels can be docked, tabbed, resized, floated inside the main window, maximized (` key), closed, and reopened from View › Panels. Layout persists per workspace across restarts. | P0 | 1.0 | WP-05 | T-APP-002 |
| REQ-APP-003 | Four built-in workspaces (WS-01 Edit, WS-02 Color, WS-03 Audio, WS-04 Captions) switchable from View › Workspaces and Alt+1..Alt+4; "Reset workspace"; users can save, rename, and delete custom workspaces. | P0 | 1.0 | WP-05, WP-24 | T-APP-002 |
| REQ-APP-004 | Every user action is a command with a stable ID, a localized label, an enabled state, and an optional shortcut (12 §12.10). Commands are reachable from menus, context menus, the command palette (Ctrl+Shift+P, fuzzy search, shows shortcuts), and keyboard shortcuts. | P0 | 1.0 | WP-05 | T-APP-003 |
| REQ-APP-005 | Welcome screen on startup when no project is restored: New Project (with sequence presets), Open Project, recent projects with thumbnails and a "missing" indicator, Getting Started link. | P0 | 1.0 | WP-05 | T-APP-004 |
| REQ-APP-006 | Non-modal toast notifications plus a notification center. Error notifications show the error code (12 §12.13) and a recovery action button when one exists. | P0 | 1.0 | WP-05 | T-APP-005 |
| REQ-APP-007 | Status bar shows: background job summary, autosave state, playback statistics (fps, dropped frames), proxy mode, renderer (GPU name or "WARP"). | P0 | 1.0 | WP-05, WP-07, WP-11 | T-APP-001 |
| REQ-APP-008 | Single instance: launching with a `.alvs` path while the app runs forwards the path to the running instance, which opens it (prompting to save the current project if dirty). One project is open at a time. | P0 | 1.0 | WP-05 | T-APP-006 |
| REQ-APP-009 | Themes: Dark (default), Light, High Contrast (activates automatically when Windows contrast themes are on); user-selectable accent color (12 §12.17); switching applies without restart. | P0 | 1.0 | WP-04 | T-UI-001 |
| REQ-APP-010 | Per-monitor DPI awareness and an application zoom of 80–200%. Text and icons are crisp at 100/125/150/175/200% scaling. | P0 | 1.0 | WP-04, WP-05 | T-UI-002 |
| REQ-APP-011 | Panel focus cycling with F6 / Shift+F6 and "maximize focused panel" with the ` key. | P1 | 1.0 | WP-05 | T-A11Y-001 |
| REQ-APP-012 | Closing the project or the app with unsaved changes asks Save / Don't Save / Cancel. The title bar shows a dirty marker (•). | P0 | 1.0 | WP-05 | T-APP-007 |
| REQ-APP-013 | Native Windows on ARM64 build. | P2 | 1.1 | — | — |

### 1.6.2 Projects and sequences (PRJ)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-PRJ-001 | New Project dialog (DLG-01): name, folder, sequence preset (12 §12.8, default "Auto — match first clip" with 1080p30 fallback), frame rate (12 §12.9), audio sample rate 48 kHz or 44.1 kHz. All sequences are SDR Rec.709 with square pixels and a stereo master in 1.0. | P0 | 1.0 | WP-02, WP-05 | T-PRJ-001 |
| REQ-PRJ-002 | Save, Save As, Open, Close. Project files are UTF-8 JSON `.alvs` (03 §3.4) written atomically. Media paths are stored absolute plus relative to the project folder; on open, the relative path is tried first when the absolute path is missing. | P0 | 1.0 | WP-03 | T-PRJ-002 |
| REQ-PRJ-003 | Autosave when dirty every N seconds (setting `alvs.general.autosaveIntervalSec`, default 60) to a sidecar file; on the next start after a crash, offer recovery (DLG-15). | P0 | 1.0 | WP-03 | T-PRJ-003, T-REL-001 |
| REQ-PRJ-004 | Every manual save also writes a timestamped backup; keep the newest 20 (setting `alvs.general.backupsKeep`). "Restore from backup…" lists backups with time and size (DLG-16) and restores into a new file. | P0 | 1.0 | WP-03 | T-PRJ-004 |
| REQ-PRJ-005 | A project contains one or more sequences: create, duplicate, rename, delete (cannot delete the last one), and edit sequence settings. Changing a sequence frame rate keeps clip times (snapped to the new frame grid) and reports any adjusted items. | P0 | 1.0 | WP-02, WP-12 | T-PRJ-005 |
| REQ-PRJ-006 | Nested sequences: a sequence can be placed as a clip in another sequence; "Nest selection" moves selected clips into a new sequence and replaces them with a nested clip. Cycles are rejected (E1006). | P1 | 1.0 | WP-12 | T-EDT-006 |
| REQ-PRJ-007 | Undo and redo for every edit (EditOp, 04 §4.4); at least 200 steps guaranteed and up to 1,000 within a 512 MB budget; History panel (PNL-13) lists steps and jumps to any step. | P0 | 1.0 | WP-02, WP-13 | T-PRJ-006 |
| REQ-PRJ-008 | Project schema versioning with forward-only migrations; the original is backed up before migration; files from a newer application version open read-only with a warning (E1004). | P0 | 1.0 | WP-02 | T-PRJ-007 |
| REQ-PRJ-009 | "Collect files…" copies the project, all used media, LUTs, and caption files into a `<name>.alvs-collected` folder and rewrites paths to relative ones. | P1 | 1.0 | WP-23 | T-INT-003 |
| REQ-PRJ-010 | Missing media appear as offline (red striped clip, "offline" frame in viewers). The Relink dialog (DLG-07) searches a folder by file name, size, and duration fingerprint and relinks all matches in one action. | P0 | 1.0 | WP-08 | T-MED-005 |
| REQ-PRJ-011 | A lock file prevents two application instances from writing the same project; a stale lock (dead process or older than 24 h on another host) can be taken over after confirmation. | P0 | 1.0 | WP-03 | T-PRJ-008 |

### 1.6.3 Media and proxies (MED)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-MED-001 | Import by dialog (Ctrl+I), drag and drop of files and folders (recursive), and paste of file paths. Supported inputs are whatever the FFmpeg build decodes; the guaranteed set is in 12 §12.20. Still images get the default duration `alvs.timeline.defaultStillDurationSec` (default 5 s); images larger than 8192 px on a side are downscaled for preview only. | P0 | 1.0 | WP-06, WP-08 | T-MED-001 |
| REQ-MED-002 | Every imported file is probed in a worker process (crash isolation). Metadata shown: duration, resolution, frame rate and VFR flag, codec, bit depth, chroma format, color primaries/transfer/matrix/range, HDR flag, rotation, audio channels and sample rate. | P0 | 1.0 | WP-06, WP-08 | T-MED-002 |
| REQ-MED-003 | Media panel (PNL-01): list and grid views, sort, search, bins (nested folders) with drag and drop, rename, remove from project (never deletes files), reveal in Explorer, usage indicator, and label colors (12 §12.17). | P0 | 1.0 | WP-08 | T-MED-003 |
| REQ-MED-004 | Thumbnails in the media panel with hover scrubbing; waveforms for audio-only items. Generation runs in background jobs and is cached (03 §3.7). | P0 | 1.0 | WP-08 | T-MED-004 |
| REQ-MED-005 | Proxies: create manually, or automatically for media taller than `alvs.media.proxyThresholdHeight` (default 1440) when `alvs.media.proxyAuto` is on; global "Use proxies" toggle; per-item proxy status; delete proxies. Export never uses proxies (REQ-EXP-011). | P0 | 1.0 | WP-08 | T-MED-006 |
| REQ-MED-006 | Hardware video decoding through D3D11VA for H.264, HEVC, VP9, and AV1 when the GPU supports it, with automatic software fallback per stream. | P0 | 1.0 | WP-06 | T-MED-007 |
| REQ-MED-007 | Variable-frame-rate footage plays and exports in sync: frames are selected by presentation timestamp, never by frame index. | P0 | 1.0 | WP-06, WP-11 | T-MED-008 |
| REQ-MED-008 | Rotation metadata is honored; HDR sources (PQ, HLG) are tone-mapped to SDR Rec.709 (02 §2.6.3); 10-bit sources keep 10-bit precision through the pipeline. | P0 | 1.0 | WP-06, WP-09 | T-MED-009, T-COL-002 |
| REQ-MED-009 | Numbered image sequences (for example `shot_0001.png`) can be imported as one clip with a chosen frame rate. | P1 | 1.0 | WP-08 | T-MED-010 |
| REQ-MED-010 | Interlaced sources are deinterlaced automatically (BWDIF) with a per-clip override (off / auto / force). | P1 | 1.0 | WP-06 | T-MED-011 |
| REQ-MED-011 | "Optimize media" transcodes selected items to an edit-friendly intra-frame file using the XP-09 settings (12 §12.7) as a background job and relinks to it on request. | P1 | 1.0 | WP-23 | T-MED-012 |
| REQ-MED-012 | High-quality stabilization and motion analysis based on feature tracking. | P2 | 1.1 | — | — |
| REQ-MED-013 | "Basic stabilize" analyzes a clip in a background job and renders a stabilized intermediate that the clip uses in place of its source (baked), with strength and crop options; reversible. | P1 | 1.0 | WP-23 | T-MED-013 |
| REQ-MED-014 | Screen recording (Windows Graphics Capture) and webcam capture. | P2 | 1.1 | — | — |

### 1.6.4 Playback and viewers (PLY)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-PLY-001 | Program viewer (PNL-06) shows the active sequence at the playhead; Source viewer (PNL-07) shows the selected media item with its own in/out marks. Dual-viewer layout on windows ≥ 1600 px wide, single viewer with a Source/Program toggle below that; the user can override. | P0 | 1.0 | WP-11 | T-PLY-001 |
| REQ-PLY-002 | Transport: play/pause, J/K/L shuttle (speeds −8…+8 per 12 §12.10), frame step, jump to start/end, previous/next edit point, loop between in/out, play in to out. | P0 | 1.0 | WP-11 | T-PLY-002 |
| REQ-PLY-003 | Audio and video stay in sync within ±1 frame (target ±10 ms) during 10 minutes of continuous playback; the audio clock is the master clock. | P0 | 1.0 | WP-11 | T-PLY-003 |
| REQ-PLY-004 | Preview resolution Auto / Full / Half / Quarter; dropped-frame indicator; "Play every frame" mode (no drops, may play slower than real time). | P0 | 1.0 | WP-11 | T-PLY-004 |
| REQ-PLY-005 | Time display as timecode `HH:MM:SS:FF` (drop-frame `;` separator for 29.97 and 59.94), frames, or seconds; the timecode field accepts typed times (absolute or `+/-` relative) to jump. | P0 | 1.0 | WP-11 | T-PLY-005 |
| REQ-PLY-006 | Viewer overlays: action-safe 93% and title-safe 90% guides, center cross, rule-of-thirds grid, transparency checkerboard; viewer zoom Fit and 25–400%, with pan. | P0 | 1.0 | WP-11 | T-PLY-006 |
| REQ-PLY-007 | Scrubbing by dragging the playhead or the viewer scrub bar shows frames within NFR-PERF-06 latency; optional audio scrub plays short snippets. | P0 | 1.0 | WP-11 | T-PLY-007, T-PERF-006 |
| REQ-PLY-008 | Full-screen playback (F11) and an external preview window on a chosen monitor showing a clean feed (no overlays), rendered by the native GPU path. | P1 | 1.0 | WP-11 | T-PLY-008 |
| REQ-PLY-009 | On-viewer handles for the selected clip's position, scale, rotation, anchor, and crop, with snapping to frame center and edges. | P0 | 1.0 | WP-14 | T-FX-003 |

### 1.6.5 Timeline editing (EDT)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-EDT-001 | Unlimited video and audio tracks (verified with 50 of each): add, delete, reorder, rename, resize height, lock, hide (video), mute/solo (audio), source-patching targets. | P0 | 1.0 | WP-12, WP-13 | T-EDT-001 |
| REQ-EDT-002 | Edit operations: insert, overwrite, append, replace, lift, extract, delete (leave gap), ripple delete, close gap, select gap, delete all gaps on a track. | P0 | 1.0 | WP-12, WP-13 | T-EDT-002 |
| REQ-EDT-003 | Split at playhead for selected clips or all unlocked tracks (Ctrl+K), and the Blade tool (B) that splits where clicked (Shift+click splits all tracks). | P0 | 1.0 | WP-12, WP-13 | T-EDT-002 |
| REQ-EDT-004 | Trimming: ripple, roll, slip, slide, extend edit to playhead (E), ripple-trim previous/next to playhead (Q/W), numeric trim by ±N frames, trimming limited by available handles. | P0 | 1.0 | WP-12, WP-13 | T-EDT-003 |
| REQ-EDT-005 | Snapping (toggle S) to playhead, clip edges, markers, in/out marks, and caption boundaries, with a visible snap line. | P0 | 1.0 | WP-13 | T-EDT-004 |
| REQ-EDT-006 | Selection: click, Ctrl+click, Shift+click range, marquee, select forward/backward from playhead (track or all tracks), select all, deselect; linked selection toggle (Ctrl+Shift+L). | P0 | 1.0 | WP-13 | T-EDT-005 |
| REQ-EDT-007 | Link/unlink audio and video of the same source; group/ungroup arbitrary clips (Ctrl+G / Ctrl+Shift+G); groups move and delete together. | P0 | 1.0 | WP-12, WP-13 | T-EDT-005 |
| REQ-EDT-008 | Copy, cut, paste at playhead onto targeted tracks, duplicate (Alt+drag or Ctrl+D), and paste attributes (effects, intrinsic properties, audio settings) with a checklist dialog. | P0 | 1.0 | WP-12, WP-13 | T-EDT-007 |
| REQ-EDT-009 | Clip speed 0.1×–16× (rational), reverse, and freeze frame of a chosen duration (DLG-10); audio pitch is preserved by default for speed changes (REQ-AUD-009) and audio is muted for reverse. | P0 | 1.0 | WP-09, WP-10, WP-12, WP-13 | T-EDT-008 |
| REQ-EDT-010 | Markers on sequences and clips: name, color (12 §12.17), note, optional duration (range marker), type standard/chapter; add (M), edit (Shift+M, DLG-11), delete, next/previous; Markers panel (PNL-12). | P0 | 1.0 | WP-12, WP-13 | T-EDT-009 |
| REQ-EDT-011 | Timeline zoom (Ctrl+= / Ctrl+-, Alt+wheel, zoom slider), fit to window (Shift+Z), horizontal and vertical scrolling, playhead follow modes (page, smooth, off). | P0 | 1.0 | WP-13 | T-EDT-010 |
| REQ-EDT-012 | Clip display: name, thumbnails (off / first frame / filmstrip), waveform, badges for effects, speed, reverse, proxy, and offline; label color. | P0 | 1.0 | WP-13 | T-EDT-010 |
| REQ-EDT-013 | Keyboard-only editing: every operation in REQ-EDT-002..010 is reachable by shortcut or command palette; arrow keys move selection between clips and tracks when the timeline has focus. | P0 | 1.0 | WP-13 | T-A11Y-002 |
| REQ-EDT-014 | Edit invariants (03 §3.5): clips never overlap on a track; video clip boundaries lie on the sequence frame grid; audio clip boundaries may be sub-frame only through the audio sub-frame nudge command. | P0 | 1.0 | WP-12 | T-EDT-011 |
| REQ-EDT-015 | Timeline list view: an accessible table of clips per track (name, start, end, duration, effects) with full keyboard navigation and screen-reader announcements; selecting a row selects the clip. | P1 | 1.0 | WP-13 | T-A11Y-002 |
| REQ-EDT-016 | Drag media from the Media panel or Source viewer onto the timeline; insert (,) and overwrite (.) from the Source viewer honor source in/out and track targets. | P0 | 1.0 | WP-13 | T-EDT-012 |
| REQ-EDT-017 | Drag only video or only audio of a source by using the video-only / audio-only drag handles in the Source viewer. | P0 | 1.0 | WP-13 | T-EDT-012 |
| REQ-EDT-018 | Speed ramps (time remapping with keyframes). | P2 | 1.1 | — | — |
| REQ-EDT-019 | Silence detection with automatic jump cuts. | P2 | 1.1 | — | — |
| REQ-EDT-020 | Multicam editing. | P2 | 1.2 | — | — |

### 1.6.6 Effects, keyframes, and masks (FX)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-FX-001 | Every visual clip has intrinsic properties: transform (position, scale uniform/non-uniform, rotation, anchor point, flip H/V), crop (left/top/right/bottom, feather), opacity, and blend mode (BLM-01..BLM-24). | P0 | 1.0 | WP-09, WP-14 | T-FX-001, T-RND-003 |
| REQ-FX-002 | Effects panel (PNL-02): categories, search, favorites; apply by drag onto a clip or adjustment layer, or with Enter/double-click to the selected clips. | P0 | 1.0 | WP-14 | T-FX-002 |
| REQ-FX-003 | The Inspector (PNL-05) shows the effect stack of the selected clip: reorder by drag, enable/bypass, reset, remove, copy/paste effects, collapse; save and apply effect presets (user library). | P0 | 1.0 | WP-14 | T-FX-002 |
| REQ-FX-004 | The video effects in 12 §12.1 marked release 1.0 are available with exactly the listed parameters, ranges, and defaults. | P0 | 1.0 | WP-15 | T-FX-004 |
| REQ-FX-005 | Every animatable parameter can be keyframed: stopwatch toggle, add/remove at playhead, previous/next keyframe, per-segment easing (EAS-01..EAS-34), keyframe markers in the Inspector and on the selected clip in the timeline. | P0 | 1.0 | WP-14 | T-FX-005 |
| REQ-FX-006 | Masks on any visual clip: rectangle, ellipse, and Bezier path (MSK-01..03) with feather, expansion, opacity, invert, and combine mode (add, subtract, intersect); mask shapes are keyframable. Masks can limit a single effect to the masked area. | P0 | 1.0 | WP-15 | T-FX-006 |
| REQ-FX-007 | Keying: chroma key with spill suppression and luma key (VFX-13, VFX-14) with a matte preview mode. | P0 | 1.0 | WP-15 | T-FX-007 |
| REQ-FX-008 | LUT effect (VFX-04) applies `.cube` 1D and 3D LUTs (sizes 2–65) with intensity; LUT files are copied into the project's `assets/luts/` folder and referenced by hash. | P0 | 1.0 | WP-15 | T-FX-008 |
| REQ-FX-009 | Adjustment layer clips (GEN-04) apply their effects to the composite of all tracks below them for their duration. | P0 | 1.0 | WP-09, WP-17 | T-FX-009 |
| REQ-FX-010 | Graph editor in the Inspector for one parameter at a time: value curve with Bezier handles, zoom, frame all. | P1 | 1.0 | WP-14 | T-FX-005 |
| REQ-FX-011 | All video effects run on the GPU through the same render path for preview and export; with no usable GPU the WARP software device renders identical results (within 09 §9.6 tolerance). | P0 | 1.0 | WP-09, WP-22 | T-RND-001, T-EXP-004 |
| REQ-FX-012 | Parameter UI is generated from effect manifests (04 §4.8): scrubbable number fields with sliders, angle dial, color picker with viewer eyedropper, on-viewer point picker, checkbox, dropdown, curve editor, LUT picker. | P0 | 1.0 | WP-14 | T-FX-003 |
| REQ-FX-013 | Motion tracking to attach effects, masks, or titles to moving objects. | P2 | 1.1 | — | — |
| REQ-FX-014 | Community shader effect packs (WGSL + manifest, no native code) installable from the Effects panel. | P2 | 1.2 | — | — |

### 1.6.7 Transitions (TRN)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-TRN-001 | Video transitions TRS-01..TRS-13 can be placed on an edit point (center, start, or end aligned). When handles are insufficient, the duration is reduced to the available handles and the user is notified (E1101). | P0 | 1.0 | WP-16 | T-TRN-001 |
| REQ-TRN-002 | Audio crossfades TRS-14..TRS-16 (constant power is the default). | P0 | 1.0 | WP-16 | T-TRN-002 |
| REQ-TRN-003 | "Add default transition" (Ctrl+T video, Ctrl+Shift+T audio) at the nearest edit point or to all selected edit points; default duration setting `alvs.timeline.defaultTransitionSec` (default 1.0 s). | P0 | 1.0 | WP-16 | T-TRN-003 |
| REQ-TRN-004 | Transition parameters (12 §12.3) are edited in the Inspector; duration is trimmed by dragging transition edges on the timeline. | P0 | 1.0 | WP-16 | T-TRN-003 |

### 1.6.8 Titles and generators (TTL)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-TTL-001 | Title clips contain one or more text layers: font (bundled FNT-01..FNT-08 and installed system fonts), size, weight, italic, fill color, stroke, shadow, background box (padding, corner radius, opacity), alignment, line height, letter spacing, position. Arabic text is shaped correctly and mixed-direction text follows the Unicode bidirectional algorithm. | P0 | 1.0 | WP-17 | T-TTL-001 |
| REQ-TTL-002 | In and out animation presets TAN-01..TAN-10 with adjustable duration and easing. | P0 | 1.0 | WP-17 | T-TTL-002 |
| REQ-TTL-003 | Titles panel (PNL-04) offers templates TPL-01..TPL-12; inserting one creates an editable title clip. | P0 | 1.0 | WP-17 | T-TTL-003 |
| REQ-TTL-004 | Generators GEN-01..GEN-05 (solid color, gradient, color bars, adjustment layer, tone) with default duration `alvs.timeline.defaultGeneratorDurationSec` (default 5 s). | P0 | 1.0 | WP-17 | T-TTL-004 |
| REQ-TTL-005 | Save a styled title as a user template (stored in the user library). | P1 | 1.0 | WP-17 | T-TTL-003 |
| REQ-TTL-006 | Edit title text directly on the Program viewer (double-click the text). | P1 | 1.0 | WP-17 | T-TTL-001 |

### 1.6.9 Audio (AUD)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-AUD-001 | Audio mixing at 32-bit float and the sequence rate: clip gain (−∞ to +24 dB), volume keyframes, pan, fade-in/out handles on clips, track volume/pan/mute/solo, master volume. | P0 | 1.0 | WP-10 | T-AUD-001 |
| REQ-AUD-002 | Mixer panel (PNL-09): one strip per audio track (fader, pan, mute, solo, peak meter with hold, effects slot) plus a master strip with a loudness meter. | P0 | 1.0 | WP-18 | T-AUD-002 |
| REQ-AUD-003 | Audio effects in 12 §12.2 marked release 1.0, on clips, tracks, and master. | P0 | 1.0 | WP-18 | T-AUD-003 |
| REQ-AUD-004 | Loudness measurement per ITU-R BS.1770 / EBU R128 (integrated, short-term, momentary, true peak) and normalization of clips or the export to −14, −16, −23 LUFS or a custom target with a true-peak ceiling. | P0 | 1.0 | WP-18 | T-AUD-004 |
| REQ-AUD-005 | Auto-ducking: lower selected music tracks under dialogue tracks (amount, threshold, attack, release); the result is editable volume keyframes. | P1 | 1.0 | WP-18 | T-AUD-005 |
| REQ-AUD-006 | Voice-over recording to a target track at the playhead: input device selection, input meter, 3-second countdown, punch-in between in/out marks; the recording is saved as WAV in the project folder and inserted as a clip. | P0 | 1.0 | WP-18 | T-AUD-006 |
| REQ-AUD-007 | Waveforms on timeline audio clips with a draggable clip-gain line. | P0 | 1.0 | WP-13 | T-EDT-010 |
| REQ-AUD-008 | Mono and stereo sources; per-clip channel mapping (stereo, left to both, right to both, mono mix, swap); 5.1 sources are downmixed to stereo using the ITU-R BS.775 coefficients. | P0 | 1.0 | WP-10 | T-AUD-007 |
| REQ-AUD-009 | Pitch-preserving speed change between 0.25× and 4× for speech and music clips. | P1 | 1.0 | WP-10 | T-AUD-008 |
| REQ-AUD-010 | Output device selection (WASAPI shared mode), automatic sample-rate conversion, buffer presets (low / normal / safe), and recovery when the device is removed (fall back to the default device, E4001). | P0 | 1.0 | WP-10 | T-AUD-009 |
| REQ-AUD-011 | Local text-to-speech voices including Arabic. | P2 | 1.1 | — | — |
| REQ-AUD-012 | Audio plugin hosting (VST3, CLAP) in an isolated process. | P2 | 1.2 | — | — |

### 1.6.10 Color and scopes (COL)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-COL-001 | Color workspace (WS-02) with large controls for VFX-01 (basic correction), VFX-02 (color wheels), VFX-03 (curves), VFX-04 (LUT), VFX-05 (levels), VFX-06 (black & white). | P0 | 1.0 | WP-15, WP-19 | T-COL-001 |
| REQ-COL-002 | Scopes panel (PNL-10): luma waveform, RGB parade, vectorscope with 75% targets and skin-tone line, histogram (RGB and luma); computed on the GPU from the program output at ≥ 10 updates per second during playback. | P0 | 1.0 | WP-19 | T-COL-003 |
| REQ-COL-003 | Color pipeline per 02 §2.6.3: Rec.709 display-referred working space, correct YUV matrix/range/primaries conversion, HDR tone mapping, RGBA16F internal precision. | P0 | 1.0 | WP-09 | T-COL-002, T-RND-002 |
| REQ-COL-004 | Before/after split view in the Program viewer for the selected clip (bypasses that clip's effects on one side). | P1 | 1.0 | WP-19 | T-COL-004 |
| REQ-COL-005 | Copy grade / paste grade: copies the Color-category effects of one clip to selected clips. | P1 | 1.0 | WP-19 | T-COL-004 |
| REQ-COL-006 | HSL qualifier (secondary correction) and hue curves. | P2 | 1.1 | — | — |
| REQ-COL-007 | HDR (PQ/HLG) timelines and HDR export with metadata. | P2 | 1.1 | — | — |

### 1.6.11 Captions (CAP)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-CAP-001 | Sequences have zero or more caption tracks, each with a name and a BCP 47 language tag; captions have start, end, and text, and appear in a caption lane on the timeline. | P0 | 1.0 | WP-20 | T-CAP-001 |
| REQ-CAP-002 | Captions panel (PNL-11): editable list (start, end, text), add at playhead, split at cursor, merge, delete, find and replace, click to seek; spell checking in text fields. | P0 | 1.0 | WP-20 | T-CAP-002 |
| REQ-CAP-003 | Import SRT, WebVTT, and ASS/SSA (text, timing, and basic style); export SRT, WebVTT, and ASS. Input encodings: UTF-8 (with or without BOM), UTF-16 LE/BE; Windows-1252 and Windows-1256 by detection with a confirmation prompt. | P0 | 1.0 | WP-20 | T-CAP-003 |
| REQ-CAP-004 | Caption style per track (font, size, color, outline, shadow, background box, vertical position, alignment, maximum lines) and per-caption position override. | P0 | 1.0 | WP-20 | T-CAP-004 |
| REQ-CAP-005 | At export, captions can be burned in, written as a sidecar file, or embedded as a subtitle stream (MP4 `mov_text`, MKV/WebM WebVTT/SRT). | P0 | 1.0 | WP-20, WP-22 | T-CAP-005 |
| REQ-CAP-006 | Right-to-left and mixed-direction captions render correctly (shaping, bidi, punctuation placement) in the editor, viewer, burn-in, and sidecar files. | P0 | 1.0 | WP-20 | T-CAP-006 |
| REQ-CAP-007 | Readability rules with warnings: default 42 characters per line, at most 2 lines, minimum duration 0.7 s, maximum 7 s, minimum gap 2 frames (settings in 03 §3.6). | P0 | 1.0 | WP-20 | T-CAP-007 |

### 1.6.12 Speech-to-text (STT)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-STT-001 | Transcribe selected clips or a whole sequence (mixdown) into captions with local Whisper models; language auto-detect or chosen (at least en, ar, fr, es, de, pt); optional translate-to-English. | P0 | 1.0 | WP-21 | T-STT-001 |
| REQ-STT-002 | Model manager (DLG-09): models MDL-01..MDL-05 with size and speed/quality guidance; download with progress, pause/resume, SHA-256 verification; delete; shows disk usage and storage folder. | P0 | 1.0 | WP-21 | T-STT-002 |
| REQ-STT-003 | Transcription runs as a background job (cancellable, with progress); CPU by default; GPU acceleration through Vulkan when available and enabled. | P0 | 1.0 | WP-21 | T-STT-003 |
| REQ-STT-004 | Results become a **new** caption track (never overwrite) segmented with REQ-CAP-007 rules using word timestamps. | P0 | 1.0 | WP-21 | T-STT-004 |
| REQ-STT-005 | Transcript panel (PNL-15): searchable transcript of a media item; clicking a word seeks the Source viewer to it. | P1 | 1.0 | WP-21 | T-STT-005 |
| REQ-STT-006 | No network access during transcription; models are downloaded only on explicit user action. | P0 | 1.0 | WP-21 | T-SEC-004 |

### 1.6.13 Export (EXP)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-EXP-001 | Export dialog (DLG-04): preset (XP-01..XP-20) or custom settings — container, video codec, resolution (sequence or scaled), frame rate (sequence), quality mode (CRF/CQ, VBR, CBR), bitrate, keyframe interval, 8/10-bit, audio codec, bitrate, sample rate, channels; file name and folder; range (whole sequence, in to out, selected clips); captions option; audio-only and video-only. | P0 | 1.0 | WP-22 | T-EXP-001 |
| REQ-EXP-002 | Hardware encoders (NVIDIA NVENC, AMD AMF, Intel QSV) for H.264, HEVC, and AV1 are detected by a test encode at startup (cached per driver version); "Use hardware encoding" falls back to software when unavailable. | P0 | 1.0 | WP-22 | T-EXP-002 |
| REQ-EXP-003 | Render queue: add exports, reorder, run sequentially in worker processes, pause, cancel, retry, progress and ETA, completion notification, open output folder. Editing continues during export. | P0 | 1.0 | WP-22 | T-EXP-003 |
| REQ-EXP-004 | Export frame (Ctrl+Shift+E) at the playhead as PNG or JPEG at sequence resolution. | P0 | 1.0 | WP-22 | T-EXP-005 |
| REQ-EXP-005 | Audio-only export presets XP-16..XP-20. | P0 | 1.0 | WP-22 | T-EXP-001 |
| REQ-EXP-006 | Alpha-channel export via ProRes 4444 (XP-10) and PNG sequence (XP-14). | P0 | 1.0 | WP-22 | T-EXP-006 |
| REQ-EXP-007 | Preflight before queueing: estimated size versus free disk space, writable path, overwrite confirmation, offline media warning, codec constraints (even dimensions for 4:2:0, level limits). | P0 | 1.0 | WP-22 | T-EXP-007 |
| REQ-EXP-008 | Output verification after each export: probe the file and confirm duration, frame count, stream layout, and color tags (BT.709, limited range); MP4/MOV use fast start. | P0 | 1.0 | WP-22 | T-EXP-004 |
| REQ-EXP-009 | Chapter markers can be embedded (MP4, MKV) at export. | P0 | 1.0 | WP-22 | T-EXP-008 |
| REQ-EXP-010 | Save, rename, delete, import, and export user presets (JSON). | P1 | 1.0 | WP-22 | T-EXP-009 |
| REQ-EXP-011 | Exports always use original media (never proxies) at full quality regardless of preview resolution. | P0 | 1.0 | WP-22 | T-EXP-004 |
| REQ-EXP-012 | Animated GIF (XP-13, two-pass palette) and JPEG/PNG image sequence export. | P1 | 1.0 | WP-22 | T-EXP-006 |

### 1.6.14 Background jobs (JOB)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-JOB-001 | Probing, thumbnails, waveforms, proxies, optimization, stabilization, loudness analysis, transcription, and exports run as jobs in worker processes; the UI never blocks on them. | P0 | 1.0 | WP-07 | T-JOB-001 |
| REQ-JOB-002 | Jobs panel (PNL-14): status, progress, ETA, cancel, retry, pause/resume queue, open output, view log; per-type concurrency (04 §4.6.4). | P0 | 1.0 | WP-07 | T-JOB-002 |
| REQ-JOB-003 | A crashing or hung worker fails only its job (retried up to 2 times with backoff when retryable); unfinished jobs are re-queued after an application restart. | P0 | 1.0 | WP-07 | T-JOB-003 |
| REQ-JOB-004 | Job outputs are written to `*.part` files and renamed atomically on success; cancelled or failed jobs remove partial files. | P0 | 1.0 | WP-07 | T-JOB-004 |

### 1.6.15 Settings (SET)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-SET-001 | Settings dialog (DLG-05) with pages General, Appearance, Playback, Timeline, Media & Cache, Export, Keyboard, AI, Advanced, covering every key in 03 §3.6; search across settings; reset page or all. | P0 | 1.0 | WP-24 | T-SET-001 |
| REQ-SET-002 | Keyboard shortcut editor (DLG-06): search by command or key, assign, remove, conflict detection within the same context, reset one or all, import/export keymap JSON, printable shortcut sheet. | P0 | 1.0 | WP-24 | T-SET-002 |
| REQ-SET-003 | Settings persist atomically, validate against the schema (invalid values revert to defaults with a notice), preserve unknown keys, and recover from a corrupt file by restoring the last good backup. | P0 | 1.0 | WP-03 | T-SET-003 |
| REQ-SET-004 | Cache management: location, size limit, usage by category, and clear thumbnails/waveforms/proxies/analysis separately. | P0 | 1.0 | WP-24 | T-SET-004 |

### 1.6.16 Localization (I18N)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-I18N-001 | The UI is available in en, ar, fr, es, de, and pt-BR (LOC-01..LOC-06); the language can be changed at runtime without restart; the default follows the Windows display language when supported, else English. | P0 | 1.0 | WP-04, WP-25 | T-I18N-001 |
| REQ-I18N-002 | Arabic uses a mirrored right-to-left layout following 07 §7.11 (timeline, rulers, transport, and media time do not mirror). | P0 | 1.0 | WP-04, WP-25 | T-I18N-002 |
| REQ-I18N-003 | Messages use ICU MessageFormat (plurals, select); numbers and dates use locale formatting; timecodes and frame numbers always use Western digits and left-to-right isolation. | P0 | 1.0 | WP-04 | T-I18N-003 |
| REQ-I18N-004 | At release every key exists in all six locales; CI fails on missing or extra keys; pseudo-locales LOC-90 (expanded) and LOC-91 (RTL) are used in tests. | P0 | 1.0 | WP-04, WP-25 | T-I18N-004 |
| REQ-I18N-005 | Shortcuts bind to physical keys (`KeyboardEvent.code`) so they work on Arabic, French (AZERTY), and German (QWERTZ) layouts; key labels display the current layout's characters when available. | P0 | 1.0 | WP-05 | T-I18N-005 |
| REQ-I18N-006 | In-app help topics are available in all six locales (English fallback per topic until translated). | P1 | 1.0 | WP-26 | T-HLP-002 |
| REQ-I18N-007 | Bundled fonts cover Latin and Arabic for UI and titles; other scripts use installed Windows fonts as fallback. | P0 | 1.0 | WP-04, WP-17 | T-I18N-002 |

### 1.6.17 Accessibility (A11Y)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-A11Y-001 | Every function is keyboard-operable with visible focus, logical tab order, and no keyboard traps; F6 cycles panels. | P0 | 1.0 | WP-05, WP-25 | T-A11Y-001 |
| REQ-A11Y-002 | Screen readers (Narrator, NVDA) get names, roles, values, and states for all controls; edits, errors, and on-demand timecode are announced through live regions. | P0 | 1.0 | WP-04, WP-05, WP-25 | T-A11Y-003 |
| REQ-A11Y-003 | Contrast meets WCAG 2.2 AA (4.5:1 text, 3:1 UI components and focus indicators) in Dark and Light; the High Contrast theme follows Windows forced colors. | P0 | 1.0 | WP-04 | T-A11Y-004 |
| REQ-A11Y-004 | The Windows "Animation effects" off setting (`prefers-reduced-motion`) disables non-essential motion; no information is conveyed by color alone. | P0 | 1.0 | WP-04 | T-A11Y-004 |

### 1.6.18 Help and onboarding (HLP)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-HLP-001 | First-run setup (DLG-19): language, theme, privacy statement (no telemetry), optional hardware summary; then an optional 5-step tour that can be replayed from Help. | P0 | 1.0 | WP-26 | T-HLP-001 |
| REQ-HLP-002 | Help menu: Getting Started, User Guide (in-app offline viewer), Keyboard Shortcuts sheet with search, Report a Problem (opens a prefilled GitHub issue in the browser after the user reviews the text), Check for Updates, About. | P0 | 1.0 | WP-26 | T-HLP-002 |
| REQ-HLP-003 | Tooltips with the shortcut on every icon button; empty states explain the next step (07 §7.10). | P0 | 1.0 | WP-04, WP-05 | T-HLP-003 |
| REQ-HLP-004 | About dialog (DLG-14): version, build commit and date, license (full AGPL text), third-party notices, source code link for the exact version, trademark notice. | P0 | 1.0 | WP-26 | T-LIC-001 |

### 1.6.19 Updates (UPD)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-UPD-001 | Check for updates at most once per 24 h (setting `alvs.general.checkUpdates`, default on) against a signed manifest on GitHub Releases (stable or beta channel); notify non-modally; download and install on user action; the Ed25519 signature is verified before installing. | P0 | 1.0 | WP-28 | T-UPD-001 |
| REQ-UPD-002 | Installation never starts during an export or with unsaved changes (the user is asked to save first); the previous installer is kept so the user can roll back from Help › About. | P0 | 1.0 | WP-28 | T-UPD-002 |
| REQ-UPD-003 | A portable ZIP build stores settings next to the executable and never self-updates; update checks can be turned off completely. | P1 | 1.0 | WP-28 | T-UPD-003 |

### 1.6.20 Reliability and diagnostics (REL)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-REL-001 | After an abnormal exit, the crash recovery dialog (DLG-15) offers to open the autosave, open the last saved version, or discard; it shows both timestamps. | P0 | 1.0 | WP-03, WP-27 | T-REL-001 |
| REQ-REL-002 | Safe mode (hold Shift while starting, or `--safe-mode`): WARP renderer, no hardware decoding, default layout, GPU transcription off, with a persistent banner. | P0 | 1.0 | WP-27 | T-REL-002 |
| REQ-REL-003 | Rolling logs (10 files × 10 MB) in the logs folder (03 §3.8) with structured fields; user file paths are redacted when exporting diagnostics. | P0 | 1.0 | WP-03 | T-REL-003 |
| REQ-REL-004 | "Create diagnostics bundle" (DLG-18) writes a ZIP with system info, GPU/driver, audio devices, redacted settings, logs, recent job errors, and crash reports; the user previews the file list before saving. | P0 | 1.0 | WP-27 | T-REL-004 |
| REQ-REL-005 | Crashes are recorded locally only: Rust panic reports and native minidumps written by an out-of-process crash handler; on the next start the user may open "Report a Problem". Nothing is uploaded automatically. | P0 | 1.0 | WP-27 | T-REL-005 |
| REQ-REL-006 | GPU device loss (driver reset) recreates the device and resumes without data loss; after 3 losses within 10 minutes the session switches to WARP and tells the user. | P0 | 1.0 | WP-09 | T-REL-006 |
| REQ-REL-007 | Disk-full and permission errors on save, export, or cache writes keep previous files intact and show an actionable error (E8001, E5003). | P0 | 1.0 | WP-03, WP-22 | T-REL-007 |

### 1.6.21 Interoperability (INT)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-INT-001 | Export a sequence as a CMX 3600 EDL (one video track and up to four audio channels; reel names from file names). | P1 | 1.0 | WP-23 | T-INT-001 |
| REQ-INT-002 | Export markers as a chapter text list (`00:00 Title`, first chapter at 00:00) and as CSV; import markers from CSV. | P0 | 1.0 | WP-23 | T-INT-002 |
| REQ-INT-003 | `.cube` LUT import validates size, domain, and data with line/column error messages (E2010). | P0 | 1.0 | WP-15 | T-FX-008 |
| REQ-INT-004 | OpenTimelineIO (`.otio`) import and export. | P2 | 1.1 | — | — |

### 1.6.22 Licensing in the product (LIC)

| ID | Requirement | Pri | Rel | WP | Tests |
|---|---|---|---|---|---|
| REQ-LIC-001 | The installer and the About dialog include the AGPL-3.0 license text and generated third-party notices (Rust crates, npm packages, FFmpeg and codec libraries, fonts, icons, models). | P0 | 1.0 | WP-26, WP-28 | T-LIC-001 |
| REQ-LIC-002 | Every release publishes the tagged source code and an archive of the exact third-party library sources used to build FFmpeg and its codecs. | P0 | 1.0 | WP-28 | T-LIC-002 |
| REQ-LIC-003 | README and About show the trademark notice from TRADEMARKS.md. | P0 | 1.0 | WP-26 | T-LIC-001 |

---

## 1.7 Non-functional requirements

Reference machines R-LOW, R-MID, R-HIGH are defined in 09 §9.2.

| ID | Requirement | Target | Rel | Tests |
|---|---|---|---|---|
| NFR-PERF-01 | Startup to Welcome screen | Cold ≤ 3.0 s, warm ≤ 1.5 s on R-MID | 1.0 | T-PERF-001 |
| NFR-PERF-02 | Open a 30-minute 1080p project with 500 clips | ≤ 3 s to first frame on R-MID | 1.0 | T-PERF-002 |
| NFR-PERF-03 | 1080p30 H.264 single layer, Full preview | 0 dropped frames in 60 s on R-LOW (hardware decode) | 1.0 | T-PERF-003 |
| NFR-PERF-04 | 4K30 HEVC 10-bit, 2 layers + 3 effects + 1 title, Half preview | ≥ 29.97 fps sustained on R-MID | 1.0 | T-PERF-004 |
| NFR-PERF-05 | Timeline interaction with 1,000 clips | ≥ 60 fps rendering, p95 input-to-paint ≤ 50 ms on R-MID | 1.0 | T-PERF-005 |
| NFR-PERF-06 | Scrub frame latency (1080p H.264 long-GOP) | Cached ≤ 50 ms, uncached ≤ 250 ms on R-MID | 1.0 | T-PERF-006 |
| NFR-PERF-07 | Export 1080p30 H.264 (XP-05 software) / NVENC | ≥ 1.5× / ≥ 4× real time on R-MID | 1.0 | T-PERF-007 |
| NFR-PERF-08 | Import a 1-hour clip | First thumbnail ≤ 1 s; filmstrip and waveform complete ≤ 60 s in background | 1.0 | T-PERF-008 |
| NFR-PERF-09 | Transcription with MDL-03 | CPU ≥ 1× real time on R-MID; Vulkan GPU ≥ 5× | 1.0 | T-PERF-009 |
| NFR-RES-01 | Memory (all processes incl. WebView2) | ≤ 2.0 GB editing a 30-min 1080p project; ≤ 3.5 GB 4K without proxies | 1.0 | T-PERF-010 |
| NFR-RES-02 | Idle usage (project open, not playing) | CPU ≤ 1%, GPU ≤ 1% averaged over 60 s | 1.0 | T-PERF-011 |
| NFR-RES-03 | Size | Installer ≤ 150 MB; installed ≤ 400 MB (models excluded) | 1.0 | T-PERF-012 |
| NFR-REL-01 | Stability | ≥ 99.5% crash-free sessions in the beta cohort (local session counters reported voluntarily) | 1.0 | T-REL-008 |
| NFR-REL-02 | Data-loss bound | ≤ one autosave interval of edits lost on crash or power loss; zero corrupted project files | 1.0 | T-REL-001, T-PRJ-003 |
| NFR-REL-03 | Soak | 8-hour scripted edit/playback loop: no crash, memory growth ≤ 10% | 1.0 | T-REL-009 |
| NFR-SEC-01 | Vulnerabilities | 0 known high/critical advisories in shipped dependencies at release | 1.0 | T-SEC-001 |
| NFR-PRIV-01 | Network use | Only update checks (when enabled), user-initiated model downloads, and links the user opens in the browser | 1.0 | T-SEC-004 |
| NFR-UX-01 | Learnability | ≥ 4 of 5 first-time users complete UC-01, UC-03, UC-10 without help within 20 minutes total | 1.0 | T-UX-001 |
| NFR-MNT-01 | Test coverage | Line coverage ≥ 80% for `alvs-core`, `alvs-model`, `alvs-formats`; ≥ 60% other crates; ≥ 70% UI state/logic modules | 1.0 | T-QA-001 |
| NFR-MNT-02 | CI duration | Pull-request CI ≤ 25 minutes with warm caches | 1.0 | T-QA-002 |
| NFR-INS-01 | Installation | Per-user install without administrator rights in ≤ 60 s; uninstall never touches user projects or media | 1.0 | T-UPD-004 |
| NFR-COMP-01 | Platform | Windows 11 22H2+ x64, any GPU with a Direct3D 12 driver (WARP fallback), 8 GB RAM minimum, CPU with AVX2 | 1.0 | T-QA-003 |

---

## 1.8 Assumptions and constraints

1. The owner works on Windows 11 x64 with administrator rights for installing development tools.
2. The owner has a free GitHub account; the repository is public; GitHub Actions minutes for public repositories are free.
3. The owner chooses and pays (or not) for an LLM provider for OpenCode; the project itself requires no paid service (OD-05).
4. Reference machines R-LOW and R-MID are available to the owner at least for release testing (09 §9.2); CI runs on GitHub-hosted Windows runners without a GPU (WARP).
5. FFmpeg with GPL components is built from source through vcpkg; the resulting binaries are AGPL-compatible (02 ADR-004, 05 §5.6).

## 1.9 Dependencies on other documents

- Every REQ above is implemented by the work packages named in its row (10 §10.7) and verified by the named tests (09 §9.10).
- UI behavior for panels and dialogs referenced here is specified in 07.
- Data structures referenced here are specified in 03; interfaces in 04.

## 1.10 Acceptance summary

Release 1.0 is accepted when every P0 requirement with release `1.0` passes its tests, every P1 requirement passes or has an owner-signed waiver, every NFR target is met, and the release gates GATE-01..GATE-12 in 09 §9.11 pass.
