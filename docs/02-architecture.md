# 02 — Architecture

Canonical for: architecture decisions (ADR), components and dependency rules (CMP), process model, render/audio/playback/export design, threading, error and logging model, repository layout, coding standards.
Depends on: [00-index.md](00-index.md) §0.8 (versions), [03-data.md](03-data.md) (data model), [04-interfaces.md](04-interfaces.md) (contracts), [05-security.md](05-security.md) (controls).

---

## 2.1 Principles

1. **One engine, two outputs.** Preview and export use the same timeline evaluator, render graph, and audio graph. Only quality settings differ (REQ-FX-011, REQ-EXP-011).
2. **Rust owns the truth.** The project model, undo history, and every rule live in Rust. The UI renders state and sends intents.
3. **Isolate the risky work.** Decoding unknown files for probing, thumbnails, proxies, transcription, and exports run in worker processes. A crash there never takes down the editor.
4. **GPU first, deterministic fallback.** All pixel processing runs as WGSL shaders on Direct3D 12. The WARP software adapter runs the same shaders when no GPU is usable and in CI.
5. **Simple, typed boundaries.** Typed Tauri commands for UI↔core, JSON Lines for core↔worker, shared memory only for preview frames.
6. **Every decision has a reason.** Each ADR below records the alternatives rejected so agents do not re-litigate them.

---

## 2.2 Architecture decision records (ADR)

