# 05 — Security

Canonical for: security objectives, trust boundaries, threat model (THR), security controls (SEC), webview hardening, dependency and license policy, update and release integrity, privacy, vulnerability handling.
Depends on: [02-architecture.md](02-architecture.md), [03-data.md](03-data.md), [04-interfaces.md](04-interfaces.md), [08-operations.md](08-operations.md).

---

## 5.1 Objectives and assumptions

**Objectives**

1. Opening any media, project, caption, LUT, or library file can at worst fail with an error; it cannot execute code, read or write files outside the user's intent, or contact the network.
2. Updates and downloaded models cannot be tampered with.
3. No personal data leaves the machine (NFR-PRIV-01).
4. Contributions and dependencies cannot silently introduce malicious code or incompatible licenses.

**Assumptions**

- The Windows user account is the security boundary; other processes of the same user are trusted at the OS level (they could already modify the per-user installation).
- The application runs without administrator rights and installs per user.
- The WebView2 runtime and the GPU driver are kept updated by Windows Update.

---

## 5.2 Assets

| Asset | Why it matters |
|---|---|
| User projects, media, recordings | Irreplaceable creative work |
| Update signing key (Ed25519) | Controls code that runs on every user's machine |
| Repository integrity (`main`, release tags) | Source of every build |
| User privacy (paths, names, content) | No telemetry promise |

---

## 5.3 Trust boundaries

```text
 untrusted files ─► [ worker process ]──JSON Lines──► [ GUI core (Rust) ] ◄──typed IPC──► [ WebView2 UI ]
 (media, captions,     probe, decode,                    validation, path            CSP, no remote
  LUTs, projects)      transcode, export                 policy, model actor         content
                                                                 │
 network (GitHub Releases, Hugging Face) ◄── allowlisted HTTPS ──┘ (updates, model downloads only)
```

Rules: every arrow into the core is validated; nothing from a file or the network reaches a shell, a scripting engine, or a dynamic library loader.

---

## 5.4 Webview hardening (Tauri)

| Setting | Value |
|---|---|
| Content Security Policy | `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' asset: http://asset.localhost data: blob:; font-src 'self' asset: http://asset.localhost; connect-src ipc: http://ipc.localhost; media-src 'self' blob:; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'` (Tauri adds hashes/nonces for its own injected code). A dependency needing more MUST be justified by an ADR. |
| Remote content | None. The webview only loads the bundled UI. Navigation to any other origin is cancelled; `window.open` is denied; external links go through `system_open_url` (SEC-09). |
| DevTools | Disabled in release builds. |
| Capabilities | Main window: the application's own commands; `dialog:allow-open`, `dialog:allow-save`, `dialog:allow-message`, `dialog:allow-ask`; updater (`updater:default`); `core:event:default`, `core:window:default` (minimize/maximize/close/drag for the custom title bar). **Not granted:** fs, shell, http, process plugins. |
| Asset protocol | Enabled with scope limited to the cache thumbnail folders (`$APPLOCALDATA/cache/thumbs/**`) and bundled `assets/` (fonts, help). |
| HTML injection | `dangerouslySetInnerHTML` is banned by lint; the help viewer uses `react-markdown` with raw HTML disabled; user text (names, captions, notes) is always rendered as text. |

---

## 5.5 Threats (THR) and controls

| ID | Threat | Controls | Tests |
|---|---|---|---|
| THR-01 | A malicious media file exploits a decoder | SEC-01, SEC-02, SEC-12, SEC-14; residual risk: in-process playback decoding of files that passed probing — mitigated by autosave and crash recovery (REQ-REL-001) | T-SEC-003 |
| THR-02 | A malicious project/library file (huge structures, path traversal, device or network paths) | SEC-03, SEC-04, SEC-06 | T-SEC-003, T-PRJ-007 |
| THR-03 | Malicious caption/LUT/CSV files trigger parser bugs | SEC-05, size limits (4.11) | T-SEC-002 |
| THR-04 | Script injection in the UI leads to arbitrary IPC calls | SEC-17, SEC-06, SEC-07 | T-SEC-005 |
| THR-05 | Compromised dependency or CI action | SEC-12, SEC-19 | T-SEC-001 |
| THR-06 | Tampered update | SEC-16 (Ed25519 signature verified by the updater with the embedded public key; HTTPS) | T-UPD-001 |
| THR-07 | Tampered speech model | SEC-11 | T-STT-002 |
| THR-08 | Privacy leakage (telemetry, logs, network) | SEC-10, SEC-20, 3.11 | T-SEC-004 |
| THR-09 | DLL planting | SEC-14 | T-SEC-005 |
| THR-10 | Leaked secrets (signing key, tokens) | SEC-15 | T-SEC-006 |
| THR-11 | Malicious pull request | SEC-19, review checklist 08 OPS-04 | T-SEC-007 |
| THR-12 | Decompression bombs / huge images exhaust memory | SEC-02 dimension limits, SEC-03 | T-SEC-003 |
| THR-13 | Credential leak through UNC paths in a shared project (Windows sends NTLM credentials to SMB servers) | SEC-04 asks before touching any `\\server\share` path from a project file | T-SEC-003 |

