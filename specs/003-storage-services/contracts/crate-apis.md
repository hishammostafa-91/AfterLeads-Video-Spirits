# WP-03 Contracts — `alvs-store` public API

Rust signatures are the planning sketch; exact names are finalized in implementation and enforced by tests. Every item cites its canonical source. `#![deny(missing_docs)]` is not required on `alvs-store` (docs/02 §2.4 rule 4 lists core/model/formats) but the crate documents its public items.

## 1. `alvs-core` additions (shared catalog, CMP-01)

```rust
// error.rs — seven new catalog codes with 12 §12.13 severities (research D4)
pub enum ErrorCode { /* …WP-02 set… */ E1007, E1008, E5003, E8001, E8002, E8003, E8004 }

// redact.rs — CMP-01 "logging setup helpers" (research D15; REQ-REL-003)
pub fn redact_paths(text: &str) -> String;   // drive-letter, UNC, env-root paths → "<path>"
```

## 2. `alvs-store` modules (CMP-08; deps: `alvs-core` + baseline externals only)

```rust
// paths.rs — 03 §3.8
pub struct AppPaths { /* all canonical roots as PathBuf */ }
impl AppPaths {
    pub fn resolve() -> Result<Self, StoreError>;        // env roots + identifier folder
    pub fn resolve_portable(exe_dir: &Path) -> Result<Self, StoreError>; // portable.flag → <exe>\data
    pub fn logs_dir(&self) -> &Path;  pub fn cache_db(&self) -> &Path;
    pub fn settings_file(&self) -> &Path;  /* …one accessor per 03 §3.8 row… */
}

// error.rs — 02 §2.11; one variant ↔ one code; 04 §4.9 envelope conversion
pub enum StoreError { /* ReadOnlyNewer, MigrationFailed, Locked, SaveFailed,
                         DiskFull, SettingsRestored, CacheRebuilt, AccessDenied,
                         WriteFailed, PathRejected, … */ }
impl StoreError { pub fn code(&self) -> alvs_core::error::ErrorCode;
                  pub fn envelope(self) -> alvs_core::error::ErrorEnvelope; }

// atomic.rs — 03 §3.9.1 (research D12/D13)
pub fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), StoreError>;
// tmp-in-folder → sync_all(=FlushFileBuffers) → MoveFileExW(REPLACE|WRITE_THROUGH);
// original kept on failure; Fs seam is pub(crate) for step-wise failure injection.

// settings.rs + keys.rs — 03 §3.6; REQ-SET-003
pub struct Settings { /* document + notices */ }
impl Settings {
    pub fn load(paths: &AppPaths) -> (Settings, Vec<Notice>);   // validate/restore/.bak per D7
    pub fn get<T: FromSetting>(&self, key: SettingKey) -> T;    // typed getters for consumed keys
    pub fn set(&mut self, key: SettingKey, value: Value);       // debounced write on flush
    pub fn flush(&mut self) -> Result<(), StoreError>;          // + refresh settings.json.bak
}

// cache.rs + migrations/ — 03 §3.7
pub struct Cache { /* rusqlite Connection */ }
impl Cache {
    pub fn open(paths: &AppPaths, settings: &Settings) -> Result<Self, StoreError>;
    // pragmas + embedded NNN migrations in one tx + quick_check/rebuild (E8003)
    pub fn recent(&self) -> RecentProjects;                     // handle below
}

// recent.rs — 03 §3.7/§3.9.2
pub trait RecentProjects {
    fn record(&self, path: &Path, name: &str, thumb: Option<&Path>) -> Result<(), StoreError>;
    fn list(&self) -> Result<Vec<RecentEntry>, StoreError>;      // newest-first
    fn set_pinned(&self, path: &Path, pinned: bool) -> Result<(), StoreError>;
}   // record trims to alvs.general.recentLimit

// project.rs — 03 §3.9.2/§3.9.6; REQ-PRJ-002 (research D2)
pub trait DocumentHooks {
    type Doc;
    fn serialize(doc: &Self::Doc) -> Result<Vec<u8>, StoreError>;
    fn parse_and_validate(bytes: &[u8]) -> Result<Self::Doc, StoreError>;
    fn migrate(bytes: &[u8]) -> Result<(Vec<u8>, bool), StoreError>; // migrated?, E1005
    fn fixup_media_paths(doc: &mut Self::Doc, project_dir: &Path) -> Result<(), StoreError>;
}
pub struct ProjectStore { /* … */ }
impl ProjectStore {
    pub fn save(&self, doc: &impl DocumentHooks::Doc /* via H */) -> Result<SaveReceipt, StoreError>;
    // §3.9.2: serialize (caller thread) → atomic write → backup → prune →
    // delete sidecar → recent.record
    pub fn open(path: &Path) -> Result<Opened<H::Doc>, StoreError>;
    // exist → size gate → parse → migrate(.bak first) → validate → lock → fixup media
    pub fn backups(&self) -> Result<Vec<BackupEntry>, StoreError>;   // time + size
    pub fn restore_backup(&self, which: &BackupEntry) -> Result<PathBuf, StoreError>;
}

// autosave.rs — 03 §3.9.3; REQ-PRJ-003; NFR-REL-02 (research D10)
pub trait Clock { fn now_ms(&self) -> i64; }
pub struct Autosave { /* pure decision core + driver */ }
impl Autosave {
    pub fn decision(dirty: bool, since_last_ms: i64, interval_sec: u32) -> AutosaveDecision;
    pub fn spawn(interval_sec: u32, clock: Arc<dyn Clock>, writer: …) -> Self; // production
}

// lock.rs — 03 §3.9.4; REQ-PRJ-011 (research D11)
pub enum LockOutcome { Acquired, ReadOnly, TakeoverAvailable }
impl LockFile {
    pub fn acquire(project: &Path) -> Result<(Self, LockOutcome), StoreError>;
    pub fn release(self) -> Result<(), StoreError>;               // removed on close
}

// session.rs — 03 §3.9.5; REQ-REL-001 services (research D14)
pub struct SessionMarker;
impl SessionMarker {
    pub fn create(paths: &AppPaths) -> Result<Self, StoreError>;
    pub fn clear(self) -> Result<(), StoreError>;                 // clean exit
    pub fn evaluate(main: &Path, autosave: &Path) -> Option<RecoveryOffer>;
}

// logging.rs — 02 §2.11; 03 §3.9.8; REQ-REL-003 (research D3)
pub struct LogGuard { /* WorkerGuard */ }
pub fn init_logging(paths: &AppPaths, level: LogLevel) -> Result<LogGuard, StoreError>;
// fmt().json() + UtcTime::rfc_3339 + custom 10 MB × 10 rotating writer + non_blocking
pub struct SizeRotatingWriter { /* … */ }                          // unit-testable directly
```

