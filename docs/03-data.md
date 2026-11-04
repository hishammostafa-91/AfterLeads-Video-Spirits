# 03 — Data

Canonical for: data ownership, units, the `.alvs` project format, invariants, settings keys, cache database, file locations, and data lifecycle (save, autosave, backups, locks, migrations, eviction).
Depends on: [02-architecture.md](02-architecture.md) (components), [04-interfaces.md](04-interfaces.md) (EditOp, DTOs), [12-catalogs.md](12-catalogs.md) (IDs of effects, presets, colors).

---

## 3.1 Data stores and ownership

| Store | Owner | Location (3.8) | Lifetime | Loss impact |
|---|---|---|---|---|
| Project file `.alvs` | User | User-chosen project folder | Permanent | High — protected by atomic save, autosave, backups |
| Autosave sidecar | App (for the user) | Next to the project file | Until next manual save or clean close | Recovery source |
| Backups | App (for the user) | `<project folder>/Backups/` | Newest N kept (REQ-PRJ-004) | Recovery source |
| Project assets (LUTs, recordings, derived media) | User | Inside the project folder | Permanent | Medium |
| Settings, keymap, workspaces | User | App config folder | Permanent | Low (defaults restore) |
| User library (presets, templates) | User | App config folder | Permanent | Medium |
| Cache database and cache files | App | App local-data folder | Rebuildable; evicted by LRU | None (rebuilt) |
| Proxies | App | Cache or project folder (setting) | Rebuildable | None (rebuilt) |
| Speech models | App | App local-data folder | Until deleted | None (re-downloaded) |
| Logs, crash dumps | App | App local-data folder | Rolling | None |
| Job snapshots and temp files | App | `%TEMP%\AfterLeads-Video-Spirits\<pid>\` | Deleted at exit and at next start | None |

No data is ever stored outside these locations, and nothing is sent over the network (NFR-PRIV-01).

---

## 3.2 Conventions

1. JSON files are UTF-8 without BOM, LF line endings, 2-space indentation, keys in `camelCase`.
2. IDs are UUID v7 strings (lowercase, hyphenated). They are unique within a project and never reused.
3. Every persisted object keeps unknown fields: Rust structs carry `#[serde(flatten)] extra: BTreeMap<String, serde_json::Value>` so files from newer versions do not lose data when re-saved (read-only rules in 3.9.6 still apply).
4. Enumerations are stored as their catalog string IDs (for example `"fx.color.basic"`, `"normal"`, `"CLR-03"`).
5. Numbers: integers for times (flicks), counts, and sizes; floats (f64 in Rust, `number` in TS) for parameter values.
6. ts-rs maps Rust `i64` to TypeScript `bigint` by default. Every flicks field MUST be annotated `#[ts(type = "number")]` (values stay below 2^53, see 3.3.1).

---

## 3.3 Units

### 3.3.1 Time: flicks

- 1 second = **705,600,000 flicks**. All persisted times and durations are signed 64-bit integers in flicks.
- Every frame rate in 12 §12.9 and the audio rates 48,000 and 44,100 Hz have an integer duration in flicks:

| Rate | Flicks per frame/sample |
|---|---|
| 23.976 (24000/1001) | 29,429,400 |
| 24 | 29,400,000 |
| 25 | 28,224,000 |
| 29.97 (30000/1001) | 23,543,520 |
| 30 | 23,520,000 |
| 50 | 14,112,000 |
| 59.94 (60000/1001) | 11,771,760 |
| 60 | 11,760,000 |
| 48,000 Hz sample | 14,700 |
| 44,100 Hz sample | 16,000 |

- Frame `n` of a sequence starts at exactly `n × frameFlicks`. A custom frame rate `num/den` is valid only if `705600000 × den` is divisible by `num` and 1 ≤ num/den ≤ 120.
- Source timestamps from media with other rates are converted to flicks with round-half-to-even (error < 1.5 ns).
- Maximum sequence length: 24 hours (6.1 × 10^13 flicks < 2^53).
- Timecode formatting and parsing live in `alvs-core::time` and are the only conversion code for display (REQ-PLY-005).

### 3.3.2 Rational numbers

Frame rates and clip speeds are `{ "num": u32, "den": u32 }`, always reduced, `den > 0`.

### 3.3.3 Space

- Positions and sizes in effect and transform parameters are normalized: `x` in frame widths, `y` and all lengths (radius, feather, distance) in frame heights. (0, 0) is the top-left corner, (1, 1) the bottom-right, (0.5, 0.5) the center.
- The UI converts to pixels of the sequence resolution for display and input.
- Angles are degrees, clockwise.

### 3.3.4 Audio levels

Gains in dB (f64); −∞ is stored as `-200` and displayed as "−∞". Pan is −1 (left) … +1 (right).

---

