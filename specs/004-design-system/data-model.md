# WP-04 Data Model — entities, rules, and the story matrix

This package owns presentation entities; there is no persisted database. Field types are TypeScript sketches (finalized in implementation and enforced by tests); every rule traces to the spec's FRs.

## Entities

### 1. Design tokens (`app/src/design/tokens.css`)

| Field | Type | Rule |
|---|---|---|
| `name` | CSS custom property | Exactly the 07 §7.4 names (`--bg-app` … `--tl-offline`, type/space/radius/elevation/motion tokens); components may reference only these (FR-001, lint-enforced) |
| `dark` / `light` | color/token value | §7.4.1/§7.4.2 values verbatim; contrast against the usual background meets 4.5:1 text / 3:1 components (FR-013) |
| `highContrast` | system color | §7.4.3 mapping under `forced-colors: active` only |
| `timeline` | dark/light pair | §7.4.4 table verbatim; label colors CLR-01..08 override bodies, never the type icon |

State transitions: none (static sheet); the *active values* change with `data-theme` / `data-accent` / forced-colors (Theme below).

### 2. Theme (`themeStore`)

| Field | Type | Rule |
|---|---|---|
| `theme` | `"dark" \| "light" \| "high-contrast"` | dark default; switch applies without restart; `high-contrast` applies the §7.4.3 system-color mapping for manual selection (owner ruling F05) |
| `accent` | `"clr-06" \| "clr-07" \| "clr-04" \| "clr-02" \| "clr-08"` | CLR-06 default; swaps the three accent tokens per §7.4.1 values (FR-003) |
| `zoom` | integer 80–200 | clamped; applied as CSS `zoom` on the root (FR-004, D3) |
| `forcedColors` | boolean (read-only signal) | follows the OS media query; when active the token mapping of §7.4.3 governs regardless of `theme` |

Transitions: `(theme, accent, zoom) →(set)→ same` with immediate `<html>` attribute/style application; persistence through the storage seam (D12). Validation: zoom outside 80–200 is clamped, never rejected (REQ-APP-010's range is the control's range).

### 3. Locale catalog (`app/src/locales/`)

| Field | Type | Rule |
|---|---|---|
| `tag` | `"en" \| "ar" \| "fr" \| "es" \| "de" \| "pt-BR"` | LOC-01..06, shipped (12 §12.14) |
| `keys` | flat map of ICU strings | identical key sets across all six files; IDs `area.component.purpose`; ICU plurals/select incl. Arabic's six categories (FR-010/FR-011) |
| `meta` | `en.meta.json` | one description per key; optional max-length for space-limited labels (07 §7.12 rule 4) |
| `pseudo` | `"en-XA" \| "ar-XB"` | generated at test time only; never shipped (D14) |

Validation (FR-011): key-set equality (missing or extra ⇒ `i18n:check` exit ≠ 0); every key has meta; brand strings never translated (§7.12 rule 7).

### 4. Component (`app/src/components/*`)

| Field | Type | Rule |
|---|---|---|
| `name` | 07 §7.6 inventory | all components delivered; RAC-based unless listed as composed (D6) |
| `props` | per-component contract | [contracts/design-system-apis.md](contracts/design-system-apis.md) |
| `styling` | CSS Module | tokens + logical properties only (FR-016) |
| `a11y` | name/role/value/state | keyboard-operable; visible focus; announced state changes (FR-012) |

### 5. Story and the snapshot matrix (harness)

| Field | Type | Rule |
|---|---|---|
| `states` | default, hover, focus-visible, pressed, disabled, error, RTL, compact, high-contrast | every component has all nine (FR-007) |
| `axes` | theme ∈ {dark, light} × locale ∈ {en, ar} | every state snapshotted in all four combinations |
| `axe` | zero violations | per story, per combination (SC-006) |

Determinism rules: viewport 1280×720, `reducedMotion: reduce`, fonts awaited, animations disabled (D7).

### 6. Font bundle (`assets/fonts/fonts.json`)

| Field | Type | Rule |
|---|---|---|
| `id`/`family` | FNT-01..03 | Inter variable, Noto Sans Arabic variable, JetBrains Mono (12 §12.16) |
| `file` + `licenseFile` | path | separate files; OFL 1.1 license present per family |
| `sourceUrl` + `sha256` | string | recorded at bundling; integrity check re-hashes and fails on mismatch (FR-005) |

## Relationships

- Theme → Tokens: selects the active value block (`data-theme`, `data-accent`, forced-colors).
- Component → Tokens: reads custom properties; never raw values.
- Component → i18n runtime: all user-visible strings via message IDs; `dir`/`lang` from `direction.ts`.
- Story → Component × State × Theme × Locale: the full matrix is the acceptance artifact.
- Harness → Mock fixtures: typed shapes from `src/ipc/bindings` (no IPC calls).

## Validation summary

1. Key-set equality across locales (i18n:check). 2. Token-only styling (Stylelint). 3. No JSX literals / no `any` (ESLint + tsc). 4. Contrast ratios (harness axe + token values). 5. Zoom clamp 80–200. 6. Font hashes match the manifest. 7. Zero axe violations per story. 8. Snapshot determinism (repeat-run equality).