| ID | Decision | Rationale | Rejected alternatives |
|---|---|---|---|
| ADR-001 | Desktop shell: **Tauri 2** with the system **WebView2** (Chromium) on Windows 11. | Small footprint (WebView2 ships with Windows 11), Rust backend, mature web platform for RTL, ICU formatting, and accessibility (ARIA → UI Automation). Highest AI-agent productivity for the UI layer. | Qt/C++ (slower agent iteration, memory-safety risk, heavy build on Windows); Electron (bundles Chromium, heavy); native Rust GUI toolkits (incomplete Arabic shaping/RTL layout and accessibility). |
| ADR-002 | Engine and app core in **Rust** (edition 2024, toolchain 1.99.1). | Memory safety for a long-running media app, excellent compiler diagnostics for agents, one build tool (cargo). | C++ (UB risk), C# (.NET interop with FFmpeg/GPU adds layers). |
| ADR-003 | GPU processing through **wgpu 30 on Direct3D 12**, shaders in **WGSL**; one implementation per effect; **WARP** fallback via `force_fallback_adapter`. | Windows 11 guarantees a D3D12-class driver; WARP executes identical shaders for fallback and deterministic CI golden tests. Avoids maintaining CPU and GPU versions of every effect. | Separate CPU reference implementations (doubles effect work); raw D3D12 (more code, less portable for later macOS/Linux). |
| ADR-004 | Media I/O through **FFmpeg 9.0.2** built by **vcpkg** (port `ffmpeg`, triplet `x64-windows-static`) with features `avcodec avdevice avfilter avformat swresample swscale gpl x264 x265 svt-av1 dav1d vpx opus mp3lame vorbis webp nvcodec amf qsv zlib bzip2 lzma ffmpeg ffprobe`; Rust bindings **ffmpeg-next 9**. Static linking into the single executable. | Broadest codec coverage; GPL components (x264, x265) are compatible with AGPL-3.0-or-later; static linking avoids DLL search-path problems; the MSVC C runtime is linked statically everywhere (Rust `+crt-static`, vcpkg `x64-windows-static`) so users never need the VC++ Redistributable; vcpkg gives a reproducible source build whose sources can be published (REQ-LIC-002). `ffmpeg`/`ffprobe` executables are built for tests only and are never shipped. | Prebuilt third-party FFmpeg binaries (harder source compliance); Media Foundation only (limited formats). |
| ADR-005 | Preview frames reach the UI through **WebView2 SharedBuffer** (`ICoreWebView2Environment12::CreateSharedBuffer`, `ICoreWebView2_17::PostSharedBufferToScript`) and are drawn by a WebGL2 presenter. Full-screen and external preview use a **native window** with a wgpu swap chain. | Zero-copy CPU handoff into the web page without IPC serialization; HTML overlays stay above the video with no window-layering problems. Verified available in Tauri 2.12 (`PlatformWebview::environment()/controller()`) with webview2-com 0.39. | Native child window over the webview (overlay and z-order bugs); JSON/base64 frames over IPC (too slow). |
| ADR-006 | **Single executable with modes**: `gui` (default), `--worker`, `cli`, `--crash-server`. | One copy of the statically linked FFmpeg; workers isolate crashes; the CLI powers tests and automation. | Separate worker executables (duplicates ~60 MB of codecs). |
| ADR-007 | Core↔worker protocol: **JSON Lines over stdin/stdout** with heartbeats (04 §4.6). | Simple, debuggable, testable, no sockets. | Named pipes with custom framing; TCP (needs firewall/security work). |
| ADR-008 | Project files are **JSON** (`.alvs`) validated by a **JSON Schema** generated with schemars; all times are **integer flicks** (03 §3.3). | Human-readable, diffable, recoverable; flicks represent every supported frame rate and audio rate exactly. | Binary formats (opaque); floating-point seconds (drift); per-sequence frame counts (break with mixed rates). |
| ADR-009 | **Immutable state tree with `Arc` sharing**; every edit is an `EditOp` that produces a new root; undo/redo move between roots; UI updates are entity patches computed by `Arc::ptr_eq` diffing. | Undo cannot drift from redo; cheap snapshots; thread-safe sharing with the engine through `arc-swap`. | Inverse-command undo (bug-prone); mutable model with locks (contention with the render thread). |
| ADR-010 | IPC types are generated from Rust with **ts-rs 12** into `app/src/ipc/bindings/`; a CI check fails when generated files are stale. | One source of truth for DTOs; no hand-written type drift. `tauri-specta` is still a release candidate and is not used. | Hand-written TS types; RC dependencies. |
| ADR-011 | UI stack: **React 19 + TypeScript 6.0 + Vite 8**, **React Aria Components** (accessible, RTL-aware primitives), **dockview** (docking), **Zustand** (state), **react-intl** (ICU), **CSS Modules** with design tokens and logical properties (Stylelint enforced). | Proven, accessible, internationalized building blocks with large agent training corpora. TypeScript 6.0 is used because typescript-eslint supports `<6.1`. | Tailwind (physical utility classes leak RTL bugs), heavy UI kits with fixed look, TypeScript 7 (tooling not ready). |
| ADR-012 | The timeline and scopes are drawn on **Canvas 2D / WebGL2** with virtualization, not one DOM element per clip. | 60 fps with thousands of clips (NFR-PERF-05). | DOM per clip (layout cost). |
| ADR-013 | Working color space: **Rec.709 primaries, display-referred (BT.1886 / gamma 2.4 encoded), RGBA16F, premultiplied alpha**. HDR and wide-gamut sources are converted on input (2.6.3). | Matches the established behavior of display-referred editing pipelines (blend modes and dissolves look as users expect); keeps 10-bit+ precision. HDR timelines are planned for 1.1 (REQ-COL-007). | Linear-light working space (changes blend-mode semantics; more conversions). |
| ADR-014 | Audio I/O through **cpal 0.18 (WASAPI shared mode)**; 32-bit float internal mixing at the sequence rate; the **audio device clock is the master clock**. | Stable, low-latency, no proprietary SDK. | Exclusive mode by default (blocks other apps); ASIO (license/driver dependency). |
| ADR-015 | Text for titles and captions is shaped and laid out by **cosmic-text 0.19** (HarfBuzz-compatible shaping via rustybuzz, Unicode bidi), rasterized on the CPU per text change, animated on the GPU. | Correct Arabic shaping and mixed-direction text in exports, identical to preview. | Browser text rendering (would differ between preview and export). |
| ADR-016 | License **AGPL-3.0-or-later** for all project code and docs; **DCO** sign-off (no CLA); **trademark policy** protecting the names "AfterLeads" and "AfterLeads-Video-Spirits". | Strongest standard copyleft: any distributed or network-served modification must share its source, so improvements return to the community; compatible with GPL FFmpeg/x264/x265; the trademark policy makes renamed forks the only option for modified distributions, which encourages contributing upstream. | GPL-3.0 (no network clause, e.g., a hosted render service could keep changes private); MPL/Apache/MIT (allow closed forks). |
| ADR-017 | Speech-to-text through **whisper-rs 0.16** (bundled whisper.cpp); CPU (AVX2) default; optional Vulkan backend; models downloaded on demand and verified by SHA-256 (12 §12.15). | Fully local, MIT-licensed models, good multilingual accuracy including Arabic. | Cloud APIs (privacy, cost); ONNX runtime STT (no stable Rust binding baseline). |
| ADR-018 | Cache database **SQLite** (rusqlite `bundled`); large blobs (thumbnails, waveforms, proxies) as files referenced by the database. | Transactional, simple, rebuildable. | Embedding blobs in SQLite (database bloat). |
| ADR-019 | Packaging with the Tauri bundler: **NSIS per-user installer** plus a **portable ZIP**; updates through **tauri-plugin-updater** with **Ed25519** signatures and a static manifest on **GitHub Releases**. | No admin rights, no paid hosting, signed updates. | MSI only (admin-oriented); custom update server (cost). |
| ADR-020 | Testing: Rust unit/property/golden tests (golden images rendered on WARP), UI harness with a mock backend (Vitest + Playwright), end-to-end with **WebdriverIO + tauri-driver** on Windows. | Deterministic, runs on free GitHub Windows runners. | Manual-only UI testing. |
| ADR-021 | Development process: **OpenCode 2.2** agents, **Spec Kit 1.2.5** spec-driven workflow, GitHub Flow with protected `main` and merge commits (10 §10.4). | Structured, reviewable AI-driven development with explicit specs, plans, and tasks per work package. | Ad-hoc prompting (hallucination-prone). |
| ADR-022 | Crash capture with **crash-handler 0.8 + minidumper 0.11** (out-of-process minidumps written by the same executable in `--crash-server` mode) plus a Rust panic hook. Local only (REQ-REL-005). | Captures native crashes inside FFmpeg or drivers without admin-only Windows settings. | Windows Error Reporting LocalDumps (machine-wide registry, admin rights). |
| ADR-023 | HTTP client **ureq 3** with features `platform-verifier` and `win-system-proxy` for model downloads; the updater plugin uses its own client. | Small, blocking, uses the Windows certificate store and system proxy. | Custom TLS stacks. |

