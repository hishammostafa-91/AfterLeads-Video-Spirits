# 09 — Quality

Canonical for: test strategy, reference machines, test data, test methods, the test catalog (T), release gates (GATE), waivers, bug severity, performance and usability test procedures.
Depends on: [01-product.md](01-product.md) (REQ/NFR), [08-operations.md](08-operations.md) (CI), [10-execution.md](10-execution.md) (work packages).

---

## 9.1 Strategy

| Level | Share | Tools | Runs |
|---|---|---|---|
| Unit and property | ~60% | `cargo nextest`, `proptest`, `insta`; Vitest | Every PR |
| Integration (engine, jobs, store, workers) | ~20% | Rust integration tests, CLI | Every PR (smoke) / nightly (full) |
| Golden media (images, audio) | ~8% | `alvs-testkit` on WARP | Every PR (smoke) / nightly (full) / release (hardware) |
| UI harness (components, panels, dialogs with mock backend) | ~8% | Playwright, axe-core | Every PR |
| End-to-end (real app) | ~3% | WebdriverIO + tauri-driver | PR smoke / nightly full |
| Manual (usability, screen readers, hardware) | ~1% | Checklists | Release candidates |

Principles: tests are deterministic (seeded, no network, no wall-clock dependence); every REQ has at least one test (VER-03); tests are never skipped to get green (G-08); a fixed bug gets a regression test.

## 9.2 Reference machines

| ID | Hardware | Use |
|---|---|---|
| R-LOW | 4-core/8-thread x86-64 CPU with AVX2, 8 GB RAM, integrated GPU (Intel UHD 620 class) with HEVC hardware decode, SSD, 1920×1080 at 100% | Minimum-spec gates (NFR-PERF-03, NFR-COMP-01) |
| R-MID | 8-core CPU (Ryzen 7 5700X / Core i7-11700 class), 16 GB RAM, discrete GPU with 8 GB VRAM and hardware HEVC/AV1 decode (GeForce RTX 3060 / Radeon RX 6600 / Arc A750 class), NVMe SSD, 2560×1440 at 150% | Most performance gates |
| R-HIGH (optional) | 12+ cores, 32 GB RAM, GPU with AV1 hardware encode, 3840×2160 at 200% | High-DPI and 4K checks |
| CI | GitHub `windows-2025` runner, no GPU (WARP) | Automated suites |

Release gates need R-MID plus either R-LOW or a second GPU vendor (GATE-03).

## 9.3 Test data

- **Generated media** (never committed): `tools/media/generate_test_media.ps1` uses the vcpkg-built `ffmpeg` tool to create a deterministic corpus in `target/test-media/` — color bars and moving test patterns in H.264, HEVC 8/10-bit, AV1, VP9, ProRes 422/4444 (alpha), DNxHR, FFV1, MPEG-2, MJPEG; 23.976/25/29.97/50/59.94/60 fps; variable frame rate (timestamp jitter); interlaced; rotation metadata 90/180/270; HDR-tagged PQ and HLG gradients; PNG/JPEG/WebP/TIFF/EXR stills and a numbered PNG sequence; audio sines/sweeps/noise at 44.1/48 kHz in mono, stereo, 5.1; an A/V sync file (white flash + 1 kHz beep each second). The script prints and stores SHA-256 of each output; CI caches the folder keyed by the script hash.
- **Committed fixtures** (≤ 1 MB each, license recorded in `tests/fixtures/SOURCES.md`): caption files (valid, malformed, encodings including Windows-1256 Arabic), `.cube` LUTs (valid/invalid), project files for migrations and validation, CSV markers, speech clips (CC0/public domain, 6.5).
- **Malicious corpus** (T-SEC-003): JSON bombs, deeply nested JSON, path traversal and device paths, UNC paths, HLS/concat playlists disguised as `.mp4`, oversized image headers, truncated media.

## 9.4 Unit and property tests (per crate)