---

## 5.6 Dependency and license policy

**License allowlist** (enforced by `cargo deny check licenses` and `tools/check_npm_licenses.py`):

`MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `0BSD`, `Unicode-3.0`, `Unicode-DFS-2016`, `BSL-1.0`, `MPL-2.0`, `CC0-1.0`, `Unlicense`, `WTFPL`, `CDLA-Permissive-2.0`, `LGPL-2.1-or-later`, `LGPL-2.1-only`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later`, `AGPL-3.0-or-later`; fonts `OFL-1.1` (bundled as separate files); documentation/data `CC-BY-4.0` with attribution.

**Denied:** `GPL-2.0-only` (incompatible with version 3), `SSPL`, `BUSL`, Commons Clause, any `CC-BY-NC*`/`CC-BY-ND*`, proprietary or missing licenses. FFmpeg MUST NOT be built with `nonfree` features.

**Rules**

1. New dependencies require an ADR approved by the owner (G-11) and must appear in 00 §0.8.
2. Lockfiles are committed; CI installs with `--locked` (cargo) and `--frozen-lockfile` (pnpm).
3. `cargo deny` checks advisories, licenses, bans (duplicate major versions reviewed), and sources (crates.io only).
4. pnpm runs install scripts only for packages in `pnpm.onlyBuiltDependencies` (reviewed list).
5. GitHub Actions are pinned to full commit SHAs; Dependabot proposes updates weekly for cargo, npm, and actions.
6. vcpkg uses a pinned `builtin-baseline`; port features are listed in ADR-004.

---

## 5.7 Security controls (SEC)