## 3.4 Project file (`.alvs`)

### 3.4.1 Top level

```json
{
  "format": "afterleads-video-spirits.project",
  "schemaVersion": 1,
  "id": "01933b6e-7a1c-7c2e-9b1a-3f0d6c2a8e11",
  "name": "Interview Cut",
  "createdUtc": "2026-11-01T10:00:00Z",
  "modifiedUtc": "2026-11-01T12:30:05Z",
  "savedBy": { "app": "AfterLeads-Video-Spirits", "version": "1.0.0", "commit": "3f2a9c1" },
  "settings": { "defaultSequenceId": "01933b6e-…", "proxyMode": false },
  "media": [],
  "bins": [],
  "assets": [],
  "sequences": [],
  "ui": {}
}
```

| Field | Type | Rules |
|---|---|---|
| `format` | string | Exactly `afterleads-video-spirits.project` |
| `schemaVersion` | int | 1 for release 1.0; migrations are forward-only (3.9.6) |
| `id`, `name` | string | Name 1–200 characters |
| `createdUtc`, `modifiedUtc` | RFC 3339 UTC | `modifiedUtc ≥ createdUtc` |
| `savedBy` | object | Application name, version, commit of the last writer |
| `settings` | object | Project-level options (below) |
| `media`, `bins`, `assets`, `sequences` | arrays | See 3.4.2–3.4.5; at least one sequence |
| `ui` | object | Non-undoable view state (3.4.11) |

`settings`: `defaultSequenceId` (string), `proxyMode` (bool, "Use proxies" toggle, REQ-MED-005).

### 3.4.2 Media item

```json
{
  "id": "…", "kind": "video", "name": "interview.mp4",
  "path": "D:\\Footage\\interview.mp4", "relPath": "..\\Footage\\interview.mp4",
  "fingerprint": { "size": 734003200, "mtimeUtc": "2026-10-30T08:12:00Z", "headTailSha256": "9a1…" },
  "probe": {
    "durationFlicks": 1270080000000, "container": "mov,mp4,m4a,3gp,3g2,mj2",
    "streams": [
      { "index": 0, "type": "video", "codec": "hevc", "width": 3840, "height": 2160,
        "frameRate": { "num": 30000, "den": 1001 }, "vfr": true, "bitDepth": 10, "chroma": "420",
        "primaries": "bt2020", "transfer": "arib-std-b67", "matrix": "bt2020nc", "range": "limited",
        "hdr": "hlg", "rotation": 90, "interlaced": false },
      { "index": 1, "type": "audio", "codec": "aac", "channels": 2, "layout": "stereo", "sampleRate": 48000 }
    ]
  },
  "imageSequence": null,
  "binId": null, "label": null, "notes": "",
  "derived": { "optimized": null, "stabilized": null },
  "addedUtc": "2026-11-01T10:05:00Z"
}
```

| Field | Rules |
|---|---|
| `kind` | `video` \| `audio` \| `image` \| `imageSequence` |
| `path` / `relPath` | Absolute path and path relative to the project folder; on open, `relPath` is tried when `path` is missing (REQ-PRJ-002) |
| `fingerprint` | Size, modification time, SHA-256 of the first and last MiB; used for relinking (REQ-PRJ-010) and cache keys (`mediaKey` = SHA-256 of the canonical JSON of the fingerprint) |
| `probe` | Snapshot of probe results needed to edit offline; the cache database holds the full probe |
| `imageSequence` | `{ "pattern": "shot_%04d.png", "first": 1, "count": 240, "frameRate": {…} }` for `kind = imageSequence` |
| `label` | `CLR-01`..`CLR-08` or null |
| `derived.optimized` | `{ "relPath": "Derived/Optimized/<id>.mov", "preset": "xp.prores.lt", "active": true }` or null (REQ-MED-011) |
| `derived.stabilized` | `{ "relPath": "Derived/Stabilized/<id>.mov", "strength": 0.5, "crop": "auto", "active": true }` or null (REQ-MED-013) |

Probe enums use FFmpeg's names for codecs, primaries, transfer, matrix, and range.

### 3.4.3 Bin

`{ "id", "name", "parentId": null | binId, "label": null | "CLR-xx", "expanded": true }`. Bins form a tree (no cycles); a media item belongs to at most one bin (`media.binId`).

### 3.4.4 Asset

`{ "id", "kind": "lut", "name": "Film.cube", "relPath": "assets/luts/<sha256>.cube", "sha256": "…", "lutInfo": { "type": "3d", "size": 33 } }`. Imported LUTs are copied into the project folder (REQ-FX-008).

### 3.4.5 Sequence