| Crate | Must cover |
|---|---|
| `alvs-core` | Flicks/timecode conversions for every FPS entry (drop-frame included), rational reduction, path policy (SEC-04) |
| `alvs-model` | Every `EditOp` (happy path + rejection), invariants (03 §3.5) after random edit sequences (proptest: 10,000 sequences of 50 ops in CI, 1,000,000 nightly), undo/redo round-trip equality, entity patch correctness, serialization round-trip, migrations, schema generation |
| `alvs-formats` | Each parser/writer round-trip and error positions; encoding detection |
| `alvs-media` | Probe fields on the generated corpus; seek accuracy (frame-exact by PTS); VFR mapping; HW/SW decode equivalence (frame checksum tolerance); protocol/demuxer restrictions (SEC-02) |
| `alvs-render` | Shader parameter packing layouts; render graph construction; each effect, blend mode, transition, mask, and color conversion (golden) |
| `alvs-audio` | DSP measurements (9.7), mixer solo/mute rules, automation ramps |
| `alvs-engine` | Timeline evaluation at edit points and transitions; frame cache invalidation by dirty ranges; export pipeline frame counts |
| `alvs-store`, `alvs-jobs`, `alvs-stt` | Settings validation/recovery, migrations, eviction, lock files; job state machine, retries, supervision; manifest verification, segmentation |

## 9.5 Rendering correctness

Rendering tests use the CLI `frame` mode or library calls with `force_fallback_adapter` (WARP) so results are deterministic on CI.

## 9.6 Golden image tests

1. Each golden case = project fixture + time + expected PNG in `tests/golden/<case>.png` (8-bit RGBA, generated on WARP).
2. Pass if max absolute channel difference ≤ 2/255 and PSNR ≥ 50 dB against the baseline; on hardware GPUs (release gate) pass if PSNR ≥ 42 dB and SSIM ≥ 0.995.
3. Baselines change only with `UPDATE_GOLDEN=1 cargo nextest run -p alvs-render golden` in a PR that shows before/after images and explains the cause; the owner approves.
4. Matrix: every VFX at defaults and one documented non-default; every transition at p = 0.25/0.5/0.75; every blend mode on the standard source/backdrop pair; YUV matrices (601/709/2020), ranges, 10-bit; HLG and PQ tone mapping; transform/crop/opacity; masks with feather; titles (en, ar, mixed bidi); caption burn-in (en, ar).

## 9.7 Audio tests

| Test | Pass criterion |
|---|---|
| Null test (effects bypassed, unity gain) | Output bit-identical, or residual ≤ −120 dBFS when resampling |
| Loudness (BS.1770) | A generated stereo 1 kHz sine at −23 dBFS peak in both channels (20 s) measures −23.0 LUFS ±0.1 integrated; the same at −33 dBFS measures −33.0 ±0.1 |
| True peak | Inter-sample peak test signal measured within ±0.2 dB of the analytic value |
| EQ | Measured gain at band centers within ±0.5 dB of settings |
| Compressor / limiter | Static curve within ±0.5 dB; limiter output never exceeds the ceiling (4× oversampled check) |
| A/V sync | Exported sync file: flash and beep aligned within ±1 ms after export and within ±10 ms during playback measurement (REQ-PLY-003) |
| Denoise | SNR improvement ≥ 6 dB on speech fixture + white noise at 10 dB SNR |

## 9.8 UI harness tests

- The harness (`pnpm -C app harness`) runs the real UI with the mock backend (`app/src/mocks/`), which implements every command/event in 04 with in-memory state and canned frames.
- Visual snapshots: every panel and dialog in Dark and Light, `en` and `ar`; key screens in High Contrast and at 200% zoom. Baselines are produced on the CI Windows runner only.
- Accessibility: axe-core with zero serious/critical violations per screen; keyboard scripts for each dialog (open, traverse, close) and the timeline keyboard model (7.9.3).
- Localization: pseudo-locale LOC-90 overflow detection (any text element with `scrollWidth > clientWidth` and no tooltip fails) and LOC-91 RTL rendering.

## 9.9 End-to-end tests

- **T-E2E-001 Smoke:** launch → first-run → new project → import generated clips → split, trim, add transition and title → export XP-02 (2 s) → output verified.
- **T-E2E-002 Crash recovery:** edit → wait for autosave → kill the process → relaunch → recover → contents equal.
- **T-E2E-003 Captions:** import SRT (Arabic) → edit → burn-in export → verify the burned frame against its golden image.
- **T-E2E-004 Transcription:** MDL-01 on the English speech fixture → a new caption track exists.
- **T-E2E-005 Language:** switch to Arabic → UI mirrored → restart → persisted; shortcuts work with an Arabic keyboard layout.
- **T-E2E-006 Jobs:** create proxies → cancel one → retry → toggle proxy mode → playback uses proxies, export uses originals.

