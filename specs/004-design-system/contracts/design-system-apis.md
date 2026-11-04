# WP-04 Contracts — the design system's public surface

TypeScript signatures are the planning sketch; exact names finalize in implementation and are enforced by tests. Every item cites its canonical source. The UI gates (02 §2.14) are enforced from this package on.

## 1. Theme and zoom (`app/src/design/theme.ts`)

```ts
type ThemeName = 'dark' | 'light';               // REQ-APP-009; dark default
type Accent = 'clr-06' | 'clr-07' | 'clr-04' | 'clr-02' | 'clr-08';  // 12 §12.17
type Zoom = number;                               // clamped 80..200 (REQ-APP-010)

interface ThemeStore {
  theme: ThemeName; accent: Accent; zoom: Zoom;
  setTheme(t: ThemeName): void;                   // applies without restart
  setAccent(a: Accent): void;                     // swaps the 3 accent tokens
  setZoom(z: number): void;                       // clamps, applies CSS zoom
  forcedColorsActive(): boolean;                  // §7.4.3 signal
}
// Persistence seam (D12): { read(): Partial<...>; write(s): void } —
// the harness binds URL params; WP-05 binds alvs.appearance.* keys.
```

## 2. i18n runtime (`app/src/i18n/`)

```ts
type LocaleTag = 'en' | 'ar' | 'fr' | 'es' | 'de' | 'pt-BR';   // LOC-01..06
// runtime.tsx: <I18n> wraps react-intl's IntlProvider; language switch
// is a store update — no reload (REQ-I18N-001).
function resolveDefaultLocale(navigator: readonly string[]): LocaleTag;  // Windows display language → match, else 'en'
// direction.ts (07 §7.11): applies dir/lang to <html>;
// components with non-mirrored surfaces (timeline axis, ruler, transport,
// timecode) set dir="ltr" explicitly with a comment citing §7.11.
// format.ts (REQ-I18N-003): Intl number/date with numberingSystem 'latn';
// ltrIsolate(text) → `<bdi dir="ltr">` wrapper helper for timecode/frames.
// pseudo.ts (LOC-90/91, tests only): pseudoEn(en), pseudoAr(ar).
```

## 3. Components (`app/src/components/`)

Inventory = 07 §7.6 exactly; RAC-based unless noted (D6). Shared prop conventions: every user-visible string is a message ID; every interactive element takes `isDisabled`, forwards refs, and is keyboard-operable; sizes come from tokens (28/24 px controls, 24×24 px minimum target).

| Component | Key contract points (from 07 §7.6) |
|---|---|
| Button / IconButton / ToggleButton / SegmentedControl | variants primary/secondary/ghost/danger; primary once per dialog; danger asks confirmation for irreversible actions; IconButton = icon + tooltip(label+shortcut) + accessible name (REQ-HLP-003) |
| MenuBar / Menu / ContextMenu / Submenu | shortcuts right-aligned (left in RTL); checkable items |
| Tabs | panel-tab primitive (dockview integration is WP-05) |
| Dialog / AlertDialog / Popover / Tooltip / Toast | focus trap, Esc closes (except unsaved-changes prompts); tooltip 500 ms delay; toasts also target the notification center (region) |
| NumberField | scrub (Shift ×10, Ctrl ×0.1), wheel-when-focused, arrows, typed math (`+10`, `*2`), unit suffix, mixed state "—" |
| Slider / AngleDial | paired NumberField; double-click resets |
| ColorPicker / ColorWheel | HSV + hue + hex/RGB + recent colors / chroma puck + luma; eyedropper and viewer sampling land with WP-11+ (no viewer yet) |
| CurveEditor / PointControl | add/move/delete points, channel tabs, reset / numeric x/y + handle slot (viewer overlay later) |
| TimecodeField | segmented `HH:MM:SS:FF`, typed + `+`/`−` relative entry, always LTR (§7.11) |
| KeyframeToggle / KeyframeNavigator | stopwatch state; ◀ ◆ ▶ with enabled states |
| SearchField / TextField / TextArea | `dir="auto"` for user content |
| Checkbox / Switch / RadioGroup / Select / ComboBox | RAC semantics |
| Tree / GridList / Table / ListBox | windowed beyond 200 rows (`useWindowing`, D10) |
| ProgressBar / Meter / Spinner / Skeleton | indeterminate only before the first progress value |
| Badge | text + icon state chips (Proxy, Offline, 10-bit, HDR, VFR); never color alone |
| EmptyState | icon, one sentence, primary action, shortcut hint |

## 4. Harness URL and scripts

```text
/harness?theme=dark|light&locale=en|ar|fr|es|de|pt-BR|en-XA|ar-XB&density=comfortable|compact
```

- `pnpm -C app typecheck` → `tsc --noEmit` (strict, no `any`).
- `pnpm -C app lint` → ESLint (all rule families) + Stylelint + `prettier --check`.
- `pnpm -C app test` → Vitest (+ font-manifest integrity re-hash).
- `pnpm -C app i18n:check` → locale key-set equality + meta coverage.
- `pnpm -C app harness` → Playwright: snapshots (dark/light × en/ar, all stories) + per-story axe.

These first four script names are what `tools/verify.ps1` probes; defining them activates the `ui-*` and `i18n` gates.

## 5. Schemas

- `assets/fonts/fonts.json`: `{ "fonts": [{ id, family, file, sourceUrl, sha256, licenseFile }] }` (12 §12.16).
- `en.meta.json`: `{ "<message id>": { "description": string, "maxlength?"?: number } }` (07 §7.12 rule 4).

## 6. Behavioral guarantees (test-enforced)

1. **G1** — theme/accent/zoom changes apply without restart and propagate to every token consumer (T-UI-001).
2. **G2** — Dark and Light meet WCAG 2.2 AA (4.5:1 text, 3:1 components/focus); High Contrast maps via forced colors (T-A11Y-004; REQ-A11Y-003).
3. **G3** — all six locale files carry identical key sets; the check fails on any drift (T-I18N-004).
4. **G4** — RTL mirrors layout/directional icons while timeline/ruler/transport/timecode stay LTR; user content is `dir="auto"` (07 §7.11; snapshots in `ar`).
5. **G5** — snapshots are deterministic: identical inputs ⇒ identical pixels across repeat runs (D7).
6. **G6** — no raw colors or physical properties in component CSS (Stylelint); no JSX literals, no `any` (ESLint/tsc).
7. **G7** — zero axe violations on every story in every snapshot combination (SC-006).
8. **G8** — font files match their recorded SHA-256 and every license file is present (12 §12.16).