---

## 2.3 Process model

| Process | Started by | Role | Isolation and failure policy |
|---|---|---|---|
| GUI (`AfterLeads-Video-Spirits.exe`) | User | Tauri app: window, UI host, project model, undo, job orchestration, playback engine (decode threads, render thread, audio) | Autosave + recovery; GPU device-loss recovery (REQ-REL-006) |
| WebView2 processes | WebView2 runtime | Render the UI | Managed by WebView2; a crashed renderer is reloaded and state re-synchronized from Rust (snapshot) |
| Worker (`… --worker --job <file>`) | GUI job scheduler | One job: probe, thumbnails, waveform, proxy, optimize, stabilize, loudness, transcribe, export | Crash → job retried (REQ-JOB-003); never shares memory with the GUI |
| Crash server (`… --crash-server <id>`) | GUI at startup | Writes minidumps for GUI and worker crashes | If it is missing, crashes are still logged by the panic hook |
| CLI (`… cli <command>`) | Terminal / tests | Headless probe, render, frame export, validate | Same engine as workers; exit codes in 04 §4.7 |

Main entry (`app/src-tauri/src/main.rs`) parses the mode **before** any Tauri initialization:

```text
main()
 ├─ "--crash-server" → crash::server::run()
 ├─ "--worker"       → worker::run(job_path)          (no window, no Tauri)
 ├─ "cli"            → AttachConsole(ATTACH_PARENT_PROCESS); cli::run(args)
 └─ otherwise        → gui::run()                      (tauri::Builder)
```

The release build sets `#![windows_subsystem = "windows"]`; CLI mode attaches to the parent console to print output.

---

## 2.4 Components (CMP) and dependency rules

| ID | Component | Path | Responsibilities | May depend on |
|---|---|---|---|---|
| CMP-01 | `alvs-core` | `crates/alvs-core` | Flicks and rational time, IDs (UUID v7), error codes and `AppError`, logging setup helpers, path utilities (long paths, canonicalization) | — |
| CMP-02 | `alvs-model` | `crates/alvs-model` | Project/sequence/track/clip/effect/keyframe/caption/marker model, `EditOp` application, invariants and validation, undo/redo, entity patches, (de)serialization, JSON Schema, migrations, ts-rs exports | CMP-01 |
| CMP-03 | `alvs-formats` | `crates/alvs-formats` | Parsers/writers: SRT, WebVTT, ASS/SSA, `.cube` LUT, EDL, marker CSV and chapter text, text-encoding detection | CMP-01, CMP-02 |
| CMP-04 | `alvs-media` | `crates/alvs-media` | FFmpeg wrapper: probe, demux/decode (hardware and software), seeking, audio decode/resample, encode/mux, filter graphs (deinterlace, atempo, palettes), protocol whitelist | CMP-01 |
| CMP-05 | `alvs-render` | `crates/alvs-render` | wgpu device management, texture pool, source upload and color conversion, intrinsic transform/crop, effect runtime (WGSL), masks, blend modes, transitions, text/title rendering, scopes compute, RGB→YUV conversion, readback | CMP-01, CMP-02, CMP-04 |
| CMP-06 | `alvs-audio` | `crates/alvs-audio` | Audio graph, DSP effects, meters, loudness, cpal output/input, recording | CMP-01, CMP-02, CMP-04 |
| CMP-07 | `alvs-engine` | `crates/alvs-engine` | Timeline evaluator (what plays at time t), playback/transport and clocks, frame cache, preview sink trait, export pipeline, thumbnails/waveform/proxy/optimize/stabilize job implementations | CMP-01..CMP-06 |
| CMP-08 | `alvs-store` | `crates/alvs-store` | App paths, settings (JSON + schema), cache database (SQLite), recent projects, autosave/backup files, project lock, logs location | CMP-01 |
| CMP-09 | `alvs-jobs` | `crates/alvs-jobs` | Job queue, scheduling and concurrency, worker spawning and supervision, worker protocol (both sides), persistence | CMP-01, CMP-02, CMP-08 |
| CMP-10 | `alvs-stt` | `crates/alvs-stt` | Model manifest and downloads, whisper inference, segmentation into captions | CMP-01, CMP-02, CMP-04, CMP-08 |
| CMP-11 | `alvs-testkit` | `crates/alvs-testkit` | Test media generation (FFmpeg lavfi), golden image comparison (PSNR/SSIM), fixtures; dev-dependency only | CMP-01, CMP-04, CMP-05 |
| CMP-12 | App backend | `app/src-tauri` (crate `afterleads-video-spirits`) | Modes, Tauri commands/events, preview transport, windows, updater, crash handling, worker and CLI entry points | CMP-01..CMP-10 |
| CMP-13 | UI | `app/src` | React application: shell, panels, dialogs, timeline canvas, i18n, keyboard, IPC client, mock backend | CMP-12 (through IPC only) |
| CMP-14 | Repository tooling | `tools/` | Setup, git helpers, docs checks, i18n checks, asset generators, release scripts | — |

Rules (checked by `tools/check_deps.py` in CI, VER-09):