## 9.10 Test catalog

Columns: ID, level (U unit, P property, I integration, G golden, H UI harness, E end-to-end, M manual, R reference-machine measurement), what is verified, requirements covered.

| ID | Level | Verifies | Covers |
|---|---|---|---|
| T-APP-001 | H, E | Window chrome, menus match 12 §12.11, toolbar, status bar fields, min size | REQ-APP-001, REQ-APP-007 |
| T-APP-002 | H | Docking operations, workspace switching, reset, save/rename/delete workspace, persistence | REQ-APP-002, REQ-APP-003 |
| T-APP-003 | U, H | Command registry completeness vs 12 §12.10, palette fuzzy search, enablement | REQ-APP-004 |
| T-APP-004 | H | Welcome screen states (no recents, recents, missing project) | REQ-APP-005 |
| T-APP-005 | H | Toasts, notification center, error envelope rendering with recovery | REQ-APP-006 |
| T-APP-006 | E | Second launch forwards path; dirty prompt | REQ-APP-008 |
| T-APP-007 | H, E | Unsaved changes prompt on close/exit/open | REQ-APP-012 |
| T-UI-001 | H | Themes (dark/light/HC), accent switching without restart | REQ-APP-009 |
| T-UI-002 | H, M | Zoom 80–200% and DPI 100–200% rendering crisp, no clipping | REQ-APP-010 |
| T-PRJ-001 | U, H | New project request, presets, defaults, folder creation | REQ-PRJ-001 |
| T-PRJ-002 | I | Save/open round-trip, relative path fallback, atomic write under simulated failure | REQ-PRJ-002 |
| T-PRJ-003 | I | Autosave timing and content | REQ-PRJ-003, NFR-REL-02 |
| T-PRJ-004 | I | Backup creation, pruning, restore | REQ-PRJ-004 |
| T-PRJ-005 | U, I | Sequence create/duplicate/delete; frame-rate change snapping + E1104 | REQ-PRJ-005 |
| T-PRJ-006 | P, H | Undo/redo equality, 200-step guarantee, memory budget, History jump | REQ-PRJ-007 |
| T-PRJ-007 | U | Migrations, newer-version read-only, malicious limits | REQ-PRJ-008 |
| T-PRJ-008 | I | Lock file creation, read-only, stale takeover | REQ-PRJ-011 |
| T-MED-001 | I | Import of the guaranteed formats (12 §12.20), drag-drop, still duration, large images | REQ-MED-001 |
| T-MED-002 | I | Probe isolation and metadata fields | REQ-MED-002 |
| T-MED-003 | H | Media panel views, sort, search, folders, labels, remove | REQ-MED-003 |
| T-MED-004 | I, H | Thumbnails/filmstrip/waveform generation and display | REQ-MED-004 |
| T-MED-005 | I, H | Offline detection and relink search | REQ-PRJ-010 |
| T-MED-006 | I, E | Proxy creation, use, deletion, never in export | REQ-MED-005 |
| T-MED-007 | R, I | Hardware decode on supported GPUs; fallback | REQ-MED-006 |
| T-MED-008 | I | VFR playback/export frame selection by PTS | REQ-MED-007 |
| T-MED-009 | G | Rotation, HDR tone mapping, 10-bit precision | REQ-MED-008 |
| T-MED-010 | I | Image sequence detection and frame rate | REQ-MED-009 |
| T-MED-011 | G | Deinterlace auto/off/force | REQ-MED-010 |
| T-MED-012 | I | Optimize media job and relink | REQ-MED-011 |
| T-MED-013 | I | Basic stabilize job and toggle | REQ-MED-013 |
| T-PLY-001 | E, R | Program/Source viewers, dual/single layout, preview transport path | REQ-PLY-001 |
| T-PLY-002 | E | Transport commands and shuttle speeds | REQ-PLY-002 |
| T-PLY-003 | R | A/V sync over 10 minutes | REQ-PLY-003 |
| T-PLY-004 | R, H | Preview resolutions, dropped-frame counter, play every frame | REQ-PLY-004 |
| T-PLY-005 | U, H | Timecode display/entry incl. drop-frame | REQ-PLY-005 |
| T-PLY-006 | H | Overlays and zoom/pan | REQ-PLY-006 |
| T-PLY-007 | R | Scrub latency, audio scrub | REQ-PLY-007 |
| T-PLY-008 | R | Full screen and external window | REQ-PLY-008 |
| T-EDT-001 | P, H | Track operations | REQ-EDT-001 |
| T-EDT-002 | P, H | Insert/overwrite/append/replace/lift/extract/delete/ripple/gaps, split, blade | REQ-EDT-002, REQ-EDT-003 |
| T-EDT-003 | P, H | Ripple/roll/slip/slide/extend/Q/W/numeric trims with handle limits | REQ-EDT-004 |
| T-EDT-004 | H | Snapping targets and indicator | REQ-EDT-005 |
| T-EDT-005 | H | Selection modes, linked selection, link/group | REQ-EDT-006, REQ-EDT-007 |
| T-EDT-006 | P, I | Nesting, cycle rejection | REQ-PRJ-006 |
| T-EDT-007 | P, H | Copy/paste/duplicate/paste attributes | REQ-EDT-008 |
| T-EDT-008 | G, I | Speed/reverse/freeze correctness (video frames and audio) | REQ-EDT-009 |
| T-EDT-009 | P, H | Markers create/edit/navigate, Markers panel | REQ-EDT-010 |
| T-EDT-010 | H, R | Zoom/scroll/follow, clip display (thumbnails, waveforms, badges, gain line) | REQ-EDT-011, REQ-EDT-012, REQ-AUD-007 |
| T-EDT-011 | P | Invariants hold after random edits | REQ-EDT-014 |
| T-EDT-012 | E | Source → timeline insert/overwrite, video-only/audio-only drags | REQ-EDT-016, REQ-EDT-017 |
| T-FX-001 | G, H | Intrinsic transform/crop/opacity/blend | REQ-FX-001 |
| T-FX-002 | H | Effects panel, apply, stack operations, presets | REQ-FX-002, REQ-FX-003 |
| T-FX-003 | H, E | Generated parameter UI controls, on-viewer handles, eyedropper | REQ-FX-012, REQ-PLY-009 |
| T-FX-004 | G, U | Every 12 §12.1 effect: manifest equals catalog, golden output | REQ-FX-004 |
| T-FX-005 | U, H | Keyframes: all easings, add/remove/navigate, graph editor | REQ-FX-005, REQ-FX-010 |
| T-FX-006 | G, H | Masks shapes, feather, modes, effect masks, animation | REQ-FX-006 |
| T-FX-007 | G | Chroma/luma key and spill | REQ-FX-007 |
| T-FX-008 | U, G | `.cube` parsing/errors, LUT apply, copying into project | REQ-FX-008, REQ-INT-003 |
| T-FX-009 | G | Adjustment layers | REQ-FX-009 |
| T-RND-001 | G | WARP vs hardware equivalence within tolerance | REQ-FX-011 |
| T-RND-002 | G | Color pipeline cases | REQ-COL-003 |
| T-RND-003 | G | All 24 blend modes | REQ-FX-001 |
| T-TRN-001 | G, P | Video transitions, alignment, handle shortening | REQ-TRN-001 |
| T-TRN-002 | U | Audio crossfade curves | REQ-TRN-002 |
| T-TRN-003 | H, P | Default transition commands, parameter editing, duration trimming | REQ-TRN-003, REQ-TRN-004 |
| T-TTL-001 | G, H | Title layout features, Arabic shaping, bidi, on-viewer editing | REQ-TTL-001, REQ-TTL-006 |
| T-TTL-002 | G | Title animations | REQ-TTL-002 |
| T-TTL-003 | H | Templates insert and user templates | REQ-TTL-003, REQ-TTL-005 |
| T-TTL-004 | G | Generators | REQ-TTL-004 |
| T-AUD-001 | U, I | Gain, volume automation, pan, fades, track controls | REQ-AUD-001 |
| T-AUD-002 | H | Mixer panel | REQ-AUD-002 |
| T-AUD-003 | U | Each AFX per 9.7 | REQ-AUD-003 |
| T-AUD-004 | U, I | Loudness measurement and normalization | REQ-AUD-004 |
| T-AUD-005 | I | Auto-ducking keyframes | REQ-AUD-005 |
| T-AUD-006 | R, E | Voice-over recording, countdown, punch-in, latency compensation | REQ-AUD-006 |
| T-AUD-007 | U | Channel modes and 5.1 downmix coefficients | REQ-AUD-008 |
| T-AUD-008 | U | Pitch-preserving speed | REQ-AUD-009 |
| T-AUD-009 | R | Device selection, buffer presets, device removal recovery | REQ-AUD-010 |
| T-COL-001 | H | Color workspace controls | REQ-COL-001 |
| T-COL-002 | G | HDR to SDR and gamut conversion | REQ-COL-003, REQ-MED-008 |
| T-COL-003 | G, R | Scopes correctness on test patterns and update rate | REQ-COL-002 |
| T-COL-004 | H | Before/after split, copy/paste grade | REQ-COL-004, REQ-COL-005 |
| T-CAP-001 | P | Caption tracks model | REQ-CAP-001 |
| T-CAP-002 | H | Captions panel editing | REQ-CAP-002 |
| T-CAP-003 | U | SRT/VTT/ASS import/export, encodings | REQ-CAP-003 |
| T-CAP-004 | G, H | Caption styling | REQ-CAP-004 |
| T-CAP-005 | E, I | Burn-in, sidecar, embedded streams | REQ-CAP-005 |
| T-CAP-006 | G | RTL and mixed captions | REQ-CAP-006 |
| T-CAP-007 | U | Readability rules and warnings | REQ-CAP-007 |
| T-STT-001 | I, R | Transcription accuracy gates (6.5) | REQ-STT-001 |
| T-STT-002 | I | Model manager download/resume/verify/delete with a local test server | REQ-STT-002 |
| T-STT-003 | I | Background job, cancel, GPU fallback | REQ-STT-003 |
| T-STT-004 | U | Segmentation rules (6.3), new track | REQ-STT-004 |
| T-STT-005 | H | Transcript panel | REQ-STT-005 |
| T-EXP-001 | I, H | Export dialog settings and all presets XP-01..XP-20 produce valid files | REQ-EXP-001, REQ-EXP-005 |
| T-EXP-002 | R | Hardware encoder detection, quality mapping, fallback | REQ-EXP-002 |
| T-EXP-003 | I, H | Render queue operations | REQ-EXP-003 |
| T-EXP-004 | I | Output verification, originals not proxies, preview/export equivalence | REQ-EXP-008, REQ-EXP-011, REQ-FX-011 |
| T-EXP-005 | I | Export frame | REQ-EXP-004 |
| T-EXP-006 | I | Alpha export, GIF, image sequences | REQ-EXP-006, REQ-EXP-012 |
| T-EXP-007 | U, I | Preflight checks | REQ-EXP-007 |
| T-EXP-008 | I | Chapters embedded | REQ-EXP-009 |
| T-EXP-009 | H | User presets | REQ-EXP-010 |
| T-JOB-001 | I | Jobs run in workers | REQ-JOB-001 |
| T-JOB-002 | H | Jobs panel | REQ-JOB-002 |
| T-JOB-003 | I | Worker crash/hang handling and re-queue after restart | REQ-JOB-003 |
| T-JOB-004 | I | Atomic outputs and cleanup | REQ-JOB-004 |
| T-SET-001 | H | Settings dialog pages, search, reset | REQ-SET-001 |
| T-SET-002 | U, H | Shortcut editor and zero default conflicts | REQ-SET-002 |
| T-SET-003 | U | Settings validation, unknown keys, corruption recovery | REQ-SET-003 |
| T-SET-004 | I, H | Cache management and clearing | REQ-SET-004 |
| T-I18N-001 | H, E | Six locales, runtime switch, default selection | REQ-I18N-001 |
| T-I18N-002 | H, G | RTL mirroring rules (7.11), fonts | REQ-I18N-002, REQ-I18N-007 |
| T-I18N-003 | U | ICU plural/select, number/date formatting, Western digits for time | REQ-I18N-003 |
| T-I18N-004 | U | Identical keys across locales; pseudo-locale overflow | REQ-I18N-004 |
| T-I18N-005 | U, E | Physical-key shortcuts on AZERTY, QWERTZ, Arabic layouts | REQ-I18N-005 |
| T-A11Y-001 | H, M | Keyboard-only operation, F6, maximize panel | REQ-A11Y-001, REQ-APP-011 |
| T-A11Y-002 | H, M | Timeline keyboard model and list view announcements | REQ-EDT-013, REQ-EDT-015 |
| T-A11Y-003 | M | Narrator and NVDA walkthrough of UC-01 and UC-09 | REQ-A11Y-002 |
| T-A11Y-004 | H | Contrast checks, forced colors, reduced motion | REQ-A11Y-003, REQ-A11Y-004 |
| T-HLP-001 | H | First-run setup and tour | REQ-HLP-001 |
| T-HLP-002 | H | Help menu items, in-app guide per locale, report URL | REQ-HLP-002, REQ-I18N-006 |
| T-HLP-003 | H | Tooltips with shortcuts, empty states | REQ-HLP-003 |
| T-LIC-001 | H, I | About dialog, license text, notices completeness, trademark notice | REQ-HLP-004, REQ-LIC-001, REQ-LIC-003 |
| T-LIC-002 | I | Release assets include source and third-party sources | REQ-LIC-002 |
| T-UPD-001 | I, E | Update check, signature verification, tamper rejection | REQ-UPD-001 |
| T-UPD-002 | E | No install during export/unsaved; rollback | REQ-UPD-002 |
| T-UPD-003 | E | Portable mode | REQ-UPD-003 |
| T-UPD-004 | M | Clean install, upgrade, uninstall with and without data | NFR-INS-01 |
| T-REL-001 | E | Crash recovery flow | REQ-REL-001, REQ-PRJ-003, NFR-REL-02 |
| T-REL-002 | E | Safe mode | REQ-REL-002 |
| T-REL-003 | U | Log rotation and redaction | REQ-REL-003 |
| T-REL-004 | I | Diagnostics bundle contents and redaction | REQ-REL-004 |
| T-REL-005 | I | Panic report and minidump creation for GUI and worker | REQ-REL-005 |
| T-REL-006 | I | GPU device loss simulation and WARP switch | REQ-REL-006 |
| T-REL-007 | I | Disk-full and permission errors on save/export/cache | REQ-REL-007 |
| T-REL-008 | M | Beta crash-free sessions | NFR-REL-01 |
| T-REL-009 | R | 8-hour soak | NFR-REL-03 |
| T-INT-001 | U | EDL export format | REQ-INT-001 |
| T-INT-002 | U | Chapter list and CSV export/import | REQ-INT-002 |
| T-INT-003 | I | Collect files | REQ-PRJ-009 |
| T-SEC-001 | I | Dependency advisories and license policy | NFR-SEC-01 |
| T-SEC-002 | I | Fuzzing (SEC-05) | THR-03 |
| T-SEC-003 | I | Malicious corpus (9.3) handled safely | THR-01, THR-02, THR-12, THR-13 |
| T-SEC-004 | I, E | No network except allowed cases; transcription offline | NFR-PRIV-01, REQ-STT-006 |
| T-SEC-005 | I, H | CSP, capabilities, navigation lock, DLL search order, binary flags | THR-04, THR-09 |
| T-SEC-006 | I | Pre-commit hook rejects private keys and `.env` files; repository secret scanning and push protection enabled (settings audit) | THR-10 |
| T-SEC-007 | I | Workflow audit: no `pull_request_target` with untrusted checkout, actions pinned to SHAs, default token read-only, DCO and owner review required | THR-11 |
| T-PERF-001 | R | Startup times | NFR-PERF-01 |
| T-PERF-002 | R | Large project open | NFR-PERF-02 |
| T-PERF-003 | R | 1080p playback on R-LOW | NFR-PERF-03 |
| T-PERF-004 | R | 4K multi-layer playback on R-MID | NFR-PERF-04 |
| T-PERF-005 | R, H | Timeline 1,000 clips frame rate and latency | NFR-PERF-05 |
| T-PERF-006 | R | Scrub latency | NFR-PERF-06, REQ-PLY-007 |
| T-PERF-007 | R | Export speed | NFR-PERF-07 |
| T-PERF-008 | R | Import and thumbnail timing | NFR-PERF-08 |
| T-PERF-009 | R | Transcription speed | NFR-PERF-09 |
| T-PERF-010 | R | Memory | NFR-RES-01 |
| T-PERF-011 | R | Idle usage | NFR-RES-02 |
| T-PERF-012 | I | Installer and install size | NFR-RES-03 |
| T-UX-001 | M | Usability protocol 9.14 | NFR-UX-01 |
| T-QA-001 | I | Coverage thresholds | NFR-MNT-01 |
| T-QA-002 | I | PR CI duration | NFR-MNT-02 |
| T-QA-003 | M | Platform matrix (Windows 11 23H2/24H2/25H2, NVIDIA/AMD/Intel, 8 GB) | NFR-COMP-01 |
| T-E2E-001 | E | 9.9 | REQ-APP-001, REQ-EXP-001 |
| T-E2E-002 | E | 9.9 | REQ-REL-001 |
| T-E2E-003 | E | 9.9 | REQ-CAP-005 |
| T-E2E-004 | E | 9.9 | REQ-STT-001 |
| T-E2E-005 | E | 9.9 | REQ-I18N-001 |
| T-E2E-006 | E | 9.9 | REQ-MED-005 |

