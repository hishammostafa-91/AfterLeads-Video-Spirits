# WP-03 Data Model — Storage and application services

All shapes are normative in docs/03 §3.6–3.9; this artifact maps them to the delivery modules and records validation/state the spec's FRs demand. Nothing here adds fields beyond the canonical documents.

## 1. Entities and module map

| Entity (canonical) | Fields / shape | Module (`crates/alvs-store/src/`) | Rules |
|---|---|---|---|
| **App folders** (03 §3.8) | settings/library/cache/models/logs/crashes/marker/updates/temp/default-project roots; installed vs portable (`portable.flag` → `<exe>\data\…`) | `paths.rs` | SEC-04 containment on every write; Unicode + long paths via `alvs-core::path`; env roots resolved once |
| **Settings document** (03 §3.6) | flat `{ "settingsVersion": 1, <key>: value }`; 68-key table (key, kind, range/enum, default, page) | `settings.rs` (+ `keys.rs` table) | version must be 1; per-key validation, invalid → default + one E8002; unknown keys preserved; 500 ms debounce; atomic write; `.bak` last-good |
| **Cache database** (03 §3.7) | `cache.sqlite`: `meta`, `media_cache`, `thumbnails`, `waveforms`, `proxies`, `jobs` (+ `jobs_by_state`), `recent_projects`, `models`, `encoder_probe`, `analysis` | `cache.rs` + `migrations/001_init.sql` | WAL / FK ON / NORMAL / busy_timeout 5000; migrations `NNN_<name>.sql` in one transaction, version in `meta`; `quick_check` → rename `.corrupt-<ts>` + rebuild (E8003) |
| **Recent project row** (03 §3.7) | `path` (PK), `name`, `thumbnail_file?`, `opened_utc`, `pinned` | `recent.rs` | upsert on save/open; newest-first; trim to `alvs.general.recentLimit` |
| **Project document** (`.alvs`, WP-02 types) | bytes on disk; model-aware steps via `DocumentHooks` (D2) | `project.rs` | open: exist → raw-size gate → parse → migrate (`.v<old>.bak` first) → validate → lock; save: §3.9.2 pipeline |
| **Autosave sidecar** (03 §3.8/§3.9.3) | `<Name>.alvs.autosave`, exists only while dirty | `autosave.rs` | dirty ∧ interval elapsed → atomic write; never the main file; deleted on save/clean close |
| **Backup** (03 §3.8/§3.9.2; REQ-PRJ-004) | `Backups\<Name>_<yyyyMMdd-HHmmss>.alvs` + size/time for listing | `backup.rs` | one per manual save; prune to `alvs.general.backupsKeep`; restore copies into a new file |
| **Lock file** (03 §3.9.4; REQ-PRJ-011) | `<Name>.alvs.lock` = `{ pid, host, user, startedUtc, appVersion }` | `lock.rs` | exclusive create; live-same-host → read-only (E1007); dead → take-over; other host ≥ 24 h → take-over; removed on close |
| **Session marker** (03 §3.8/§3.9.5) | `session.marker` = `{ pid, startedUtc }` | `session.rs` | created at startup, removed on clean exit; left-behind + autosave-newer → recovery offer with both timestamps |
| **Log set** (03 §3.8/§3.9.8; REQ-REL-003) | `app.log` + `app.log.1…9`, 10 MB rotation, JSON lines (`ts`,`level`,`target`,`msg`, + `job_id`/`project_id`/`code` when known) | `logging.rs` (writer) + `alvs-core::redact` | level from `alvs.advanced.logLevel`; paths debug-only and redacted; content/caption text never |
| **Migration backup** (03 §3.9.6) | `<Name>.alvs.v<old>.bak` | `project.rs` | copied before the migration driver runs; original unchanged on failure (E1005) |

**App-backend composition** (`app/src-tauri/src/project_io.rs`, D2): implements `DocumentHooks` for `alvs_model::Project` (serialize per 03 §3.2; parse+`validate`; `alvs_model::migrate` driver; media abs+rel resolution) — the only model-aware code, living where the CMP rows allow it.

## 2. Relationships

- `Settings` → consumed by autosave (interval), backup (keep-count), recent (limit), logging (level, redaction flag), paths (cache/models/default-project overrides).
- `ProjectStore` **uses** atomic-write, backup, lock, session, and recent — it is the pipeline orchestrator (03 §3.9.2); `Autosave` writes through the same atomic primitive, independently of saves.
- `Cache` owns `recent_projects` (the only table WP-03 actively uses; writers of the other tables arrive with WP-07/08/21 — D16).
- `Session marker` evaluation reads autosave/main mtimes; it does not parse documents.

## 3. Validation rules (from the spec FRs)

1. Settings: version gate, per-key kind/range/enum from the §3.6 table, one collective E8002 notice, `.bak` recovery, defaults when no backup exists (spec assumption).
2. Open pipeline: raw document length against the §3.5.15 limit *before* parse (SEC-03); migrated results validated (E1005 keeps the original); `schemaVersion` newer → read-only E1004.
3. Path safety: every write target inside project folder / cache / user library / user-picked roots; device/reserved names and UNC-from-project rejected or gated (SEC-04 → E9005) via `alvs_core::path::check_path`.
4. Lock semantics table: `{live same-host → E1007 read-only; dead → take-over; other host < 24 h → E1007; other host ≥ 24 h → take-over}`.
5. Recovery predicate: `marker_left ∧ mtime(autosave) > mtime(main)`.
6. Rotation bounds: file count ≤ 10, size trigger 10 MB (REQ-REL-003 literals).

## 4. State transitions

**Project document** (per 03 §3.9):
```
absent ──save──▶ saved ──edit──▶ dirty ──save──▶ saved
                    │                │
                    │                └─interval+dirty──▶ autosaved (sidecar)
                    ├──open(lock)──▶ open-saved / open-readonly(E1004/E1007)
                    └──crash──────▶ marker left ──startup──▶ recovery offer ─▶ open-unsaved(autosave) ─save─▶ saved
older schema ──open──▶ migrated (bak written, validated) ──save──▶ saved
newer schema ──open──▶ read-only (E1004)
```

**Settings load**: `read → parse? → version 1? → validate keys → (defaults+notice for invalid) → live` ; any parse/version failure → `restore .bak → validate → live` ; no `.bak` → `defaults + E8002`. **Write**: `change → pending(500 ms) → atomic write → refresh .bak`.

**Cache open**: `open → pragmas → migrate(tx) → quick_check? ─ok─▶ ready` ; failure → `close → rename .corrupt-<ts> → recreate → migrate → ready (+E8003)`.

**Lock**: `free ─create_new─▶ held(ours)` ; on open collision → `readonly(E1007) | takeover(dead | other-host≥24h)` ; close → removed.

**Log writer**: `app.log ─10 MB─▶ shift(.9←…←.1), delete >9, fresh app.log` (count ≤ 10 at all times).

## 5. Error surface (12 §12.13; research D4)

`StoreError` variants map 1:1 — E1004 read-only-newer, E1005 migration-failed, E1007 locked, E1008 save-failed, E8001 disk-full, E8002 settings-restored, E8003 cache-rebuilt, E8004 access-denied, E5003 write-failed (export-context wording, reserved for REQ-REL-007's second half), E9005 path-rejected — each convertible to the 04 §4.9 envelope with `messageKey = error.<code>`.