1. Crates depend only on the crates listed in their row. No cycles.
2. Only `alvs-media` uses `ffmpeg-next`; only `alvs-render` uses `wgpu`; only `alvs-audio` uses `cpal`; only `alvs-store` uses `rusqlite`; only `alvs-stt` uses `whisper-rs`; only CMP-12 uses `tauri` and `webview2-com`.
3. UI code never imports Rust types except the generated bindings in `app/src/ipc/bindings/`.
4. Public crate APIs are documented with rustdoc; `#![deny(missing_docs)]` on `alvs-core`, `alvs-model`, `alvs-formats`.

---

## 2.5 Runtime lifecycle

**Startup (GUI mode), in order**

1. Parse mode and flags (`--safe-mode`, Shift key state, file path argument).
2. Start the crash server process and install the crash handler and panic hook.
3. Resolve app paths (03 §3.8), open logs, load settings (validate, recover on corruption — REQ-SET-003).
4. Enforce single instance (tauri-plugin-single-instance): forward a project path to the running instance and exit.
5. Create the window and WebView2; the UI shows a skeleton within 300 ms.
6. Initialize GPU (2.6.9) and audio device (2.7.5) on background threads; report renderer name to the status bar.
7. Open the cache database; run cache migrations; integrity check (rebuild on failure, E8003).
8. Re-queue unfinished jobs (REQ-JOB-003).
9. Detect an abnormal previous exit (session marker file present) → crash recovery flow (REQ-REL-001).
10. Restore the last project if `alvs.general.restoreLastProject` is on, else show the Welcome screen.
11. Schedule the update check (non-blocking, REQ-UPD-001).

**Shutdown**

1. If dirty, prompt (REQ-APP-012). 2. Stop playback. 3. Ask running jobs: exports continue only if the user chooses "Finish in background" (default: cancel with confirmation). 4. Flush settings and autosave state, release the project lock. 5. Stop workers (graceful cancel, kill after 10 s). 6. Destroy GPU resources, close the database, remove the session marker.

**Safe mode** (REQ-REL-002): WARP adapter, hardware decode off, GPU transcription off, default workspace, update check off for the session.

---

## 2.6 Render engine

### 2.6.1 Frame requests

A `RenderRequest` contains the state snapshot (`Arc<ProjectState>`), sequence ID, time (flicks), target (`Preview{viewer, scale}`, `Export`, `Thumbnail{size}`, `Scope`), and quality (`Draft` for scrubbing, `Normal` for playback, `Final` for export and stills).

### 2.6.2 Timeline evaluation → render graph

For time `t` in sequence `S`:

1. Walk video tracks from bottom to top. Skip hidden tracks and disabled clips.
2. On each track, find the clip covering `t` by binary search over the start-sorted item list. If `t` is inside a transition, evaluate both clips and a transition node.
3. Map timeline time to source time: `src = sourceIn + (t − start) × speed` (reverse: `src = sourceIn + duration·speed − (t − start)·speed − 1 frame`); freeze frames hold their frame. Source frames are selected by presentation timestamp (`[pts, nextPts)` contains `src`), which makes variable-frame-rate media exact (REQ-MED-007).
4. Clip types: media → `SourceFrame` node; title → `Text` node; generator → `Generator` node; nested sequence → recursive evaluation (depth ≤ 8, cycles are impossible by invariant); adjustment layer → applies its effect chain to the accumulated image below it.
5. Per visual clip, nodes are chained: `Source → ColorIn → Intrinsic(crop, transform, flip) → Effect₁…Effectₙ → Masks → Opacity → Composite(blend mode) onto the accumulator`.
6. The accumulator starts as transparent black at sequence resolution × preview scale. The final node is `Output`: preview (RGBA8 with ordered dither) or export (YUV conversion, 2.9).

Graph construction is pure (snapshot in → node list out) and unit-tested without a GPU.

### 2.6.3 Color pipeline

- **Working space:** Rec.709 primaries, BT.1886-encoded display-referred values, full range, RGBA16F, premultiplied alpha (ADR-013).
- **YUV to RGB:** matrix from stream tags (BT.601, BT.709, BT.2020 non-constant luminance); unknown → BT.601 when height ≤ 576, else BT.709. Range from tags; unknown → limited for YUV, full for RGB and images. 10/12-bit planes are normalized exactly (P010 shifts by 6 bits).
- **SDR, BT.709/sRGB/unknown transfer:** values pass through unchanged (display-referred convention).
- **Other SDR primaries (BT.2020 SDR, Display P3):** linearize with the source transfer, convert primaries with a 3×3 matrix to BT.709 (same D65 white point), soft-clip out-of-gamut values, re-encode with gamma 2.4.
- **HDR (PQ ST 2084, HLG ARIB STD-B67):** linearize to absolute luminance (HLG with the BT.2100 OOTF for a 1000-nit display), convert BT.2020 → BT.709 primaries, tone-map with the BT.2390 EETF from the source peak (MaxCLL, else mastering peak, else 1000 nits) to 100 nits, soft-clip gamut, re-encode with gamma 2.4 (REQ-MED-008).
- **Alpha:** straight-alpha sources are premultiplied on upload; all effects operate on premultiplied data or explicitly unpremultiply/premultiply around per-color math.
- **Preview output:** working values converted to RGBA8 with a 4×4 ordered dither; no further transform (the viewer shows display-referred values).
- **Export output:** RGB → Y'CbCr with the BT.709 matrix, limited range for video codecs, full range for PNG/JPEG/GIF; 4:2:0 chroma is averaged with left (MPEG-2 style) siting; streams are tagged `bt709/bt709/bt709/tv` (REQ-EXP-008).