## 9.11 Release gates (GATE)

| ID | Gate | Evidence |
|---|---|---|
| GATE-01 | Every P0 requirement with release 1.0 passes its tests; every P1 passes or has a waiver | CI reports + traceability report (VER-03) |
| GATE-02 | All NFR targets met | Performance report from R-MID and R-LOW |
| GATE-03 | Golden and audio suites pass on WARP and on two GPU vendors | Test reports |
| GATE-04 | E2E suite passes on Windows 11 24H2 or newer | Test report |
| GATE-05 | Localization 100% in six locales; native-speaker review of Arabic and French; RTL visual review signed by the owner | i18n check + review notes |
| GATE-06 | Accessibility: axe clean; Narrator/NVDA and keyboard-only walkthroughs pass | Checklist |
| GATE-07 | Speech-to-text quality per 6.5 | Report |
| GATE-08 | Security: no high/critical advisories; 24 h cumulative fuzzing without new crashes; malicious corpus passes; CSP/capabilities audit | Reports |
| GATE-09 | Reliability: 8-hour soak; recovery tests; beta crash-free ≥ 99.5% (≥ 10 testers for 2 weeks) | Reports + beta summary |
| GATE-10 | Install/update: clean install, upgrade from the last beta, uninstall (keep/remove data), tampered update rejected, rollback | Checklist |
| GATE-11 | Licensing: notices, SBOM, third-party source archive, LICENSE in installer, allowlist clean | Release assets |
| GATE-12 | Docs: user guide (en) complete, release notes, VER checks green, STATUS shows WP-00..WP-29 done | Docs PR |