```json
{
  "id": "…", "name": "Main",
  "format": { "width": 1920, "height": 1080, "frameRate": { "num": 30, "den": 1 },
              "sampleRate": 48000, "audioLayout": "stereo", "colorSpace": "rec709-sdr",
              "dropFrameTimecode": false, "autoMatchPending": false },
  "tracks": [],
  "captionTracks": [],
  "markers": [],
  "inPoint": null, "outPoint": null,
  "masterAudio": { "volumeDb": 0, "effects": [] }
}
```

- `colorSpace` is always `rec709-sdr` in 1.0 (REQ-COL-007 adds HDR in 1.1).
- `autoMatchPending` is true for a sequence created with SQP-01 until the first video clip sets the format.
- `inPoint`/`outPoint` are flicks or null.
- `tracks` order = display order from top to bottom within each kind; video compositing order is bottom-most video track first.

### 3.4.6 Track and clip

```json
{
  "id": "…", "kind": "video", "name": "V1", "height": 64,
  "locked": false, "visible": true, "muted": false, "solo": false,
  "volumeDb": 0, "pan": 0, "targeted": true,
  "effects": [],
  "items": [
    {
      "id": "…", "type": "media", "mediaId": "…", "streamIndex": 0,
      "start": 0, "duration": 352800000, "sourceIn": 70560000,
      "speed": { "num": 1, "den": 1 }, "reverse": false, "freezeAt": null,
      "enabled": true, "name": null, "label": null,
      "linkGroupId": "…", "groupId": null,
      "intrinsic": {
        "transform": { "position": { "v": [0.5, 0.5] }, "scale": { "v": 100 },
                       "scaleX": { "v": 100 }, "scaleY": { "v": 100 }, "uniformScale": true,
                       "rotation": { "v": 0 }, "anchor": { "v": [0.5, 0.5] },
                       "flipH": false, "flipV": false },
        "crop": { "left": { "v": 0 }, "top": { "v": 0 }, "right": { "v": 0 }, "bottom": { "v": 0 },
                  "feather": { "v": 0 } },
        "opacity": { "v": 100 }, "blendMode": "normal"
      },
      "deinterlace": "auto",
      "effects": [], "masks": [], "markers": []
    }
  ],
  "transitions": []
}
```

Track fields: `kind` (`video` \| `audio`), `height` (32–240 px), `visible` (video), `muted`/`solo` (audio), `volumeDb` (−200..+12), `pan` (−1..1), `targeted` (source patching), `effects` (audio tracks only in 1.0).

Clip fields:

| Field | Rules |
|---|---|
| `type` | `media` \| `title` \| `generator` \| `sequence` |
| `mediaId`, `streamIndex` | Required for `media`; the stream used by this clip (a linked A/V pair is two clips sharing `linkGroupId`) |
| `start`, `duration` | Timeline flicks |
| `sourceIn` | Source flicks (0 for titles and generators) |
| `speed`, `reverse`, `freezeAt` | Speed 1/10 ≤ num/den ≤ 16; `freezeAt` (source flicks) shows one frame for the whole duration |
| `enabled` | Disabled clips are skipped in render and mix |
| `linkGroupId`, `groupId` | Link (A/V of the same source) and arbitrary groups (REQ-EDT-007) |
| `intrinsic` | Video clips only (12 §12.1.1) |
| `audio` | Audio clips only (below) |
| `deinterlace` | `auto` \| `off` \| `force` (REQ-MED-010) |
| `title` | Title document (3.4.9) for `type = title` |
| `generator` | `{ "generatorId": "gen.solid", "params": { … } }` for `type = generator` |
| `sequenceId` | Nested sequence for `type = sequence` |
| `effects`, `masks`, `markers` | 3.4.7, 3.4.8, 3.4.10 |

Audio clip `audio` object:

```json
{ "gainDb": 0, "volume": { "v": 0 }, "pan": { "v": 0 },
  "fadeIn": 0, "fadeOut": 0, "fadeCurve": "constantPower",
  "channelMode": "stereo", "preservePitch": true }
```

`channelMode`: `stereo` \| `leftToBoth` \| `rightToBoth` \| `monoMix` \| `swap` (REQ-AUD-008). `fadeCurve`: `linear` \| `constantPower`.

### 3.4.7 Effect instance and animatable values

```json
{ "id": "…", "effectId": "fx.color.basic", "version": 1, "enabled": true,
  "params": {
    "exposure": { "v": 0.3 },
    "saturation": { "k": [ { "t": 0, "v": 100, "e": "linear" },
                           { "t": 705600000, "v": 130, "e": "bezier", "h": [0.33, 0, 0.67, 1] } ] }
  },
  "maskIds": [] }
```

