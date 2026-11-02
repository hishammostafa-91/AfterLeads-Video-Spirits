# Data Model: WP-02 — Core types and project model

This package *implements* the canonical data model; docs/03 §3.3–3.5 remains normative for every field, rule, and JSON shape. This document maps entities to modules and records the validation/state semantics this package adds around them (research D3–D12).

## Entity → module map

| Entity (canonical section) | Module | Notes |
|---|---|---|
| Flicks, frame math (§3.3.1) | `alvs-core::time` | literal per-rate table asserted by test (D4); timecode type here too (D5) |
| Rational (§3.3.2) | `alvs-core::rational` | reduced `num/den`, `den > 0` |
| Normalized space, dB/pan (§3.3.3–3.3.4) | `alvs-model` parameter value types | documented conventions on the value types |
| Project top level, settings, view state (§3.4.1, §3.4.11) | `alvs-model::project` | `ui` is non-undoable view state |
| MediaItem (§3.4.2) | `alvs-model::media` | fingerprint/probe snapshot/imageSequence/derived |
| Bin (§3.4.3) | `alvs-model::bin` | tree, no cycles |
| Asset (§3.4.4) | `alvs-model::asset` | LUT info, sha256 |
| Sequence (§3.4.5) | `alvs-model::sequence` | format incl. `rec709-sdr`, `autoMatchPending`, master audio |
| Track, Clip, intrinsic, audio object (§3.4.6) | `alvs-model::track` | clip types media\|title\|generator\|sequence |
| EffectInstance, AnimatableValue, Keyframe (§3.4.7) | `alvs-model::effect` | static/keyframed; all §3.4.7 value types |
| Mask (§3.4.8) | `alvs-model::mask` | usage clip\|effect |
| TitleDocument (§3.4.9) | `alvs-model::title` | layers, anim in/out, template |
| Transition, Marker, CaptionTrack (§3.4.10) | `alvs-model::transition` / `marker` / `caption` | alignment coverage rules |
| EditOp, ParamTarget, EditResult (04 §4.4) | `alvs-model::edit` | serde tag `"op"`; `history.<op>` keys |
| EntityPatch (04 §4.4) | `alvs-model::patch` | revision + upsert/remove + dirtyRanges |
| History (ADR-009) | `alvs-model::history` | Arc roots; ≥200 guarantee; 512 MB estimate budget |
| Migration registry (03 §3.9.6) | `alvs-model::migrate` | forward-only steps + backup contract |
| ErrorCode, AppError, ErrorEnvelope (02 §2.11, 04 §4.9) | `alvs-core::error` | envelope shape normative |
| PathDecision (05 SEC-04) | `alvs-core::path` | Allowed / NeedsConfirmation(Unc) / Denied |

## Validation rules (03 §3.5 — all implemented in `alvs-model::validate`)

All fifteen rule groups as `Issue { code, path, detail }`: unique UUID v7 IDs; reference integrity (the §3.5.2 list); nesting acyclic, depth ≤ 8 (E1006); track sort + no overlap; frame-grid multiples (audio quarter-frame); `duration > 0`, speed 1/10..16; source bounds incl. `freezeAt`; transition adjacency/handles/one-per-edit; keyframe sort/uniqueness/type/`h`-only-for-bezier/bool-enum-hold; link-group shape; caption sort/overlap/length; marker times + chapter uniqueness; format ranges (even 16–8,192; 44.1/48 kHz); asset sha256; size limits (§3.5.15) checked first. Applied after every op in debug/tests; violation ⇒ rollback + E0002.

## State transitions

- `Model`: `(root, History)` — `apply(op)` ⇒ new root pushed to undo, redo cleared, `revision`+1; invalid ⇒ prior root retained, E0002.
- `History`: push ⇒ evict oldest while `estimated_size > 512 MB` **and** `len > 200`; cap 1,000; `undo`/`redo` move roots between stacks; `step(i)` truncates to a snapshot (History-panel jump, consumed by WP-13).
- File lifecycle (03 §3.9.6): `older schema → migrate forward → validate → open`; `newer schema → read-only (E1004)`; migration driver signals the caller (WP-03) to write the `.v<old>.bak` first.

## Relationships

- `Project` 1—N `MediaItem`/`Bin`/`Asset`/`Sequence`; `Sequence` 1—N `Track`/`CaptionTrack`/`Marker`; `Track` 1—N `Clip`/`Transition`; `Clip` 1—N `EffectInstance`/`Mask`/`Marker` + optional `TitleDocument`/generator params; `EffectInstance` N—M `Mask` (`maskIds`).
- Cross-links constrained by §3.5.2: `clip.mediaId`/`sequenceId`, `media.binId`, `bin.parentId`, `settings.defaultSequenceId`, `ui.activeSequenceId`, `asset:` value references.
- `EditOp` → new `Project` root → `EntityPatch` (ten entity kinds) — the UI-store contract (04 §4.4).
