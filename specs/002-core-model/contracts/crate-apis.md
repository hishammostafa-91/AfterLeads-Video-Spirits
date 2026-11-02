# Contract: public API of `alvs-core` and `alvs-model` (WP-02)

These crates are libraries consumed by later work packages (engine, store, app backend, UI via bindings). The contract below is the *public surface* WP-02 commits to; signatures are descriptive (exact Rust names settle at implementation, verified to compile and pass docs tests). Canonical authorities: 03 §3.2–3.5, §3.9.6; 04 §4.2, §4.4, §4.9; 02 §2.11, ADR-008/009; 05 §5.7 SEC-04.

## `alvs-core`

| Area | Contract |
|---|---|
| `time` | `Flicks(i64)`; `FLICKS_PER_SECOND = 705_600_000`; `frame_duration(rate: Rational) -> Flicks` (exact for every 12 §12.9 rate + 48k/44.1k; custom-rate validity error otherwise); `frame_start(n, rate)`; `convert_foreign(ticks, from_rate)` round-half-to-even; `MAX_SEQUENCE_LENGTH` (24 h). `Timecode`: `from_flicks(f, rate)`, `to_flicks(s: &str, rate) -> Result<(Flicks, …)>` accepting `HH:MM:SS:FF`, `;` drop-frame, frames and seconds forms, `+`/`-` relative; formatting to the same forms (REQ-PLY-005). |
| `rational` | `Rational { num: u32, den: u32 }`, always reduced, `den > 0`; construction normalizes; arithmetic needed for rate math. |
| `id` | UUID v7 mint (`new_id() -> Id`), parse/format per 03 §3.2.2 (lowercase hyphenated). |
| `path` | `PathDecision = Allowed \| NeedsConfirmation \| Denied(AppError)`; `check_path(candidate, roots)` per SEC-04 (canonicalize; reject `\\.\`, `\\?\GLOBALROOT`, reserved names; containment to supplied roots; UNC-from-project ⇒ NeedsConfirmation). |
| `error` | `ErrorCode` (codes listed in research D7), `AppError: std::error::Error` with `fn code(&self) -> ErrorCode` on every variant, `ErrorEnvelope` serializable exactly per 04 §4.9 with `recovery: Vec<{ commandId, labelKey }>`. |

## `alvs-model`

| Area | Contract |
|---|---|
| Types | Every 03 §3.4 entity (module map in [data-model.md](../data-model.md)); serde per 03 §3.2 (camelCase, catalog enum IDs, `extra` maps on persisted structs); every flicks field `#[ts(type = "number")]`. |
| `dto` | DTO re-exports deriving `Serialize + Deserialize + TS` (04 §4.2); `extra` maps **included** in the TS surface via ts-rs's `serde-json-impl` feature — flattened maps render as `T & { [key: string]: JsonValue }` (research D16); the `export_bindings` test writes `app/src/ipc/bindings/` via `TS::export_all`/`TS_RS_EXPORT_DIR` (research D15), and the **CI drift step runs in `rust-lint`** (export test + `git diff --exit-code -- app/src/ipc/bindings`, mirroring verify.ps1 lines 58–61). |
| `edit` | `EditOp` (`#[serde(tag = "op")]`, 04 §4.4 field shapes for the WP-02 categories: project/sequence/track/media/bin/marker/caption-track + `batch`); `ParamTarget`; `EditResult { patch: EntityPatch, warnings: Vec<…>, adjusted: Vec<…> }`; `Model::apply(&EditOp) -> EditResult`; `history_key()` per variant. |
| `history` | `Model::undo()/redo() -> Option<EditResult-like patch>`; `steps() -> &[Arc<Project>]`; `step(i)` jump; ≥ 200 entries guaranteed; ≤ 1,000; estimated-size budget 512 MB, oldest-first eviction never below 200. |
| `patch` | `EntityPatch` per 04 §4.4 (revision; upsert/remove for media, bins, sequences, tracks, clips, transitions, markers, captionTracks, captions, assets; dirtyRanges); `diff(old, new)` via `Arc::ptr_eq` short-circuit; patch-application equivalence is a tested property (spec US4/AC4). |
| `validate` | `validate(&Project) -> Vec<Issue>`; all 03 §3.5 rules; size limits first. |
| `migrate` | Forward-only registry + driver; validation of the migrated value; `MigrationOutcome { Migrated { from, to }, ReadOnlyNewer (E1004), BackupRequired contract }`; fixtures under `tests/fixtures/migrations/`. |
| `schema` | `project_json_schema() -> schemars::schema::Schema` (ADR-008; generation entry verified at implementation). |

## Behavioral guarantees (test-enforced)

1. Undo/redo are exact inverses over ≥ 200 steps; eviction respects the budget and the floor (T-PRJ-006).
2. Any op whose result violates 03 §3.5 leaves the model unchanged and reports E0002 semantics.
3. Round-trips: JSON ⇄ types (incl. unknown fields), flicks ⇄ timecode at every catalog rate (drop-frame included), bindings regeneration byte-stable.
4. Random edit sequences (proptest, 10,000 × 50 in CI) either apply with all invariants held or reject with a documented code (09 §9.4).
