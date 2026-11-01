# 04 — Interfaces

Canonical for: every contract between components — UI↔core IPC commands and events, the `EditOp` contract, preview frame transport, worker protocol and job specifications, CLI, effect manifests, the error envelope, file interchange rules, and core Rust traits.
Depends on: [02-architecture.md](02-architecture.md), [03-data.md](03-data.md), [12-catalogs.md](12-catalogs.md).

---

## 4.1 Interface map

| Boundary | Mechanism | Defined in |
|---|---|---|
| UI → core (requests) | Tauri commands (`invoke`) with ts-rs generated types | 4.3 |
| Core → UI (notifications) | Tauri events; Tauri channels for binary streams | 4.10 |
| Model edits | `EditOp` → `EditResult` with `EntityPatch` | 4.4 |
| Preview frames | WebView2 SharedBuffer ring + acknowledgements | 4.5 |
| Core ↔ worker | Process spawn + JSON Lines over stdio | 4.6 |
| Automation/tests | CLI mode | 4.7 |
| Effects/transitions/generators | Effect manifest + WGSL/DSP implementation | 4.8 |
| Errors | Error envelope | 4.9 |
| Files from/to other tools | SRT, WebVTT, ASS, EDL, CSV, chapter text, `.cube` | 4.11 |
| Engine internals | Rust traits | 4.12 |

---

## 4.2 IPC conventions