- An animatable value is either static `{ "v": value }` or keyframed `{ "k": [keyframe…] }`.
- Keyframe: `t` (flicks relative to the clip start), `v` (value), `e` (easing ID, EAS catalog, applies from this keyframe to the next), `h` (Bezier handles, only when `e = "bezier"`).
- Value types: number; `[x, y]` point; `[r, g, b, a]` color (0–1 floats); string (enum); bool; curve `[[x, y], …]`; LUT reference `"asset:<assetId>"`; wheel `{ "x": 0, "y": 0, "luma": 0 }`.
- Missing parameters take manifest defaults; values out of range are clamped on load with a warning (E1003 detail lists them).
- `maskIds` limits the effect to those masks of the same clip (REQ-FX-006).

### 3.4.8 Mask

`{ "id", "maskId": "mask.rect", "params": { "center": {"v": [0.5,0.5]}, … }, "feather": {"v": 0.01}, "expansion": {"v": 0}, "opacity": {"v": 100}, "invert": false, "mode": "add", "usage": "clip" }` — `usage` is `clip` (masks the clip) or `effect` (only referenced through `maskIds`).

### 3.4.9 Title document

```json
{ "layers": [
    { "id": "…", "text": "مرحبا Hello", "font": "Cairo", "size": { "v": 0.08 }, "weight": 700,
      "italic": false, "fill": { "v": [1, 1, 1, 1] },
      "stroke": { "color": { "v": [0, 0, 0, 1] }, "width": { "v": 0.004 } },
      "shadow": null,
      "box": { "color": { "v": [0, 0, 0, 0.6] }, "padding": 0.02, "radius": 0.01 },
      "align": "center", "lineHeight": 1.2, "letterSpacing": 0,
      "position": { "v": [0.5, 0.8] }, "rotation": { "v": 0 }, "opacity": { "v": 100 } } ],
  "animIn": { "animationId": "tan.fade", "duration": 352800000, "easing": "cubicOut" },
  "animOut": null,
  "templateId": "tpl.lowerThird.clean" }
```

`align`: `start` \| `center` \| `end` (resolved against the paragraph direction). Sizes are in frame heights.

### 3.4.10 Transitions, markers, captions

Transition (in `track.transitions`):

```json
{ "id": "…", "transitionId": "tr.dissolve.cross", "fromClipId": "…", "toClipId": "…",
  "duration": 705600000, "alignment": "center", "params": {} }
```

- `alignment`: `center` covers `[edit − d/2, edit + d/2]`; `start` covers `[edit, edit + d]`; `end` covers `[edit − d, edit]`.
- Media coverage: during a transition, clip A continues past its out point and clip B starts before its in point using their handles; the required handles are A: `d/2` (center) / `d` (start) / 0 (end) and B: `d/2` / 0 / `d`.

Marker (in `sequence.markers` with sequence time, or in `clip.markers` with source time):

`{ "id", "t": 0, "duration": 0, "name": "Intro", "color": "CLR-03", "note": "", "kind": "standard" }` — `kind`: `standard` \| `chapter`.

Caption track (in `sequence.captionTracks`):

```json
{ "id": "…", "name": "Arabic", "language": "ar", "enabled": true,
  "style": { "font": "Cairo", "size": 0.045, "weight": 600, "color": [1,1,1,1],
             "outlineColor": [0,0,0,1], "outlineWidth": 0.003,
             "shadowColor": [0,0,0,0.6], "shadowOffset": [0.002, 0.002], "shadowBlur": 0.003,
             "box": { "enabled": false, "color": [0,0,0,0.6], "padding": 0.01, "radius": 0.005 },
             "align": "center", "verticalPosition": 0.9, "maxLines": 2 },
  "items": [ { "id": "…", "start": 0, "end": 1411200000, "text": "مرحبا بكم", "position": null } ] }
```

### 3.4.11 View state (`ui`)

`{ "activeSequenceId", "playheads": { "<seqId>": flicks }, "timelineZoom": { "<seqId>": pixelsPerSecond }, "timelineScroll": { "<seqId>": { "x": flicks, "y": px } }, "sourceViewer": { "mediaId": null, "in": null, "out": null, "playhead": 0 }, "selectedTrackTargets": {…} }` — saved with the project, never part of undo history.

---

## 3.5 Invariants and validation

`alvs-model::validate(&Project) -> Vec<Issue>` checks every rule below on load and after every `EditOp` in debug builds and tests (T-EDT-011). A violated rule after an edit is a bug: the edit is rolled back and E0002 is reported.