### 2.6.4 Texture formats and precision

| Data | Format |
|---|---|
| 8-bit 4:2:0 (NV12, YUV420P) | Y: `R8Unorm`; UV: `Rg8Unorm` or two `R8Unorm` planes |
| 10-bit 4:2:0 (P010, YUV420P10) | Y: `R16Unorm`; UV: `Rg16Unorm` or two `R16Unorm` |
| 4:2:2 / 4:4:4 (8/10/12-bit) | Planar `R8Unorm` or `R16Unorm` per plane |
| RGB(A) 8/16-bit images | `Rgba8Unorm` / `Rgba16Unorm` |
| OpenEXR half/float | `Rgba16Float` |
| Working images, effect intermediates | `Rgba16Float` |
| Preview readback | `Rgba8Unorm` |

Feature requirement: `Rgba16Float` must support render attachment, storage, and filtering — guaranteed on D3D12 feature level 11_0 and WARP. Any optional wgpu feature used MUST be checked at device creation with a fallback path or a clear error (E3001).

### 2.6.5 Resampling

Preview uses bilinear filtering with mipmaps for downscales larger than 2×. Final quality uses separable Catmull-Rom bicubic for scale factors ≥ 0.5 and mip-prefiltered bicubic below that. Rotation uses the same filters in the transform pass.

### 2.6.6 Effect implementation rules