1. Command names are `snake_case` Rust functions (`project_open`); the UI calls them only through typed wrappers in `app/src/ipc/commands.ts` (`project.open(...)`).
2. DTOs live in `alvs-model::dto` (and other crates' `dto` modules), derive `Serialize, Deserialize, TS`, and export to `app/src/ipc/bindings/` via `cargo test -p <crate> export_bindings`. CI fails if regenerated bindings differ from the committed ones (ADR-010).
3. Every command validates its input (lengths, ranges, IDs exist, paths allowed by 05 SEC-07) and returns `Result<T, ErrorEnvelope>`; the TS wrapper rejects with the envelope.
4. Commands never block the main thread: file, decode, and model work runs on the async runtime or the model actor.
5. Binary payloads (waveform peaks, scope data) use `tauri::ipc::Response` raw bytes or `tauri::ipc::Channel`; never base64 JSON.
6. Capabilities (`app/src-tauri/capabilities/*.json`) grant the main window only the application's own commands and the minimal plugin permissions listed in 05 §5.4.

---

## 4.3 Commands

Types in `PascalCase` are generated DTOs. "Errors" lists the most relevant codes (12 §12.13).

### 4.3.1 Application

| Command | Request | Response | Errors / notes |
|---|---|---|---|
| `app_info` | — | `AppInfo { version, commit, buildDate, identifier, renderer { name, isWarp }, safeMode, portable, locale }` | — |
| `app_ready` | — | — | UI signals it mounted; core emits current state events |
| `app_quit` | `{ force: bool }` | — | Runs the shutdown sequence (02 §2.5) |

### 4.3.2 Project

| Command | Request | Response | Errors / notes |
|---|---|---|---|
| `project_new` | `NewProjectRequest { name, folder, sequencePreset, frameRate, sampleRate }` | `ProjectSnapshot` | E1008, E8004 |
| `project_open` | `{ path }` | `ProjectSnapshot` | E1001–E1005, E1007 |
| `project_open_readonly` | `{ path }` | `ProjectSnapshot` | Used after E1007 |
| `project_take_over_lock` | `{ path }` | `ProjectSnapshot` | Confirmed take-over (03 §3.9.4) |
| `project_save` / `project_save_as` | — / `{ path }` | `SaveResult { path, backupPath }` | E1008, E8001 |
| `project_close` | `{ discardChanges: bool }` | — | |
| `project_snapshot` | — | `ProjectSnapshot` (full normalized state) | Used after a WebView reload |
| `project_recent` | — | `RecentProject[]` | |
| `project_recovery_info` / `project_recover` | — / `{ choice: "autosave" \| "saved" \| "discard" }` | `RecoveryInfo` / `ProjectSnapshot` | DLG-15 |
| `project_backups` / `project_restore_backup` | — / `{ backupPath, targetPath }` | `BackupInfo[]` / `ProjectSnapshot` | E1010 |
| `project_set_proxy_mode` | `{ enabled }` | — | Not undoable (view state) |
| `project_collect` | `{ targetFolder }` | `JobId` | REQ-PRJ-009 |

### 4.3.3 Media

| Command | Request | Response | Errors / notes |
|---|---|---|---|
| `media_import` | `{ paths: string[], binId?: string }` | `ImportTicket { jobIds }` | Probes run as jobs; items are added by `EditOp::AddMedia` as probes succeed (one undo step per import action) |
| `media_relink` | `{ mediaId, path }` | `EditResult` | E2009 warning when fingerprints differ |
| `media_relink_search` | `{ folder, mediaIds? }` | `RelinkReport { matches[], unresolved[] }` | DLG-07 |
| `media_probe_details` | `{ mediaId }` | `ProbeDetails` | |
| `media_thumbnail` | `{ mediaId, kind: "poster" \| "filmstrip", height }` | `ThumbnailRef { state, assetUrl?, index? }` | Asset URL uses the scoped asset protocol (05 SEC-08) |
| `media_waveform` | `{ mediaId, stream, samplesPerPeak, startFlicks, endFlicks }` | raw bytes (`i16` min/max pairs) | |
| `media_create_proxies` / `media_delete_proxies` | `{ mediaIds }` | `JobId[]` / — | |
| `media_optimize` | `{ mediaIds }` | `JobId[]` | REQ-MED-011 |
| `media_stabilize` | `{ mediaId, strength: 0..1, crop: "auto" \| "none" }` | `JobId` | REQ-MED-013 |
| `media_set_derived_active` | `{ mediaId, which: "optimized" \| "stabilized", active }` | `EditResult` | |
| `media_reveal` | `{ mediaId }` | — | Opens Explorer with the file selected |

### 4.3.4 Edit and history

| Command | Request | Response | Errors / notes |
|---|---|---|---|
| `edit_apply` | `EditOp` (4.4) | `EditResult { revision, patch: EntityPatch, label, warnings: ErrorEnvelope[] }` | E1101–E1104, E1006, E0002 |
| `edit_undo` / `edit_redo` | — | `EditResult` | No-op returns current revision |
| `edit_history` | — | `HistoryInfo { entries: [{ index, labelKey, labelParams, timeUtc }], current }` | |
| `edit_jump` | `{ index }` | `EditResult` | |

### 4.3.5 Playback and viewers

| Command | Request | Response | Notes |
|---|---|---|---|
| `playback_set_sequence` | `{ sequenceId }` | — | |
| `playback_play` | `{ rate: number }` (−8…8, not 0) | — | |
| `playback_pause` | — | — | |
| `playback_seek` | `{ time: flicks, mode: "exact" \| "scrub" }` | — | Scrub requests are coalesced |
| `playback_step` | `{ frames: int }` | — | |
| `playback_set_loop` | `{ range: { start, end } \| null }` | — | |
| `playback_set_preview_resolution` | `{ value: "auto" \| "full" \| "half" \| "quarter" }` | — | |
| `playback_state` | — | `PlaybackState` | |
| `viewer_configure` | `{ viewer: "program" \| "source", widthPx, heightPx, devicePixelRatio, visible }` | — | Engine renders at this size × preview scale |
| `viewer_set_source` | `{ mediaId \| null, in?, out? }` | — | |
| `viewer_frame_ack` | `{ viewer, frameId }` | — | 4.5 |
| `viewer_sample_color` | `{ viewer, x, y }` (normalized) | `[r, g, b, a]` | Eyedropper (REQ-FX-012) |
| `viewer_fullscreen` / `viewer_external` | `{ on }` / `{ monitorIndex \| null }` | — | Native window (REQ-PLY-008) |
| `viewer_compare` | `{ clipId \| null, split: 0..1 }` | — | REQ-COL-004 |

### 4.3.6 Effects, titles, audio, captions

| Command | Request | Response | Notes |
|---|---|---|---|
| `fx_catalog` | — | `EffectCatalog { effects, transitions, generators, masks, titleAnimations, templates }` (manifests, 4.8) | Cached by the UI |
| `fx_presets` / `fx_preset_save` / `fx_preset_delete` | — / `{ name, effect }` / `{ name }` | `EffectPreset[]` / — | 03 §3.10 |
| `title_templates` / `title_template_save` / `title_template_delete` | — / `{ name, title }` / `{ name }` | `TitleTemplate[]` / — | REQ-TTL-005 |
| `audio_devices` | — | `{ outputs: AudioDevice[], inputs: AudioDevice[] }` | |
| `audio_set_output` | `{ deviceId \| "default" }` | — | E4001 |
| `audio_record_start` / `audio_record_stop` | `{ trackId, inputDeviceId, punch: bool }` / — | — / `EditResult` | REQ-AUD-006 |
| `audio_loudness_analyze` | `{ target: { clipIds } \| { sequenceId } }` | `JobId` | |
| `audio_normalize` | `{ clipIds, targetLufs, ceilingDbtp }` | `JobId` (result applies an `EditOp::Batch`) | REQ-AUD-004 |
| `audio_auto_duck` | `{ musicTrackIds, dialogueTrackIds, amountDb, thresholdDb, attackMs, releaseMs }` | `JobId` (result applies keyframes) | REQ-AUD-005 |
| `caption_import` | `{ sequenceId, path, encoding?: string }` | `EditResult` | E2011, E2012 |
| `caption_export` | `{ captionTrackId, format: "srt" \| "vtt" \| "ass", path }` | — | 4.11 |

### 4.3.7 Speech-to-text

| Command | Request | Response | Notes |
|---|---|---|---|
| `stt_models` | — | `ModelInfo[] { id, file, sizeBytes, installed, sha256, guidanceKey }` | MDL catalog |
| `stt_download` / `stt_delete` | `{ modelId }` | `JobId` / — | E7002, E7003 |
| `stt_transcribe` | `TranscribeRequest { source: { clipIds } \| { sequenceId }, modelId, language: "auto" \| bcp47, translate, useGpu }` | `JobId` | Result creates a new caption track (REQ-STT-004) |
| `stt_transcript` | `{ mediaId }` | `Transcript \| null` | PNL-15 |

### 4.3.8 Export and interchange

| Command | Request | Response | Notes |
|---|---|---|---|
| `export_presets` | — | `ExportPreset[]` (built-in + user) | |
| `export_preset_save` / `_delete` / `_import` / `_export` | per 03 §3.10 | | REQ-EXP-010 |
| `export_encoders` | — | `EncoderAvailability { h264, hevc, av1: { hardware: string \| null, software } }` | |
| `export_preflight` | `ExportSettings` (4.6.3) | `PreflightReport { ok, estimatedBytes, freeBytes, resolvedEncoder, warnings[], errors[] }` | REQ-EXP-007 |
| `export_enqueue` | `ExportSettings` | `JobId` | |
| `export_frame` | `{ viewer, format: "png" \| "jpeg", path }` | `{ path }` | REQ-EXP-004 |
| `interop_export_edl` | `{ sequenceId, path }` | — | REQ-INT-001 |
| `interop_export_markers` | `{ sequenceId, format: "chapters" \| "csv", path }` | — | REQ-INT-002 |
| `interop_import_markers` | `{ sequenceId, path }` | `EditResult` | |

### 4.3.9 Jobs, settings, diagnostics, updates, system

| Command | Request | Response | Notes |
|---|---|---|---|
| `jobs_list` | — | `JobInfo[]` | |
| `jobs_cancel` / `jobs_retry` | `{ jobId }` | — | |
| `jobs_pause_queue` | `{ paused }` | — | Running jobs continue; no new jobs start |
| `jobs_clear_finished` | — | — | |
| `jobs_log` | `{ jobId, tailLines }` | `string` | |
| `settings_get` | — | `SettingsMap` | 03 §3.6 |
| `settings_set` | `{ patch: SettingsMap }` | `SettingsMap` | Validates per key |
| `settings_reset` | `{ scope: "all" \| <page> }` | `SettingsMap` | |
| `settings_schema` | — | JSON Schema | Drives DLG-05 |
| `keymap_import` / `keymap_export` | `{ path }` | `SettingsMap` / — | |
| `diag_system_info` | — | `SystemInfo` | |
| `diag_create_bundle` | `{ path, includeCrashDumps }` | `{ path, files[] }` | REQ-REL-004 |
| `diag_report_problem_url` | `{ summary }` | `string` (GitHub new-issue URL with a prefilled template) | REQ-HLP-002 |
| `update_check` / `update_install` / `update_rollback` | — | `UpdateInfo \| null` / — / — | REQ-UPD-001..002 |
| `system_reveal` | `{ path }` | — | Path must be inside an allowed location (SEC-07) |
| `system_open_url` | `{ url }` | — | Only allowlisted URLs (SEC-09) |

File and folder pickers use the dialog plugin from the UI with the scoped permissions of 05 §5.4.

---

## 4.4 `EditOp` contract

`EditOp` is a tagged enum (`#[serde(tag = "op")]`). The model actor applies it to the current state, validates invariants (03 §3.5), stores the new root in the undo history, and returns an `EditResult`. Every variant has a localization key `history.<op>` for the History panel.

| `op` | Fields | Semantics |
|---|---|---|
| `addMedia` | `items: MediaItem[]`, `binId?` | Adds probed media |
| `removeMedia` | `mediaIds` | Fails (E1102) if a clip uses the media unless `alsoRemoveClips: true` |
| `updateMedia` | `mediaId`, `changes { name?, label?, notes?, binId?, path?, relPath?, fingerprint?, derived? }` | |
| `createBin` / `updateBin` / `deleteBin` | bin fields | Delete moves contents to the parent bin |
| `createSequence` | `name`, `format`, `fromPreset?` | |
| `duplicateSequence` / `deleteSequence` / `updateSequence` | `sequenceId`, changes (`name`, `format`, `inPoint`, `outPoint`) | Frame-rate change snaps items and warns E1104 |
| `addTrack` / `removeTrack` / `moveTrack` / `updateTrack` | `sequenceId`, `kind`, `index`, changes (name, height, locked, visible, muted, solo, volumeDb, pan, targeted) | |
| `insertClips` | `sequenceId`, `at`, `sources: ClipSource[]`, `targets { video?: trackId, audio?: trackId[] }`, `mode: "insert" \| "overwrite" \| "append" \| "replace"` | Insert ripples later clips on all **unlocked targeted** tracks; overwrite replaces the range; append places at sequence end |
| `moveClips` | `clipIds`, `deltaTime`, `deltaTrack`, `mode: "overwrite" \| "insert"` | Linked/grouped clips move together when linked selection is on |
| `trimClip` | `clipId`, `edge: "in" \| "out"`, `delta`, `mode: "normal" \| "ripple" \| "roll"` | Limited by handles (E1103) and neighbors |
| `slipClip` | `clipId`, `delta` | Changes `sourceIn` only |
| `slideClip` | `clipId`, `delta` | Moves the clip; neighbors' adjacent edges adjust |
| `splitClips` | `sequenceId`, `at`, `clipIds?` (default: all unlocked targeted tracks) | Effects, keyframes (re-based), and masks copy to both parts |
| `deleteClips` | `clipIds`, `ripple: bool` | |
| `liftRange` / `extractRange` | `sequenceId`, `start`, `end`, `trackIds?` | Extract ripples |
| `closeGaps` | `sequenceId`, `trackId`, `gapStart?` | One gap or all gaps |
| `linkClips` / `unlinkClips` / `groupClips` / `ungroupClips` | `clipIds` | |
| `setClipTiming` | `clipId`, `speed?`, `reverse?`, `freezeAt?`, `rippleAfter: bool` | Duration recomputed from speed when `keepDuration: false` |
| `updateClip` | `clipId`, `changes { name?, label?, enabled?, deinterlace?, streamIndex?, audio? }` | |
| `setParam` | `target: ParamTarget`, `param`, `value`, `at?: flicks` | If the parameter is keyframed and `at` is given, sets/creates the keyframe at `at`; else sets the static value |
| `setAnimated` | `target`, `param`, `animated: bool`, `at` | Enabling creates a keyframe at `at` with the current value; disabling keeps the value at `at` |
| `addKeyframes` / `removeKeyframes` / `moveKeyframes` / `setEasing` | `target`, `param`, keyframe times, `easing?`, `handles?`, `delta?` | |
| `addEffect` / `removeEffect` / `moveEffect` / `setEffectEnabled` | `owner: { clipId } \| { trackId } \| { master: sequenceId }`, `effectId`, `index`, `instanceId` | Initial params from manifest defaults or a preset |
| `pasteAttributes` | `sourceClipId`, `targetClipIds`, `parts { intrinsic, effects, audio, masks }` | REQ-EDT-008 |
| `addMask` / `updateMask` / `removeMask` | `clipId`, mask fields | |
| `addTransition` / `updateTransition` / `removeTransition` | `trackId`, `fromClipId`, `toClipId`, `transitionId`, `duration`, `alignment`, `params` | Shortens to available handles with E1101 |
| `addTitle` / `updateTitle` | `sequenceId`, `trackId`, `at`, `duration`, `title` | |
| `addGenerator` | `sequenceId`, `trackId`, `at`, `duration`, `generatorId`, `params` | |
| `nestClips` | `clipIds`, `name` | Creates a sequence and replaces the clips with one nested clip |
| `addMarkers` / `updateMarker` / `removeMarkers` | `owner: { sequenceId } \| { clipId }`, marker fields | |
| `addCaptionTrack` / `updateCaptionTrack` / `removeCaptionTrack` | caption track fields | |
| `addCaptions` / `updateCaption` / `removeCaptions` / `splitCaption` / `mergeCaptions` / `shiftCaptions` | `captionTrackId`, items / `captionId`, `at` / `delta` | |
| `renameProject` | `name` | |
| `batch` | `ops: EditOp[]`, `labelKey` | All-or-nothing; one undo step |

`ParamTarget` = `{ clipId, intrinsic: "transform" \| "crop" \| "opacity" \| "audio" }` \| `{ clipId, effectInstanceId }` \| `{ clipId, maskId }` \| `{ clipId, titleLayerId }` \| `{ trackId, effectInstanceId }` \| `{ transitionId }` \| `{ sequenceId, masterEffectInstanceId }`.

`EntityPatch`:

```json
{ "revision": 42,
  "upsert": { "media": [], "bins": [], "sequences": [], "tracks": [], "clips": [], "transitions": [],
              "markers": [], "captionTracks": [], "captions": [], "assets": [] },
  "remove": { "media": [], "bins": [], "sequences": [], "tracks": [], "clips": [], "transitions": [],
              "markers": [], "captionTracks": [], "captions": [], "assets": [] },
  "dirtyRanges": [ { "sequenceId": "…", "start": 0, "end": 705600000 } ] }
```

The UI store is normalized by these entity kinds; applying a patch replaces upserted entities and deletes removed IDs. A gap in `revision` (missed event) makes the UI call `project_snapshot`.

Interactive gestures (drag, trim) are previewed locally in the UI and committed as a single `EditOp` on pointer release (02 §2.15).

---

## 4.5 Preview frame transport

**Producer (CMP-12, `preview::shared_buffer`)**

1. Obtain `ICoreWebView2Environment12` from `PlatformWebview::environment()` and `ICoreWebView2_17` from the controller's `CoreWebView2` (inside `Webview::with_webview`).
2. Per visible viewer keep a ring of 3 slots; each slot is a shared buffer of `width × height × 4` bytes for the current render size (recreated on `viewer_configure` size changes).
3. Slot states: `Free → Writing → Posted → Free` (after acknowledgement). The presenter only writes `Free` slots; with no `Free` slot the frame is dropped and counted.
4. A frame is written as RGBA8, top-down rows, stride `width × 4`, then posted with `PostSharedBufferToScript(buffer, READ_ONLY, additionalDataAsJson)` where the JSON is:

```json
{ "type": "frame", "viewer": "program", "frameId": 1873, "slot": 1,
  "width": 1280, "height": 720, "stride": 5120,
  "time": 1270080000, "revision": 42, "colorSpace": "rec709-display" }
```

**Consumer (UI, `app/src/ipc/preview.ts`)**

1. Listen for `sharedbufferreceived` on `window.chrome.webview`; read the additional data and the buffer.
2. Upload with `texSubImage2D` (or `texImage2D` on size change) into the viewer's WebGL2 texture; draw on the next animation frame.
3. Release the script buffer and call `viewer_frame_ack({ viewer, frameId })`.

The exact WebView2 event names and the buffer release call MUST be verified against the WebView2 documentation and the generated `webview2-com` bindings during WP-11 (G-10); T-PLY-001 covers the path end to end.

**Fallback (P1):** if creating shared buffers fails, frames are sent through a `tauri::ipc::Channel` as raw bytes at a maximum of 960×540, and the status bar shows "Reduced preview".

**Native outputs:** full screen and external preview render directly into a wgpu swap chain on their own top-level window (no shared buffer).

---

## 4.6 Worker protocol

### 4.6.1 Launch

`AfterLeads-Video-Spirits.exe --worker --job "<temp>\job-<id>.json"` with stdin and stdout piped (UTF-8 JSON Lines) and stderr redirected to the job log. Environment: `ALVS_WORKER=1`, `ALVS_CRASH_SERVER=<name>`, `ALVS_LOG_DIR=<logs>`.

Job file:

```json
{ "format": "afterleads-video-spirits.job", "schemaVersion": 1, "protocol": 1,
  "jobId": "…", "kind": "export",
  "paths": { "cacheDir": "…", "tempDir": "…", "logsDir": "…", "modelsDir": "…" },
  "env": { "hardwareDecode": true, "useHardwareEncoder": true, "gpuAdapter": "auto", "forceWarp": false, "sttUseGpu": true },
  "spec": { } }
```

### 4.6.2 Messages

Worker → core (one JSON object per line):

| `t` | Fields | When |
|---|---|---|
| `hello` | `pid`, `version`, `protocol` | First line; version must equal the GUI's |
| `progress` | `fraction` 0–1, `stage`, `etaSec?`, `detail?` | At most 10 per second |
| `heartbeat` | — | Every 2 s |
| `log` | `level`, `msg` | Optional |
| `result` | `outputs: [{ kind, path, sha256? }]`, `data?` | Success, then exit code 0 |
| `error` | `code`, `message`, `retryable`, `detail?` | Failure, then exit code 1 |

Core → worker: `{"t":"cancel"}` — the worker stops, deletes partial outputs, exits with code 2.

Supervision: no `heartbeat` for 15 s → kill → E6002. Exit without `result`/`error` → E6001. Retryable failures are retried with backoff 2 s then 10 s (3 attempts total) (REQ-JOB-003). Exit code 3 = protocol or usage error (E6005).

### 4.6.3 Job specifications

| `kind` | `spec` fields | `result.data` |
|---|---|---|
| `probe` | `path` | `ProbeResult` (03 §3.4.2 `probe` + full stream list) |
| `thumbnails` | `mediaPath`, `mediaKey`, `kinds`, `height`, `intervalFlicks` | file paths written into the cache |
| `waveform` | `mediaPath`, `mediaKey`, `stream` | waveform file path |
| `proxy` | `mediaPath`, `mediaKey`, `height` (540/720/1080) | proxy path |
| `optimize` | `mediaPath`, `mediaId`, `presetId` (`xp.prores.lt`), `outputPath` | — |
| `stabilize` | `mediaPath`, `mediaId`, `strength`, `crop`, `outputPath` | — (FFmpeg `deshake` with search range `16 + round(48 × strength)` px, `edge=mirror` when crop is `none`, else a 5% center crop) |
| `loudness` | `sources: [{ mediaPath, stream, start, end }]` or `snapshotPath` + `sequenceId` | `{ integratedLufs, truePeakDbtp, lra, shortTermSeries }` |
| `transcribe` | `source` (media path + range, or snapshot + sequence for mixdown), `modelPath`, `language`, `translate`, `useGpu` | `Transcript { language, segments: [{ start, end, text, words: [{ start, end, text, p }] }] }` |
| `export` | `snapshotPath`, `sequenceId`, `settings: ExportSettings` | `{ verified: true, durationFlicks, frames }` |
| `modelDownload` | `url`, `sha256`, `size`, `targetPath` | — |
| `collect` | `snapshotPath`, `targetFolder` | `{ copiedFiles, bytes }` |

`ExportSettings`:

```json
{ "presetId": "xp.h264.high",
  "container": "mp4",
  "video": { "enabled": true, "codec": "h264", "encoder": "auto", "width": 1920, "height": 1080,
             "scaleMode": "fit", "bitDepth": 8, "chroma": "420", "rateControl": "crf", "crf": 18,
             "bitrateKbps": null, "maxBitrateKbps": null, "keyframeIntervalSec": 2, "bFrames": 3,
             "profile": "high", "speed": "medium" },
  "audio": { "enabled": true, "codec": "aac", "bitrateKbps": 320, "sampleRate": 48000, "channels": 2,
             "normalize": null },
  "range": { "kind": "sequence", "start": 0, "end": 0 },
  "captions": { "mode": "none", "trackIds": [], "sidecarFormat": "srt" },
  "chapters": true,
  "metadata": { "title": "", "artist": "", "comment": "" },
  "imageSequence": null,
  "output": { "folder": "D:\\Exports", "fileName": "Interview Cut.mp4", "overwrite": "ask" } }
```

`encoder`: `auto` (hardware when `alvs.export.useHardwareEncoder` and available) \| `software` \| `hardware`. `range.kind`: `sequence` \| `inOut` \| `selection`. `captions.mode`: `none` \| `burnIn` \| `sidecar` \| `embed`. `normalize`: `null` or `{ targetLufs, ceilingDbtp }`.

### 4.6.4 Scheduling

| Kind | Priority | Max concurrent |
|---|---|---|
| probe | 90 | 2 (always allowed, in addition to the global limit) |
| thumbnails, waveform | 80 | 2 |
| export | 70 | 1 |
| transcribe | 60 | 1 |
| proxy, optimize, stabilize | 50 | 1 each |
| loudness | 40 | 1 |
| collect | 30 | 1 |
| modelDownload | 20 | 2 |

Global limit: `alvs.advanced.maxParallelJobs` (default 2) for all kinds except probe. Jobs of the same priority run first-in, first-out.

---

## 4.7 CLI

`AfterLeads-Video-Spirits.exe cli <command> [options]`

| Command | Options | Output |
|---|---|---|
| `version` | — | Version, commit, FFmpeg version, wgpu adapter |
| `probe <file>` | `--json` | Probe result |
| `validate <project.alvs>` | `--json` | Validation issues (03 §3.5) |
| `render` | `--project <file> --sequence <id\|name> --preset <xp-id> --out <file> [--range <tc>-<tc>] [--software] [--warp]` | Progress lines; exit code |
| `frame` | `--project <file> --sequence <id\|name> --at <timecode> --out <file.png> [--warp]` | PNG |
| `schema` | `--kind project\|settings\|effect\|job --out <file>` | JSON Schema |

Exit codes: 0 success; 1 runtime error (stderr starts with the E-code); 2 usage error; 3 cancelled; 4 validation failed.

---

## 4.8 Effect manifest contract

Every catalog entry in 12 §12.1–12.6 and 12 §12.12 has one manifest compiled into the binary (`crates/alvs-render/src/effects/*/manifest.json`, `crates/alvs-audio/src/effects/*/manifest.json`, …). The manifest is the single source for parameters at runtime; T-FX-004 checks that manifests and 12 match exactly.

```json
{ "format": "afterleads-video-spirits.effect", "schemaVersion": 1,
  "id": "fx.color.basic", "version": 1, "kind": "video", "category": "color", "release": "1.0",
  "nameKey": "fx.color.basic.name", "descriptionKey": "fx.color.basic.description",
  "params": [
    { "id": "exposure", "type": "float", "min": -4, "max": 4, "default": 0, "step": 0.01,
      "unit": "ev", "animatable": true,
      "ui": { "control": "slider", "group": "light", "precision": 2, "labelKey": "fx.color.basic.exposure" } }
  ],
  "implementation": { "shader": "basic.wgsl", "passes": [ { "entry": "main", "output": "dst" } ] },
  "alpha": "preserve", "cost": "low", "migrations": [] }
```

| Field | Rules |
|---|---|
| `kind` | `video` \| `audio` \| `transition` \| `generator` \| `mask` \| `titleAnimation` |
| `params[].type` | `float`, `int`, `bool`, `enum` (with `values: [{ value, labelKey }]`), `color`, `point`, `angle`, `wheel`, `curve`, `lut`, `text` |
| `params[].ui.control` | `slider`, `number`, `dial`, `checkbox`, `dropdown`, `color`, `point`, `wheel`, `curve`, `lutPicker`, `textArea` |
| `unit` | `none`, `percent`, `ev`, `deg`, `px` (normalized length displayed in pixels), `db`, `dbtp`, `hz`, `ms`, `s`, `semitones`, `cents`, `lufs` |
| `implementation` | `shader` + `passes` for GPU kinds; `dsp: "<module>"` for audio |
| `alpha` | `preserve` \| `generate` \| `ignore` |
| `migrations` | `[{ fromVersion, renames: { old: new }, defaults: { param: value } }]` applied when a project stores an older `version` |

Rules: IDs unique; `min ≤ default ≤ max`; every `labelKey`/`nameKey` exists in the `en` catalog; boolean and enum params are not animatable.

---

## 4.9 Error envelope

```json
{ "code": "E2001", "severity": "warning", "messageKey": "error.E2001",
  "params": { "name": "interview.mp4" }, "detail": "path redacted: …\\interview.mp4",
  "retryable": false,
  "recovery": [ { "commandId": "media.relink", "labelKey": "action.relink" } ] }
```

- `severity`: `info` (toast, auto-dismiss), `warning` (toast, persistent in notification center), `error` (toast + details button; modal only when data could be lost).
- `messageKey` always equals `error.<code>`; texts live in the locale catalogs (12 §12.13 has the English source).
- `detail` is for logs and diagnostics; it is shown under "Details" and never contains secrets.
- `recovery[].commandId` is a command ID from 12 §12.10 or a dialog action.

---

## 4.10 Events

| Event | Payload | Rate |
|---|---|---|
| `model:patch` | `{ revision, patch, labelKey, source: "edit" \| "undo" \| "redo" \| "load" \| "job" }` | Per edit |
| `model:dirty` | `{ dirty }` | On change |
| `playback:state` | `PlaybackState { sequenceId, state: "paused" \| "playing" \| "scrubbing", rate, time, loop, droppedFrames, fps }` | On change |
| `playback:time` | `{ time }` | ≤ 30 Hz while playing |
| `audio:meters` | `{ master: { peak: [l, r], rms: [l, r] }, tracks: { id: { peak, rms } }, loudness: { m, s, i, tp } }` | 30 Hz while playing, 4 Hz idle |
| `scopes:data` | Channel per subscription (`scopes_subscribe { kinds, channel }`): binary frames | ≤ 15 Hz |
| `jobs:update` | `JobInfo` | On change, progress ≤ 4 Hz per job |
| `media:updated` | `{ mediaId, what: "probe" \| "thumbnail" \| "waveform" \| "proxy" \| "derived" }` | On change |
| `app:notify` | `ErrorEnvelope` or `{ severity: "info", messageKey, params }` | On event |
| `autosave:status` | `{ state: "idle" \| "saving" \| "saved" \| "failed", timeUtc }` | On change |
| `update:available` | `{ version, notesUrl, channel }` | On check |
| `app:open-file` | `{ path }` | Single-instance forwarding |
| `renderer:changed` | `{ name, isWarp }` | On device change |

---

## 4.11 File interchange contracts

### 4.11.1 Captions

| Format | Import | Export |
|---|---|---|
| SRT | Optional index lines; `HH:MM:SS,mmm --> HH:MM:SS,mmm`; tags `<i> <b> <u> <font>` are removed (count reported) | UTF-8 with BOM, CRLF, indices from 1 |
| WebVTT | `WEBVTT` header; cue IDs ignored; `NOTE`/`STYLE` blocks skipped; cue settings `line` (%) → position, `align` → alignment | UTF-8 without BOM, LF |
| ASS/SSA | `[Events]` Dialogue lines (`H:MM:SS.cc`); the most used `Style` maps to the track style; override tags `{\…}` removed | UTF-8 with BOM, CRLF; `PlayResX/Y` = sequence size; one Style from the track style |

Times convert exactly: 1 ms = 705,600 flicks. Export rounds to milliseconds (centiseconds for ASS). Encoding detection order: BOM → strict UTF-8 → UTF-16 heuristics → Windows-1256 (Arabic letter frequency) → Windows-1252, with confirmation for the last two (E2012).

### 4.11.2 Chapter list and CSV markers

- Chapter list: one line per chapter marker (all markers when none is a chapter), `M:SS Title` below one hour and `H:MM:SS Title` otherwise; if the first marker is after 0, a first line `0:00 <sequence name>` is added.
- CSV: UTF-8 with BOM, RFC 4180 quoting, header `Start,End,Name,Color,Kind,Note`, times as timecode of the sequence; import accepts the same header in any column order.

### 4.11.3 EDL (CMX 3600)

`TITLE: <sequence name>`, `FCM: DROP FRAME` or `NON-DROP FRAME`; events for the lowest video track (`V`) and the first two audio tracks (`A`, `A2`), reel `AX`, cut (`C`) and dissolve (`D <frames>`) only; each event followed by `* FROM CLIP NAME: <clip or file name>`. Other transitions export as cuts with a comment. Times use the sequence timecode starting at 00:00:00:00.

### 4.11.4 `.cube` LUTs

Keywords `TITLE`, `LUT_1D_SIZE` (2–65536), `LUT_3D_SIZE` (2–65), `DOMAIN_MIN`, `DOMAIN_MAX`; `#` comments; data rows of three floats with red changing fastest; row count must equal `size` (1D) or `size³` (3D). Maximum file size 64 MB. Errors report line and column (E2010).

---

## 4.12 Core Rust traits

```rust
// alvs-core
pub type Flicks = i64;
pub trait Clock: Send + Sync { fn now(&self) -> Flicks; }

// alvs-media
pub trait MediaDecoder: Send {
    fn seek(&mut self, t: Flicks) -> Result<(), MediaError>;
    fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, MediaError>;
    fn stream_info(&self) -> &VideoStreamInfo;
}

// alvs-render
pub trait EffectRuntime: Send + Sync {
    fn manifest(&self) -> &EffectManifest;
    fn prepare(&mut self, gpu: &GpuContext) -> Result<(), RenderError>;
    fn encode(&self, pass: &mut EffectPass<'_>, params: &ResolvedParams) -> Result<(), RenderError>;
}

// alvs-audio
pub trait AudioNode: Send {
    fn process(&mut self, block: &mut AudioBlock, ctx: &ProcessContext);
    fn latency_samples(&self) -> u32;
    fn reset(&mut self);
}

// alvs-engine
pub trait PreviewSink: Send {
    fn present(&mut self, frame: PreviewFrame) -> Result<(), EngineError>;
    fn resize(&mut self, width: u32, height: u32);
}

// alvs-jobs
pub trait JobRunner: Send + Sync {
    fn kind(&self) -> JobKind;
    fn run(&self, spec: &serde_json::Value, ctx: &mut WorkerContext) -> Result<JobOutput, AppError>;
}
```

Signatures may gain parameters during implementation only through a docs pull request that updates this section (G-19).

---

## 4.13 Compatibility rules

- UI and core ship together; IPC has no cross-version compatibility requirement, but a WebView reload MUST recover by calling `project_snapshot`.
- The worker `protocol` and `version` must match the GUI; a mismatch fails the job with E6005.
- File formats (`.alvs`, settings, library, manifests) carry `schemaVersion` and follow 03 §3.9.6.