1. All IDs are unique UUID v7 strings within the project.
2. References resolve: `clip.mediaId`, `clip.sequenceId`, `media.binId`, `bin.parentId`, `effect.maskIds`, `transition.fromClipId/toClipId`, `asset:` references, `settings.defaultSequenceId`, `ui.activeSequenceId`.
3. Nested sequences form no cycle (E1006); nesting depth ≤ 8.
4. Track items are sorted by `start` and never overlap: for consecutive clips a, b on a track, `a.start + a.duration ≤ b.start`.
5. Video and title/generator clip `start` and `duration` are multiples of the sequence frame duration; audio clips are multiples of a quarter frame (sub-frame nudge, KB-078).
6. `duration > 0`; speed reduced with 1/10 ≤ num/den ≤ 16.
7. For `media` clips: `sourceIn ≥ 0` and `sourceIn + duration × speed ≤ media duration` (images and image stills are unlimited); `freezeAt` within the media duration.
8. Transitions join two adjacent clips on the same track (`a.start + a.duration = b.start`), at most one per edit point, `0 < duration ≤` the available handles for its alignment (3.4.10).
9. Keyframes are sorted by `t`, unique per parameter, `0 ≤ t ≤ clip duration`; value types match the manifest; `h` exists only for `bezier`; boolean and enum parameters use `hold`.
10. Linked clips share the same `mediaId`; a link group has at most one video and up to two audio clips.
11. Caption items on one caption track are sorted, non-overlapping, `start < end`, text 1–1,000 characters.
12. Markers: `t ≥ 0`, `duration ≥ 0`; two chapter markers never share the same `t`.
13. Sequence format: width and height even, 16–8,192; frame rate per 3.3.1; sample rate 44,100 or 48,000.
14. Asset files exist and match `sha256` (missing → warning and the effect is bypassed, E2001).
15. Size limits (defense against malicious files, 05 THR-02): ≤ 99 tracks per kind, ≤ 20,000 clips per sequence, ≤ 100 sequences, ≤ 50,000 media items, ≤ 10,000 keyframes per parameter, ≤ 20,000 captions per track, project file ≤ 256 MB.

---

## 3.6 Settings

File: `settings.json` (3.8), a flat JSON object `{ "settingsVersion": 1, "<key>": value, … }`. Keys absent from the file take the defaults below. Unknown keys are preserved. Each key is validated; an invalid value is replaced by the default and reported once (E8002).