**Waivers:** a P1 requirement or a GATE item may be waived only by the owner in the release PR with rationale, risk, and the release that will close it. GATE-01 (P0 part), GATE-08, GATE-11, and anything risking data loss cannot be waived.

## 9.12 Bug severity

| Severity | Definition | Release impact |
|---|---|---|
| S1 Critical | Data loss/corruption, crash on a core workflow, security vulnerability, license violation | Blocks release; fix immediately |
| S2 Major | Core workflow broken with a workaround, wrong export output, major RTL/a11y failure | Blocks release unless waived |
| S3 Minor | Non-core defect, cosmetic issue with impact | Fix when convenient |
| S4 Trivial | Cosmetic, no impact | Backlog |

## 9.13 Performance test procedure

1. Use a release build on the reference machine, power plan "Balanced", plugged in, display at the listed scaling, no other heavy apps.
2. Each measurement runs 3 times after one warm-up; report the median.
3. Scripts: `tools/perf/Run-Perf.ps1` drives the CLI and the E2E harness and reads the in-app statistics (frame times, dropped frames, memory) exported through `diag_system_info`.
4. Record results in the release PR and compare with the previous release; a regression > 10% blocks the merge (G-08, 02 §2.15).

## 9.14 Usability test protocol (NFR-UX-01)

Five participants new to the application (at least two working in Arabic), each performs UC-01, UC-03, and UC-10 with the generated media pack and a speech clip; think-aloud; no help from the moderator except after 5 minutes of being stuck (counts as a failure). Success = ≥ 4 of 5 participants complete all three tasks within 20 minutes in total. Findings become issues labeled `ux`.