| ID | Control |
|---|---|
| SEC-01 | **Worker isolation:** probe, thumbnails, waveforms, proxies, optimize, stabilize, loudness, transcription, and export run in separate worker processes (02 §2.3). A file is added to a project only after a successful worker probe. |
| SEC-02 | **FFmpeg input hardening:** every `avformat_open_input` sets `protocol_whitelist=file`; inputs detected as `hls`, `dash`, `concat`, `ffconcat`, `rtsp`, `sdp`, `tee`, or other playlist/reference demuxers are rejected (E2002); MOV external data references stay disabled; image sequences use only patterns generated by the application; maximum 16,384 × 16,384 pixels and 64 streams per file; probe size and analyze duration are bounded. |
| SEC-03 | **Structured file limits:** project, settings, library, and job files are size-limited (03 §3.5 rule 15, 3.10), schema-validated, and rejected on violation. |
| SEC-04 | **Path policy** (`alvs-core::path`): canonicalize; reject Windows device paths (`\\.\`, `\\?\GLOBALROOT`, reserved names `CON`, `NUL`, `AUX`, `PRN`, `COM1–9`, `LPT1–9`); never write outside the project folder, the cache, the user library, or a folder the user picked; UNC paths from a project file require explicit confirmation before access. |
| SEC-05 | **Parser fuzzing:** `cargo fuzz` targets for SRT, WebVTT, ASS, `.cube`, marker CSV, project JSON, settings JSON, and job JSON run nightly on a Linux runner (pure-Rust crates) for 10 minutes each; property tests run on every pull request. |
| SEC-06 | **IPC validation:** every command checks types, ID existence, string lengths (names ≤ 200, text ≤ 10,000, paths ≤ 32,767 characters), and numeric ranges; violations return E9006 and are logged. |
| SEC-07 | **No generic file API:** the UI never gets file-system access; paths come from dialogs or from validated project data, and backend commands re-check them with SEC-04. |
| SEC-08 | **Asset protocol scope** limited per 5.4. |
| SEC-09 | **URL allowlist** for `system_open_url`: the repository (`https://github.com/hishammostafa-91/AfterLeads-Video-Spirits` and sub-paths), its GitHub Pages site, and `https://huggingface.co/ggerganov/whisper.cpp`; anything else is rejected (E9004). |
| SEC-10 | **Network guard:** model downloads use one HTTP module (`alvs-stt::download`) that only contacts hosts ending in `huggingface.co` or `hf.co` over HTTPS; the updater only uses the configured GitHub Releases endpoint; the UI's CSP blocks all web requests; no other crate may depend on an HTTP client (cargo-deny `bans`). |
| SEC-11 | **Model integrity:** pinned SHA-256 and size (12 §12.15) compiled into the app; verification before activation; mismatches are deleted (E7003). |
| SEC-12 | **Patch policy:** security advisories for shipped dependencies are fixed within 7 days (critical) or 14 days (high) through a patch release. |
| SEC-13 | **No runtime third-party code:** release 1.x before REQ-AUD-012 loads no plugins, scripts, or remote web content. |
| SEC-14 | **Binary hardening:** static CRT; `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)` is called first in `main`; Rust built with `-C control-flow-guard`; ASLR and DEP are the MSVC defaults and are verified on the shipped binary (T-SEC-005). |
| SEC-15 | **Secret hygiene:** no secrets in the repository; GitHub secret scanning and push protection on; the pre-commit hook rejects private-key blocks and `.env` files; the update signing key exists only as an encrypted GitHub Actions secret and in the owner's offline backup. |
| SEC-16 | **Release integrity:** Ed25519-signed update artifacts and manifest; SHA-256 checksum file; GitHub build-provenance attestations; annotated release tags signed by the owner's SSH signing key (08 OPS-08). |
| SEC-17 | **Webview hardening** per 5.4. |
| SEC-18 | **Least privilege:** per-user install, no administrator rights, no services, no scheduled tasks, no auto-start. |
| SEC-19 | **Contribution security:** protected `main` (08 OPS-02), required CI, owner review of every pull request, DCO, workflows for pull requests from forks run without secrets (`pull_request`, never `pull_request_target` with checkout of untrusted code). |
| SEC-20 | **Privacy defaults:** no telemetry, no crash upload, update checks can be disabled, diagnostics are previewed and redacted before saving. |

---

## 5.8 Privacy statement (shown in DLG-19)

AfterLeads-Video-Spirits does not collect or send any data about you or your projects. It connects to the internet only to check for updates (you can turn this off) and to download speech models when you ask it to. Crash information stays on your computer; you decide whether to share it in a problem report.

---

## 5.9 Update and release integrity

1. The owner generates the updater key pair once with `pnpm tauri signer generate` (08 OPS-09) on the development machine; the public key goes into `tauri.conf.json`; the private key and its password become GitHub Actions secrets; an encrypted offline backup is kept by the owner.
2. The release workflow signs each installer and the `latest.json` manifest; the updater rejects unsigned or mismatched packages (E9002).
3. Key rotation: publish a release signed with the old key whose new version embeds the new public key; after one release cycle, retire the old key.
4. Compromise response: remove the affected release, publish a security advisory, rotate the key, and release a fixed version (08 OPS-12).

---

## 5.10 Vulnerability handling

- Reports go to the email in `SECURITY.md` or GitHub private vulnerability reporting (free).
- Acknowledge within 7 days, fix per SEC-12, publish a GitHub Security Advisory, credit the reporter if desired.
- Supported versions: the latest minor release receives security fixes.

---

## 5.11 Security rules for agents

1. Never add, print, or commit secrets, tokens, or private keys; never paste them into prompts.
2. Never download and execute scripts (no `curl | iex`); installing tools follows 10 §10.2 only.
3. Treat every file the app opens as hostile; new parsers need fuzz targets (SEC-05) and size limits.
4. Never weaken CSP, capabilities, path policy, or network guard without an approved ADR.
5. Security-relevant pull requests must list the THR/SEC IDs they touch.