| Key | Type / range | Default | Page |
|---|---|---|---|
| `alvs.general.language` | `system` \| `en` \| `ar` \| `fr` \| `es` \| `de` \| `pt-BR` | `system` | General |
| `alvs.general.autosaveIntervalSec` | int 15–600 | 60 | General |
| `alvs.general.backupsKeep` | int 1–100 | 20 | General |
| `alvs.general.restoreLastProject` | bool | false | General |
| `alvs.general.showWelcome` | bool | true | General |
| `alvs.general.recentLimit` | int 5–50 | 20 | General |
| `alvs.general.defaultProjectFolder` | path | `%USERPROFILE%\Videos\AfterLeads-Video-Spirits` | General |
| `alvs.general.checkUpdates` | bool | true | General |
| `alvs.general.updateChannel` | `stable` \| `beta` | `stable` | General |
| `alvs.appearance.theme` | `dark` \| `light` \| `highContrast` \| `system` | `dark` | Appearance |
| `alvs.appearance.accent` | `CLR-06` \| `CLR-07` \| `CLR-04` \| `CLR-02` \| `CLR-08` | `CLR-06` | Appearance |
| `alvs.appearance.uiZoom` | int 80–200 (step 10) | 100 | Appearance |
| `alvs.appearance.density` | `comfortable` \| `compact` | `comfortable` | Appearance |
| `alvs.appearance.reduceMotion` | `system` \| `on` \| `off` | `system` | Appearance |
| `alvs.playback.previewResolution` | `auto` \| `full` \| `half` \| `quarter` | `auto` | Playback |
| `alvs.playback.playEveryFrame` | bool | false | Playback |
| `alvs.playback.hardwareDecode` | bool | true | Playback |
| `alvs.playback.audioDevice` | device ID string \| `default` | `default` | Playback |
| `alvs.playback.audioBuffer` | `low` \| `normal` \| `safe` | `normal` | Playback |
| `alvs.playback.audioScrub` | bool | true | Playback |
| `alvs.playback.timeDisplay` | `timecode` \| `frames` \| `seconds` | `timecode` | Playback |
| `alvs.playback.dropFrameTimecode` | bool | true | Playback |
| `alvs.playback.dualViewer` | `auto` \| `on` \| `off` | `auto` | Playback |
| `alvs.playback.gpuCacheMB` | int 256–8192 | 1024 | Playback |
| `alvs.timeline.snapping` | bool | true | Timeline |
| `alvs.timeline.linkedSelection` | bool | true | Timeline |
| `alvs.timeline.thumbnails` | `off` \| `first` \| `filmstrip` | `filmstrip` | Timeline |
| `alvs.timeline.waveforms` | bool | true | Timeline |
| `alvs.timeline.followPlayhead` | `page` \| `smooth` \| `off` | `page` | Timeline |
| `alvs.timeline.insertFitMode` | `fit` \| `fill` \| `none` | `fit` | Timeline |
| `alvs.timeline.trackHeightVideo` | int 32–240 | 64 | Timeline |
| `alvs.timeline.trackHeightAudio` | int 32–240 | 56 | Timeline |
| `alvs.timeline.defaultStillDurationSec` | float 0.1–600 | 5 | Timeline |
| `alvs.timeline.defaultTitleDurationSec` | float 0.1–600 | 5 | Timeline |
| `alvs.timeline.defaultGeneratorDurationSec` | float 0.1–600 | 5 | Timeline |
| `alvs.timeline.defaultTransitionSec` | float 0.1–10 | 1.0 | Timeline |
| `alvs.timeline.defaultEasing` | EAS easing ID | `linear` | Timeline |
| `alvs.captions.maxCharsPerLine` | int 20–80 | 42 | Timeline |
| `alvs.captions.maxLines` | int 1–3 | 2 | Timeline |
| `alvs.captions.minDurationSec` | float 0.3–5 | 0.7 | Timeline |
| `alvs.captions.maxDurationSec` | float 2–20 | 7 | Timeline |
| `alvs.captions.minGapFrames` | int 0–10 | 2 | Timeline |
| `alvs.media.proxyAuto` | bool | true | Media & Cache |
| `alvs.media.proxyThresholdHeight` | 1080 \| 1440 \| 2160 | 1440 | Media & Cache |
| `alvs.media.proxyResolution` | 540 \| 720 \| 1080 | 720 | Media & Cache |
| `alvs.media.proxyLocation` | `cache` \| `projectFolder` | `cache` | Media & Cache |
| `alvs.media.deinterlaceDefault` | `auto` \| `off` \| `force` | `auto` | Media & Cache |
| `alvs.media.cacheFolder` | path \| `default` | `default` | Media & Cache |
| `alvs.media.cacheMaxGB` | int 2–1000 | 20 | Media & Cache |
| `alvs.export.defaultFolder` | path \| `projectExports` | `projectExports` | Export |
| `alvs.export.useHardwareEncoder` | bool | true | Export |
| `alvs.export.lastPresetId` | XP preset ID | `xp.h264.high` | Export |
| `alvs.export.openFolderWhenDone` | bool | false | Export |
| `alvs.export.notifyWhenDone` | bool | true | Export |
| `alvs.keyboard.overrides` | object `{ commandId: [shortcut, …] }` | `{}` | Keyboard |
| `alvs.ai.sttModel` | MDL file stem (for example `small-q5_1`) | `small-q5_1` | AI |
| `alvs.ai.sttLanguage` | `auto` or BCP 47 tag | `auto` | AI |
| `alvs.ai.sttUseGpu` | bool | true | AI |
| `alvs.ai.modelsFolder` | path \| `default` | `default` | AI |
| `alvs.advanced.gpuAdapter` | `auto` \| adapter LUID string | `auto` | Advanced |
| `alvs.advanced.forceWarp` | bool | false | Advanced |
| `alvs.advanced.maxParallelJobs` | int 1–8 | 2 | Advanced |
| `alvs.advanced.logLevel` | `info` \| `debug` \| `trace` | `info` | Advanced |
| `alvs.advanced.redactDiagnostics` | bool | true | Advanced |
| `alvs.workspaces.saved` | array of `{ id, name, layout }` (dockview JSON, version 1) | `[]` | (View menu) |
| `alvs.workspaces.active` | `WS-01`..`WS-04` or a saved ID | `WS-01` | (View menu) |
| `alvs.ui.firstRunDone` | bool | false | (internal) |
| `alvs.ui.tourDone` | bool | false | (internal) |

Write policy: debounce 500 ms; atomic write (3.9.1); keep `settings.json.bak` (last good file) for recovery (REQ-SET-003).

---

## 3.7 Cache database and cache files

Database file `cache.sqlite` (3.8) opened with `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`. Migrations live in `crates/alvs-store/migrations/NNN_<name>.sql` and run in a transaction; the version is stored in `meta`.

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);

CREATE TABLE media_cache (
  media_key   TEXT PRIMARY KEY,           -- SHA-256 of the canonical fingerprint JSON
  path        TEXT NOT NULL,
  probe_json  TEXT NOT NULL,
  probed_utc  TEXT NOT NULL
);

CREATE TABLE thumbnails (
  media_key     TEXT NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('poster', 'filmstrip')),
  height        INTEGER NOT NULL,
  interval_flicks INTEGER NOT NULL,       -- 0 for poster
  file          TEXT NOT NULL,            -- relative to the cache folder
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, kind, height)
);

CREATE TABLE waveforms (
  media_key     TEXT NOT NULL,
  stream        INTEGER NOT NULL,
  file          TEXT NOT NULL,
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, stream)
);

