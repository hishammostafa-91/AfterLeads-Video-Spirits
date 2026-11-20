# Third-Party Notices

This is the project's notices manifest (G-12; docs/12 §12.16): every third-party asset that ships inside the application is recorded here with its copyright line and license. Work packages that add bundled assets extend this file in the same pull request. Once release packaging gains a generator, this file is its seed and source of truth (docs/02 §2.14 — generated files are never hand-edited after that point). Runtime and development npm dependencies are license-checked separately by `tools/check_npm_licenses.py` against the docs/05 §5.6 allowlist and are not duplicated here.

## Bundled fonts (docs/12 §12.16; added by WP-04)

Bundled as separate files under `assets/fonts/`, each with its license file; file names, source URLs, and SHA-256 hashes are recorded in `assets/fonts/fonts.json`.

| ID | Family | Copyright | License |
|---|---|---|---|
| FNT-01 | Inter (variable) | Copyright (c) 2016 The Inter Project Authors (Rasmus Andersson) | SIL Open Font License 1.1 |
| FNT-02 | Noto Sans Arabic (variable) | Copyright (c) 2015-2024 The Noto Project Authors (Google LLC) | SIL Open Font License 1.1 |
| FNT-03 | JetBrains Mono | Copyright (c) 2020 JetBrains s.r.o. | SIL Open Font License 1.1 |

The full SIL Open Font License 1.1 text for each family ships next to its font files (`assets/fonts/`, per `fonts.json`'s `licenseFile` entries).

## Iconography (docs/07 §7.5; added by WP-04)

| Asset | Copyright | License |
|---|---|---|
| Lucide base icon set (via `lucide-react`) | Copyright (c) 2020 Lucide Contributors | ISC License |

Custom product icons (docs/07 §7.5 list) are original works of this project (AGPL-3.0-or-later) and need no notice entry.