## 3. App-backend composition (CMP-12; research D2)

`app/src-tauri/src/project_io.rs`: `struct ProjectHooks;` implementing `DocumentHooks` with `Doc = alvs_model::Project` — `to_string` per 03 §3.2, `serde_json` parse + `alvs_model::validate::validate`, `alvs_model::migrate` driver + result validation, media `path`/`rel_path` resolution (absolute-first-fallback-relative per REQ-PRJ-002: relative tried when absolute missing). Thin — no logic beyond delegation.

## 4. Behavioral guarantees (test-enforced)

1. **G1** — a failed write at *any* step leaves the previous file byte-identical (T-PRJ-002, T-REL-007; Fs injection + real read-only dir).
2. **G2** — autosave never writes the main file and fires exactly on `dirty ∧ elapsed ≥ interval` (T-PRJ-003; fake clock).
3. **G3** — settings invalid values revert to defaults with one E8002, unknown keys survive round-trip, corruption recovers from `.bak` (T-SET-003).
4. **G4** — locks: live-same-host → E1007; dead → takeover; other-host < 24 h → no takeover (T-PRJ-008).
5. **G5** — cache: migrations transactional + versioned; corrupt DB rebuilt with E8003; recent list newest-first and trimmed (unit/integration).
6. **G6** — logs: ≤ 10 files, 10 MB rotation threshold, JSON fields present, paths redacted (T-REL-003).
7. **G7** — SEC-04 containment on every write path (unit, all services).