CREATE TABLE proxies (
  media_key     TEXT PRIMARY KEY,
  profile       TEXT NOT NULL,            -- e.g. 'h264-intra-720'
  file          TEXT,
  state         TEXT NOT NULL CHECK (state IN ('queued', 'building', 'ready', 'failed')),
  error_code    TEXT,
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL
);

CREATE TABLE jobs (
  id           TEXT PRIMARY KEY,
  kind         TEXT NOT NULL,
  state        TEXT NOT NULL CHECK (state IN ('queued','running','paused','succeeded','failed','cancelled')),
  priority     INTEGER NOT NULL,
  project_id   TEXT,
  spec_json    TEXT NOT NULL,
  progress     REAL NOT NULL DEFAULT 0,
  attempts     INTEGER NOT NULL DEFAULT 0,
  error_code   TEXT,
  error_detail TEXT,
  output_json  TEXT,
  log_file     TEXT,
  created_utc  TEXT NOT NULL,
  started_utc  TEXT,
  finished_utc TEXT
);
CREATE INDEX jobs_by_state ON jobs (state, priority DESC, created_utc);

CREATE TABLE recent_projects (
  path          TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  thumbnail_file TEXT,
  opened_utc    TEXT NOT NULL,
  pinned        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE models (
  id            TEXT PRIMARY KEY,           -- e.g. 'small-q5_1'
  file          TEXT NOT NULL,
  sha256        TEXT NOT NULL,
  size          INTEGER NOT NULL,
  state         TEXT NOT NULL CHECK (state IN ('downloading', 'installed', 'corrupt')),
  installed_utc TEXT
);

CREATE TABLE encoder_probe (
  driver_key  TEXT PRIMARY KEY,             -- adapter vendor/device + driver version
  result_json TEXT NOT NULL,
  probed_utc  TEXT NOT NULL
);

CREATE TABLE analysis (
  media_key   TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('loudness', 'stabilize', 'transcript')),
  params_hash TEXT NOT NULL,
  file        TEXT NOT NULL,
  created_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, kind, params_hash)
);
```

Cache file layout under the cache folder (`k` = `media_key`, `k2` = its first two hex characters):

| Data | Path | Format |
|---|---|---|
| Poster thumbnail | `thumbs/k2/k/poster-<h>.webp` | WebP |
| Filmstrip | `thumbs/k2/k/filmstrip-<h>.webp` + `filmstrip-<h>.json` | Sprite sheet + index (frame times) |
| Waveform | `waves/k2/k/<stream>.alvswf` | Binary (below) |
| Proxy | `proxies/k2/k/<profile>.mp4` | H.264 all-intra, video only (02 §2.8) |
| Analysis | `analysis/k2/k/<kind>-<params_hash>.json` | JSON |

Waveform file (little-endian): magic `ALVSWF1\0` (8 bytes), `u16 version = 1`, `u32 sampleRate`, `u16 channels`, `u16 levelCount`; then for each level `u32 samplesPerPeak` (256, 1024, 4096, 16384) and `u64 peakCount`, followed by `peakCount × channels` pairs of `i16 min, i16 max`.

Eviction: when total cache size exceeds `alvs.media.cacheMaxGB`, delete least-recently-used entries in this order: analysis, filmstrips, waveforms, posters, proxies (never proxies of media in the open project). A corrupt database is renamed `cache.sqlite.corrupt-<timestamp>` and rebuilt (E8003).

---

## 3.8 File locations

Tauri resolves base folders from the identifier `com.afterleads.videospirits`. In portable mode (a file `portable.flag` next to the executable) every app folder below is replaced by `<exe folder>\data\…`, and the default project folder (`alvs.general.defaultProjectFolder`) becomes `<exe folder>\data\Projects` — the installed default `%USERPROFILE%\Videos\AfterLeads-Video-Spirits` applies otherwise (OD-12).

| Data | Location |
|---|---|
| Settings | `%APPDATA%\com.afterleads.videospirits\settings.json` (+ `settings.json.bak`) |
| User library | `%APPDATA%\com.afterleads.videospirits\library\{effect-presets,export-presets,title-templates,keymaps}\` |
| Cache database | `%LOCALAPPDATA%\com.afterleads.videospirits\cache\cache.sqlite` |
| Cache files | `%LOCALAPPDATA%\com.afterleads.videospirits\cache\` (or `alvs.media.cacheFolder`) |
| Speech models | `%LOCALAPPDATA%\com.afterleads.videospirits\models\` (or `alvs.ai.modelsFolder`) |
| Logs | `%LOCALAPPDATA%\com.afterleads.videospirits\logs\` |
| Crash dumps and panic reports | `%LOCALAPPDATA%\com.afterleads.videospirits\crashes\` |
| Session marker | `%LOCALAPPDATA%\com.afterleads.videospirits\session.marker` |
| Previous installer (rollback) | `%LOCALAPPDATA%\com.afterleads.videospirits\updates\` |
| Temporary | `%TEMP%\AfterLeads-Video-Spirits\<pid>\` |
| Project folder | User-chosen; default under `alvs.general.defaultProjectFolder` (`%USERPROFILE%\Videos\AfterLeads-Video-Spirits`; `<exe folder>\data\Projects` in portable mode, OD-12) |

Project folder layout:

```text
<Project Folder>\
├─ <Name>.alvs                 project
├─ <Name>.alvs.autosave        autosave sidecar (exists only while dirty)
├─ <Name>.alvs.lock            lock file while open
├─ Backups\<Name>_<yyyyMMdd-HHmmss>.alvs
├─ assets\luts\<sha256>.cube
├─ Recordings\                 voice-over WAV files
├─ Derived\Optimized\  Derived\Stabilized\
├─ Proxies\                    only when alvs.media.proxyLocation = projectFolder
└─ Exports\                    default export folder
```

All paths support Unicode and long paths (`\\?\` prefix applied by `alvs-core::path` when needed).

---

## 3.9 Lifecycle

### 3.9.1 Atomic write (all user and app files)

Write to `<target>.tmp` in the same folder → flush (`FlushFileBuffers`) → replace with `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)` → on error keep the original and report (E1008/E8001/E8004).

### 3.9.2 Save

Serialize the current snapshot on a background thread → atomic write → write a backup copy into `Backups\` → prune to `alvs.general.backupsKeep` → delete the autosave sidecar → update `recent_projects`.

### 3.9.3 Autosave

When the project is dirty and `alvs.general.autosaveIntervalSec` has elapsed since the last autosave, serialize the snapshot and atomically write `<Name>.alvs.autosave`. Autosave never touches the main file (REQ-PRJ-003).

### 3.9.4 Lock

On open, create `<Name>.alvs.lock` exclusively with `{ pid, host, user, startedUtc, appVersion }`. If it exists: same host and the process is alive → offer read-only (E1007); dead process or a different host older than 24 h → offer take-over. Read-only sessions disable save and show a banner. The lock is removed on close.

### 3.9.5 Crash recovery

A session marker is created at startup and removed on clean exit. If the marker exists at startup and the last open project has an autosave newer than its main file, show DLG-15 (REQ-REL-001). Choosing the autosave opens it as an unsaved document whose next save writes the main file.

### 3.9.6 Migrations

- `schemaVersion` < current: copy the original to `<Name>.alvs.v<old>.bak`, run migrations `v(n) → v(n+1)` on the JSON value, validate, and open (saved on the next save).
- `schemaVersion` > current: open read-only with E1004; unknown fields are kept in memory.
- Every migration has fixtures (input/output pairs) and a test (T-PRJ-007).

### 3.9.7 Proxies, derived media, and models

- Proxies: created by jobs, registered in `proxies`; used when `settings.proxyMode` is on and state is `ready`; never used for export.
- Derived media (optimized/stabilized) live in the project folder and can be deactivated (the clip falls back to the original) or regenerated.
- Models: download to `<file>.part` with HTTP range resume → SHA-256 verify → rename → register in `models`; mismatch deletes the file (E7003).

### 3.9.8 Logs and crash data

Logs roll at 10 MB, keeping 10 files. Crash dumps keep the newest 10. Diagnostics bundles are created only on request and saved where the user chooses.

---

## 3.10 User library formats

| Item | File | Schema (`format`, `schemaVersion`) |
|---|---|---|
| Effect preset | `library/effect-presets/<name>.json` | `afterleads-video-spirits.effectPreset`, 1 — `{ effectId, effectVersion, params }` |
| Export preset | `library/export-presets/<name>.json` | `afterleads-video-spirits.exportPreset`, 1 — the export settings of 04 §4.6.3 without paths |
| Title template | `library/title-templates/<name>.json` | `afterleads-video-spirits.titleTemplate`, 1 — `{ name, title }` (3.4.9) |
| Keymap | `library/keymaps/<name>.json` | `afterleads-video-spirits.keymap`, 1 — `{ bindings: { commandId: [shortcut] } }` |

Imported library files are validated like project files (size ≤ 1 MB, schema check); invalid files are rejected with E1003.

---

## 3.11 Privacy and retention

- No personal data is collected. Logs avoid content (02 §2.11). Diagnostics bundles redact user names and paths when `alvs.advanced.redactDiagnostics` is on.
- "Clear cache" (REQ-SET-004) removes cache files and database rows; projects and the user library are never touched.
- Uninstall removes the program files only; settings, library, cache, and models remain unless the user ticks "Remove app data" in the uninstaller (NFR-INS-01).
