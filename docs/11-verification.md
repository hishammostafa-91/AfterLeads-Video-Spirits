# 11 — Cross-document verification

Canonical for: consistency checks across the documentation (VER), the contradiction register, interface and security cross-checks, traceability rules, and the verification record.
Depends on: all documents. Implemented by `tools/docs/check_docs.py` (run by `/doc-check`, `tools/verify.ps1`, and the `docs` CI job).

---

## 11.1 Automated checks (VER)

| ID | Check | Rule | Tool |
|---|---|---|---|
| VER-01 | ID integrity | Every ID is defined exactly once in its canonical document (00 §0.6 definition syntax: first table cell or heading); every referenced ID (including ranges such as `REQ-EDT-001..REQ-EDT-004` or `MSK-01..03`) resolves; error codes referenced anywhere exist in 12 §12.13 | `check_docs.py` |
| VER-02 | Requirement coverage | Every P0/P1 requirement with release 1.0 lists at least one work package and one test; every P2 requirement lists none | `check_docs.py` |
| VER-03 | Traceability symmetry | A requirement's WP column matches the "Implements" lines of the WP cards in 10 §10.7, and its Tests column matches the "Covers" column of 09 §9.10 (both directions) | `check_docs.py` |
| VER-04 | Test catalog integrity | Every test ID referenced anywhere exists in 09 §9.10; every test covers at least one REQ, NFR, or THR | `check_docs.py` |
| VER-05 | Global rules sync | The G-rules block between `G-RULES:BEGIN` and `G-RULES:END` is byte-identical in `docs/00-index.md`, `AGENTS.md`, and `.specify/memory/constitution.md` | `check_docs.py` |
| VER-06 | No placeholders | No unfinished markers (`TODO`, `TBD`, `FIXME` written outside code spans) in documentation | `check_docs.py` |
| VER-07 | Identity | The product name is never written as two words without the `AfterLeads-` prefix; every repository URL equals the 00 §0.5 constant | `check_docs.py` |
| VER-08 | No traces of other products | Repository text contains none of the denied tokens; the list is stored only as SHA-256 hashes in `tools/docs/denylist.sha256` so the names themselves never appear in the repository | `check_docs.py` |
| VER-09 | Component dependency rules | Crate dependencies follow 02 §2.4 | `tools/check_deps.py` (from WP-01) |
| VER-10 | Links | Relative Markdown links and `#anchors` resolve | `check_docs.py` |
| VER-11 | Shortcut conflicts | The default keymap in 12 §12.10 has zero conflicts per context | `check_docs.py` |
| VER-12 | Catalog ↔ manifests | Effect/transition/generator manifests equal the catalog (12) | T-FX-004 (from WP-14) |
| VER-13 | Locale consistency | All six locale files have identical keys and valid ICU syntax | `pnpm -C app i18n:check` (from WP-04) |
| VER-14 | Threat coverage | Every THR row lists at least one SEC control and one test | `check_docs.py` |

Adding a word to the VER-08 denylist: run `python tools/docs/check_docs.py --hash-word <word>` and append the printed hash to `tools/docs/denylist.sha256` in a docs pull request.

---

## 11.2 Contradiction register

Facts that appear in more than one document. Each row names the canonical source; reviewers confirm all other mentions agree whenever one of them changes.