1. Each effect = manifest (04 §4.8) + WGSL module(s) in `crates/alvs-render/src/effects/<effect>/`. Multi-pass effects declare passes and intermediate sizes.
2. Parameters are packed into a uniform buffer with WGSL alignment rules (`bytemuck` + explicit `#[repr(C)]` structs with padding tests).
3. Effects receive premultiplied RGBA16F and MUST output premultiplied RGBA16F.
4. Determinism: no randomness except seeded noise (`seed` + frame index hash); no dependence on wall-clock time.
5. Spatial parameters arrive in pixels of the current render resolution (the engine converts from normalized units), so preview scale does not change the look.
6. Algorithm summary per effect (the manifest's `shader` notes MUST match):

| Effect | Algorithm |
|---|---|
| VFX-01 Basic Correction | Linearize (gamma 2.4) for exposure (×2^EV) and white balance (temperature/tint as multipliers derived from a Planckian-locus approximation around D65); re-encode; contrast as an S-curve around 0.435 (mid-gray encoded); highlights/shadows as smooth luminance-masked offsets; whites/blacks as endpoint shifts; saturation/vibrance in Rec.709 luma-chroma space (vibrance weights low-saturation pixels more). |
| VFX-02 Color Wheels | Per-channel lift/gamma/gain/offset: `out = (in·gain + lift·(1 − in))^(1/gamma) + offset`, chroma wheel vectors converted to RGB offsets on the Rec.709 chroma plane; saturation after. |
| VFX-03 Curves | 1024-entry 1D lookup textures built on the CPU from monotone cubic curves; master then per channel. |
| VFX-04 LUT | 3D texture (size N³, `Rgba16Float`) sampled with tetrahedral (default) or trilinear interpolation; 1D LUTs as 1D lookups; domain mapping from the file header; intensity mixes with the input. |
| VFX-08 Gaussian Blur | Separable two-pass with σ = radius/3, kernel truncated at 3σ, mip-assisted for radius > 64 px. |
| VFX-11 Sharpen | Unsharp mask: `in + amount·(in − blur(in, radius))` applied where the absolute difference between `in` and `blur` exceeds `threshold`. |
| VFX-13 Chroma Key | Distance in a perceptual chroma space (Y'CbCr normalized) from the key color with tolerance/softness ramp; spill suppression clamps the key channel toward the average of the others; choke = erode/dilate of the matte. |
| VFX-15 Vignette | Elliptical distance field with roundness and feather, multiply/mix toward color. |
| VFX-16 Glow | Threshold → blur → additive mix with tint. |
| VFX-17 Film Grain | Value-noise texture hashed from (pixel, frame, seed), luminance-weighted, optional chroma noise. |
| VFX-21 Corner Pin | Homography computed on the CPU from 4 points; inverse mapping in the shader. |
| Others | Direct per-pixel or neighborhood formulas as named in 12 §12.1. |

### 2.6.7 Text and titles

cosmic-text shapes and lays out each text layer (bidi, line breaking, alignment) with fonts from `assets/fonts/` and installed system fonts (font database built once per session). Glyph outlines are rasterized on the CPU (fill and stroke coverage masks) at the target resolution and cached by (layout hash, size). The GPU composites fill, stroke, shadow (blurred mask), and background box (rounded-rectangle signed distance), and applies animations (TAN) as transforms, opacity, blur, and per-grapheme reveal masks. Captions use the same renderer (REQ-CAP-006).

### 2.6.8 Caches

- **Rendered frame cache:** GPU textures keyed by (sequence ID, time, quality, render-relevant hash of the entities that cover that time); LRU; budget `alvs.playback.gpuCacheMB` (default 1024, clamped to 25% of adapter memory).
- **Invalidation:** each entity patch (ADR-009) yields the affected time ranges (clip extents, transition extents, nested-sequence users); cached frames in those ranges are dropped.
- **Decoded frame cache:** per decoder, the last 8 decoded frames (CPU memory) to make frame stepping and short reverse steps fast.

### 2.6.9 GPU device management

- Adapter: `alvs.advanced.gpuAdapter` = `auto` (high-performance preference) or a specific adapter LUID; `alvs.advanced.forceWarp` forces WARP.
- Device loss: the uncaptured-error and device-lost callbacks trigger recreation of device, pipelines, and caches, then re-render of the current frame; 3 losses within 10 minutes switch the session to WARP (REQ-REL-006).
- Out of memory: halve the frame cache, then lower preview resolution one step, then report E3003.

---

## 2.7 Audio engine

### 2.7.1 Graph

```text
per clip:  decode → resample to sequence rate → channel map/downmix → time-stretch (speed)
           → clip gain + volume automation + fades → clip effects → pan (−3 dB constant power)
per track: Σ clips → track effects → track volume/pan → mute/solo
master:    Σ tracks → master effects → master volume → meters → device or encoder
```

Solo rule: if any track is soloed, only soloed tracks are heard (mute still wins). Crossfades (TRS-14..16) apply gain curves to the overlapping clips.

### 2.7.2 Scheduling

The mixer thread renders 512-frame blocks (10.7 ms at 48 kHz) ahead of the device into a lock-free ring (`rtrb`); the cpal callback only copies from the ring (no locks, no allocation). Underruns output silence and increment a counter shown in diagnostics (E4004 when sustained). Export pulls blocks synchronously with the same graph.

### 2.7.3 DSP algorithms

| Effect | Algorithm |
|---|---|
| AFX-01 Parametric EQ | RBJ "Audio EQ Cookbook" biquads (shelves, peaking, 2nd-order HP/LP), direct form II transposed, f64 coefficients, parameter smoothing over 10 ms |
| AFX-02 Compressor | Feed-forward, log-domain gain computer with soft knee, peak/RMS-blend detector, attack/release smoothing, optional automatic makeup |
| AFX-03 Limiter | 5 ms look-ahead, 4× oversampled true-peak detection (polyphase FIR), smooth gain envelope; guarantees output ≤ ceiling dBTP |
| AFX-04 Noise Gate | Hysteresis (open at threshold, close at threshold − 4 dB), attack/hold/release, attenuation `range` |
| AFX-05 De-esser | Band-pass side-chain around `frequency`; split-band gain reduction limited to `maxReduction` |
| AFX-06 Voice Denoise | RNNoise network via `nnnoiseless` on 480-sample frames at 48 kHz (resampled internally when needed), dry/wet `mix` |
| AFX-07 Reverb | Schroeder-Moorer design: 8 parallel damped comb filters and 4 series all-pass filters per channel, stereo spread, pre-delay |
| AFX-08 Delay | Circular buffer with feedback, optional ping-pong cross-feedback, 5 ms parameter smoothing |
| AFX-09 Gain / AFX-10 Channel Tools | Direct gain and channel matrix operations |
| AFX-11 Pitch Shift | Resample by the pitch ratio, then time-stretch by the inverse ratio (2.7.6) |

### 2.7.4 Loudness

`ebur128` in modes I, S, M, LRA, and TRUE_PEAK. Normalization (REQ-AUD-004) runs as an analysis job: gain = target − integrated loudness, reduced so that true peak + gain ≤ ceiling (default −1 dBTP). The resulting gain is stored on the clip (or applied at export for "normalize export").

### 2.7.5 Devices

cpal WASAPI host, shared mode, stereo float output at the device rate; sequence-rate audio is resampled with `rubato` when rates differ. Device removal or default-device change: reopen the default device and notify (E4001). Buffer presets map to WASAPI periods of 10 ms (low), 20 ms (normal), 40 ms (safe).

### 2.7.6 Time-stretch and recording

- Pitch-preserving speed (REQ-AUD-009) uses an FFmpeg `atempo` filter chain (factors outside 0.5–2.0 are split into multiple stages) inside `alvs-media`, run ahead on the decode thread.
- Voice-over (REQ-AUD-006): a cpal input stream writes into a ring; a writer thread stores 24-bit WAV via FFmpeg's PCM muxer in `<project folder>/Recordings/`; the clip is inserted on stop with the recording's measured latency compensated.

---

## 2.8 Playback engine

- **Transport state machine:** `Paused ⇄ Playing(rate) ⇄ Scrubbing`; rates −8…+8 (12 §12.10); `Play every frame` disables dropping.
- **Clock:** while playing, media time = samples consumed by the audio device callback + reported device latency (an output stream always runs, silent when needed). With no audio device, a monotonic system clock is used.
- **Scheduling:** the control thread requests frames `t + k·frameDuration` for `k = 0..N` (N = 6 at Full, adaptive); the render thread renders them in order; the presenter thread publishes each frame when its time arrives and drops frames that are late by more than half a frame (counted for the status bar, REQ-PLY-004).
- **Decoders:** a pool keyed by (media, stream, original/proxy), at most 16 open; seek = keyframe seek then decode forward; hardware frames are transferred to system memory (NV12/P010) before upload. Reverse playback decodes a GOP forward into a cache and presents backwards.
- **Scrubbing:** requests are coalesced to the latest target; a `Draft` frame is shown first, refined to `Normal` when the playhead rests 150 ms (REQ-PLY-007). Audio scrub plays 60 ms windowed grains.
- **Preview sinks:** `SharedBufferSink` (04 §4.5) for in-window viewers; `NativeWindowSink` (wgpu swap chain on a separate top-level window) for full screen and external preview (REQ-PLY-008).

---

## 2.9 Export pipeline

### 2.9.1 Flow (inside a worker)

1. Load the immutable project snapshot written by the GUI (04 §4.6.3), build the timeline evaluator, a wgpu device (WARP fallback), and an offline audio graph.
2. Preflight is repeated in the worker (paths, codecs, dimensions).
3. Pipeline with three frames in flight: render frame N+2, read back N+1, encode N. Video conversion RGB→YUV runs in a compute shader; planes are read back and wrapped in `AVFrame`s.
4. Audio blocks are pulled per frame duration and buffered to the encoder frame size.
5. Captions: burn-in uses the title renderer; sidecars are written by `alvs-formats`; embedded streams use the muxer's subtitle encoders (REQ-CAP-005). Chapters are written as container chapters (REQ-EXP-009).
6. Output goes to `<name>.part`, then is verified (REQ-EXP-008) and renamed atomically (REQ-JOB-004).

### 2.9.2 Encoder selection and quality mapping

- Software encoders per 12 §12.7. Hardware selection per 12 §12.19, preferring the vendor of the adapter that renders.
- Availability is probed at startup by opening each hardware encoder with a 256×256 test stream (cached per driver version); unavailable encoders fall back to software (E5004).
- Constant-quality mapping: hardware quality value = preset CRF + 1 using each encoder's constant-quality rate-control mode. The exact FFmpeg option names and values live in `crates/alvs-media/src/encode/hw_options.rs`; each mapping has a unit test that opens the encoder successfully on a machine with that hardware (T-EXP-002), and options are verified against `ffmpeg -h encoder=<name>` output of the built FFmpeg (G-10).

---

## 2.10 Threading and concurrency

| Thread / pool | Owner | Rules |
|---|---|---|
| Main (Tauri event loop) | CMP-12 | Never blocks more than 8 ms; long work goes to the async runtime |
| Async runtime (tokio, 2–4 workers) | CMP-12 | IPC command handlers, file I/O, job orchestration |
| Model actor | CMP-02 via CMP-12 | Single writer for `ProjectState`; publishes snapshots through `arc-swap` |
| Engine control | CMP-07 | Transport state machine; communicates by `crossbeam-channel` |
| Render | CMP-05 | Owns the wgpu device and queue |
| Decode pool (≤ 4) | CMP-04/07 | One FFmpeg decoder context per thread at a time |
| Audio mixer | CMP-06 | Real-time safe: no locks or allocation in the device callback |
| Presenter | CMP-07/12 | Publishes preview frames |

Rules: no shared mutable state without an owner; locks (`parking_lot`) are held for microseconds and never across I/O or GPU waits; every thread has a shutdown path; panics in engine threads are caught at the thread boundary, logged (E0001), and the subsystem restarts.

---

## 2.11 Errors and logging

- Each crate defines `thiserror` error enums. Every variant maps to exactly one code in 12 §12.13 through `fn code(&self) -> ErrorCode`.
- Errors crossing to the UI become the error envelope (04 §4.9); the UI maps `code` to the message key `error.<code>` with parameters.
- No `unwrap()`/`expect()` in non-test code except for documented invariants (`expect("invariant: …")`); CI runs clippy with `unwrap_used` and `expect_used` set to warn and denies new warnings.
- Logging: `tracing` with `tracing-subscriber` (JSON lines) and `tracing-appender` (rolling, REQ-REL-003). Required fields: `ts`, `level`, `target`, `msg`, plus `job_id`, `project_id`, `code` when known. Never log media content, caption text, or full user paths at info level (paths appear only at debug and are redacted in diagnostics bundles).

---

## 2.12 Roadmap readiness

| Roadmap item | Hook in the 1.0 architecture |
|---|---|
| Speed ramps (REQ-EDT-018) | Clip time mapping is a function `src(t)` in the evaluator; ramps replace the linear function with a keyframed curve. |
| Silence detection (REQ-EDT-019) | Loudness analysis jobs already produce short-term loudness series. |
| Tracking/stabilization (REQ-FX-013, REQ-MED-012) | Analysis job type and baked-media mechanism (REQ-MED-013) already exist. |
| Screen/webcam capture (REQ-MED-014) | Recording writer path from voice-over; jobs framework. |
| HSL and HDR (REQ-COL-006/007) | Effect runtime; color pipeline isolates input/output transforms. |
| TTS (REQ-AUD-011) | `alvs-stt` model manager pattern (manifests, downloads, hashes) is reused. |
| OTIO (REQ-INT-004) | `alvs-formats` pattern; model import via `EditOp::Batch`. |
| ARM64 (REQ-APP-013) | No x86-only code outside FFmpeg/whisper builds; vcpkg triplet switch. |
| Plugins (REQ-AUD-012), shader packs (REQ-FX-014) | Effect manifests are data-driven; audio graph nodes are trait objects; worker isolation model exists. |
| Multicam (REQ-EDT-020) | Nested sequences and source-viewer architecture. |

---

## 2.13 Repository layout

```text
AfterLeads-Video-Spirits/
├─ AGENTS.md  README.md  LICENSE  TRADEMARKS.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md
├─ SECURITY.md  STATUS.md  opencode.jsonc
├─ .opencode/agents/  .opencode/commands/       OpenCode agents and commands (06)
├─ .specify/                                    Spec Kit (constitution in memory/)
├─ .githooks/                                   pre-commit, commit-msg, pre-push (10 §10.4)
├─ .github/                                     workflows (WP-01), PR and issue templates
├─ docs/                                        00–12 (this documentation)
├─ specs/NNN-slug/                              Spec Kit feature artifacts per work package
├─ Cargo.toml  Cargo.lock  rust-toolchain.toml  deny.toml  about.toml  .cargo/config.toml
├─ vcpkg.json  vcpkg-configuration.json         FFmpeg build manifest (WP-06)
├─ crates/
│  ├─ alvs-core/  alvs-model/  alvs-formats/  alvs-media/  alvs-render/
│  ├─ alvs-audio/  alvs-engine/  alvs-store/  alvs-jobs/  alvs-stt/  alvs-testkit/
├─ app/
│  ├─ package.json  vite.config.ts  tsconfig.json  eslint.config.js  stylelint.config.js
│  ├─ index.html
│  ├─ src/                                      UI (07): app/, design/, panels/, dialogs/,
│  │                                            timeline/, state/, ipc/, i18n/, keyboard/,
│  │                                            commands/, help/, mocks/, locales/
│  ├─ tests/                                    Vitest unit tests, Playwright harness tests
│  └─ src-tauri/                                Rust app crate, tauri.conf.json, capabilities/
├─ assets/  fonts/  luts/  templates/  models/  icons/  help/<locale>/
├─ tests/e2e/                                   WebdriverIO specs (tauri-driver)
├─ tools/   setup/  git/  docs/  i18n/  media/  models/  assets/  release/  verify.ps1
├─ package.json  pnpm-workspace.yaml  pnpm-lock.yaml
└─ .gitattributes  .editorconfig  .gitignore
```

---

## 2.14 Coding standards

**All files**

- UTF-8, LF line endings (CRLF only for `.bat`/`.cmd`), final newline; `.editorconfig` is authoritative.
- Header on every source file (Rust, TS/TSX, CSS, WGSL, PowerShell, Python, sh):
  `SPDX-License-Identifier: AGPL-3.0-or-later` and `Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>` as comments.
- Names in English; user-facing text only through localization keys (G-13).

**Rust**

- `rustfmt` default style; `clippy -D warnings` with `pedantic` warnings reviewed per crate (`#![warn(clippy::pedantic)]` in core crates).
- Errors per 2.11; no `unsafe` outside `alvs-media` (FFmpeg FFI), `alvs-render` (buffer casts), and CMP-12 (WebView2/Win32); every `unsafe` block has a `// SAFETY:` comment and a test.
- Public functions document panics, errors, and units (flicks, normalized coordinates).
- Tests next to code (`#[cfg(test)]`) plus integration tests in `tests/`.

**TypeScript/React**

- `strict` TypeScript, no `any` (lint error), function components with hooks, no default exports except route-level lazy chunks.
- Components accessible by construction: React Aria Components for interactive primitives; `eslint-plugin-jsx-a11y` errors fail CI.
- No JSX text literals (`eslint-plugin-i18next/no-literal-string`); messages via `react-intl` IDs `area.component.key`.
- Styling with CSS Modules and design tokens (07 §7.4); only logical properties (`margin-inline-start`, `inset-inline-end`, …) — enforced by `stylelint-use-logical`.
- State: server state mirrored from Rust in normalized Zustand stores; derived data with selectors; no business rules in the UI.

**WGSL**

- One entry point per pass, uniforms declared in a struct matching the Rust `#[repr(C)]` mirror, constants documented with units.

**Commit hygiene:** Conventional Commits with scopes from 10 §10.4.5; DCO sign-off; no generated files edited by hand (bindings, notices, manifests are regenerated by scripts).

---

## 2.15 Performance engineering rules

1. Budgets are requirements (NFR-PERF, NFR-RES). Every work package that touches a hot path adds or updates a benchmark (`criterion` in Rust, Playwright performance marks in the UI harness).
2. No per-frame heap allocation in render, decode, mixer, or presenter loops after warm-up (pools and ring buffers).
3. UI: no React re-render per playback frame; the timecode and playhead are updated through refs/canvas at animation-frame rate from the playback state event.
4. Measure before optimizing: use `tracing` spans with timing in debug builds and the in-app statistics overlay (Ctrl+Shift+F12 in debug builds only).