| ID | Fact | Canonical | Also stated in |
|---|---|---|---|
| C-01 | Time unit = flicks (705,600,000 per second) | 03 §3.3.1 | 02 ADR-008, 04 §4.4, 00 §0.9 |
| C-02 | Autosave default 60 s; backups keep 20 | 03 §3.6 | 01 REQ-PRJ-003/004 |
| C-03 | Six locales en, ar, fr, es, de, pt-BR | 12 §12.14 | 00 OD-03, 01 REQ-I18N-001, 07 §7.12, AGENTS.md |
| C-04 | Working color space Rec.709 display-referred, RGBA16F | 02 ADR-013 | 01 REQ-COL-003, 03 §3.4.5 |
| C-05 | Parallel job limit default 2 (+ probe) | 04 §4.6.4 | 03 §3.6 |
| C-06 | Automatic proxies above 1440 lines, 720p proxies | 03 §3.6 | 01 REQ-MED-005 |
| C-07 | Default export preset XP-01 (`xp.h264.high`) | 12 §12.7 | 03 §3.6 |
| C-08 | Default speech model MDL-03 (`small-q5_1`) | 00 OD-09 | 03 §3.6, 12 §12.15 |
| C-09 | Branch naming pattern | 10 §10.4.2 | `.githooks/pre-push`, `tools/git/New-WorkBranch.ps1`, AGENTS.md |
| C-10 | Merge commits only; owner merges | 00 G-05, OD-10 | 08 OPS-02, 10 §10.4.7, AGENTS.md, constitution |
| C-11 | TypeScript 6.0.x and ESLint 9.x | 00 §0.8 | 02 ADR-011 |
| C-12 | Static CRT; vcpkg triplet `x64-windows-static` | 02 ADR-004 | 10 WP-00, WP-06 |
| C-13 | Required CI checks | 08 OPS-02 | 08 §8.3.1 |
| C-14 | Caption readability defaults (42 chars, 2 lines, 0.7–7 s, 2-frame gap) | 03 §3.6 | 01 REQ-CAP-007, 06 §6.3 |
| C-15 | Product identity constants | 00 §0.5 | README, SECURITY.md, TRADEMARKS.md, 08 §8.6 |
| C-16 | Update endpoints on GitHub Pages | 08 §8.8 | 05 SEC-09/SEC-10 |
| C-17 | Minimum window 1280×720 | 01 REQ-APP-001 | 07 §7.2 |

---

## 11.3 Interface cross-checks (manual, at each documentation change)

1. Every UI action in 07 (panels, dialogs, context menus) maps to a command in 12 §12.10 or a backend command in 04 §4.3.
2. Every edit requirement (EDT, FX, TRN, TTL, CAP, MED add/remove) maps to an `EditOp` in 04 §4.4.
3. Job kinds in 04 §4.6.3 equal the list in REQ-JOB-001 and the `analysis`/`jobs` usages in 03 §3.7.
4. Every persisted field in 03 §3.4 is reachable through an `EditOp` or a documented non-undoable command.
5. Settings keys in 03 §3.6 each have a UI location (DLG-05 page or View menu) and a consumer described in 02/04.

## 11.4 Security cross-checks

1. VER-14 passes (every threat has controls and tests).
2. Every new parser has a fuzz target (SEC-05) and size limits.
3. Every new network use is covered by SEC-10 and listed in NFR-PRIV-01.
4. Every new Tauri permission is listed in 05 §5.4.

## 11.5 Traceability rules

- Requirement → work package → test is recorded in 01 §1.6 (WP and Tests columns), 10 §10.7 (Implements/Acceptance), and 09 §9.10 (Covers). VER-02..VER-04 keep them consistent.
- Spec Kit artifacts (`specs/0NN-*`) cite the same IDs; `/speckit.analyze` flags tasks without a requirement.
- Pull requests list implemented REQ IDs and passed T IDs (G-09).

## 11.6 Verification record

| Date | Document set version | Result | By |
|---|---|---|---|
| 2026-11-01 | 1.0.0 | `check_docs.py`: all automated checks pass (VER-01..VER-08, VER-10, VER-11, VER-14); manual cross-checks 11.2–11.4 reviewed | Documentation baseline |
| 2026-11-02 | 1.0.1 | Setup order fixed (10 §10.2.2: Git and PowerShell 7 first, clone before running repository scripts); `main` bootstrap moved to GitHub before cloning (10 §10.4.14); OPS-02 required checks staged after WP-01 (08 §8.2); `uv tool update-shell` added so `specify` is on PATH (06 §6.11, 10 §10.2.2); only tool installation runs elevated, so git never sees an Administrators-owned clone (10 §10.2.2); `check_docs.py`: all automated checks pass | Documentation patch |

Each documentation pull request appends a row.
