# 12 — Catalogs

Canonical for: every enumeration used by the product. Other documents reference catalog IDs and never restate values.
Depends on: [03-data.md](03-data.md) (how values are stored), [04-interfaces.md](04-interfaces.md) (effect manifest contract), [07-experience.md](07-experience.md) (how they are presented).

**Units and conventions used in this catalog**

- Spatial values are resolution-independent: `x` in frame widths, `y` and sizes in frame heights (03 §3.3.3). The UI displays pixels of the current sequence.
- Angles are degrees, clockwise, 0° = pointing right (3 o'clock) unless stated.
- Colors are linear-in-storage RGBA floats 0–1 written here as hex sRGB for readability (`#RRGGBB`, alpha 1 unless stated).
- `anim` = animatable (keyframable). Ranges are inclusive. Defaults are exact.
- Every parameter label is a localization key `fx.<effectId>.<param>` (07 §7.12); the English text is the label shown here.

---

## 12.1 Video effects (VFX)

### 12.1.1 Intrinsic clip properties (not removable, always first in the chain)

| Param | Type | Range | Default | Anim |
|---|---|---|---|---|
| `position` | point | x −4..5, y −4..5 (frame units, 0.5/0.5 = center) | (0.5, 0.5) | yes |
| `scale` | float % | 0..2000 | 100 | yes |
| `scaleX`, `scaleY` | float % | 0..2000 | 100 | yes (used when `uniformScale` = false) |
| `uniformScale` | bool | — | true | no |
| `rotation` | angle | −3600..3600 | 0 | yes |
| `anchor` | point | x −4..5, y −4..5 (source frame units) | (0.5, 0.5) | yes |
| `flipH`, `flipV` | bool | — | false | no |
| `cropLeft`, `cropTop`, `cropRight`, `cropBottom` | float % | 0..100 | 0 | yes |
| `cropFeather` | float | 0..0.2 | 0 | yes |
| `opacity` | float % | 0..100 | 100 | yes |
| `blendMode` | enum BLM | BLM-01..BLM-24 | `normal` | no |

Fit rule on insert: a source whose aspect differs from the sequence is scaled to **fit** (letterboxed) by default; setting `alvs.timeline.insertFitMode` = `fill` | `fit` | `none` changes this.

### 12.1.2 Addable video effects

| ID | Effect ID | Name | Category | Rel | Parameters (`name`: type, range, default, anim) |
|---|---|---|---|---|---|
| VFX-01 | `fx.color.basic` | Basic Correction | Color | 1.0 | `exposure`: float −4..4 EV, 0, yes · `contrast`: −100..100, 0, yes · `highlights`: −100..100, 0, yes · `shadows`: −100..100, 0, yes · `whites`: −100..100, 0, yes · `blacks`: −100..100, 0, yes · `temperature`: −100..100, 0, yes · `tint`: −100..100, 0, yes · `saturation`: 0..200 %, 100, yes · `vibrance`: −100..100, 0, yes |
| VFX-02 | `fx.color.wheels` | Color Wheels | Color | 1.0 | `lift`, `gamma`, `gain`, `offset`: wheel {chroma (x, y) each −1..1 default (0,0); luma −1..1 default 0}, yes · `saturation`: 0..200 %, 100, yes · `mix`: 0..100 %, 100, yes |
| VFX-03 | `fx.color.curves` | Curves | Color | 1.0 | `master`, `red`, `green`, `blue`: curve (2–16 points, x and y 0..1, monotone cubic, default identity), no · `mix`: 0..100 %, 100, yes |
| VFX-04 | `fx.color.lut` | LUT | Color | 1.0 | `lut`: LUT asset reference (required), no · `intensity`: 0..100 %, 100, yes · `interpolation`: enum `tetrahedral` \| `trilinear`, `tetrahedral`, no |
| VFX-05 | `fx.color.levels` | Levels | Color | 1.0 | `inBlack`: 0..1, 0, yes · `inWhite`: 0..1, 1, yes · `gamma`: 0.1..10, 1, yes · `outBlack`: 0..1, 0, yes · `outWhite`: 0..1, 1, yes |
| VFX-06 | `fx.color.bw` | Black & White | Color | 1.0 | `reds`, `yellows`, `greens`, `cyans`, `blues`, `magentas`: −100..100 luminance offset per hue range, 0, yes · `tintColor`: color, `#D9B38C`, yes · `tintAmount`: 0..100 %, 0, yes |
| VFX-07 | `fx.color.invert` | Invert | Color | 1.0 | `channels`: enum `rgb` \| `luma`, `rgb`, no · `mix`: 0..100 %, 100, yes |
| VFX-08 | `fx.blur.gaussian` | Gaussian Blur | Blur & Sharpen | 1.0 | `radius`: 0..0.25, 0.01, yes · `direction`: enum `both` \| `horizontal` \| `vertical`, `both`, no · `edges`: enum `clamp` \| `transparent`, `clamp`, no |
| VFX-09 | `fx.blur.directional` | Directional Blur | Blur & Sharpen | 1.0 | `angle`: −180..180, 0, yes · `length`: 0..0.2, 0.02, yes |
| VFX-10 | `fx.blur.zoom` | Zoom Blur | Blur & Sharpen | 1.0 | `center`: point, (0.5, 0.5), yes · `amount`: 0..1, 0.2, yes · `quality`: enum `draft` \| `normal` \| `high`, `normal`, no |
| VFX-11 | `fx.sharpen` | Sharpen | Blur & Sharpen | 1.0 | `amount`: 0..300 %, 50, yes · `radius`: 0.0005..0.01, 0.001, yes · `threshold`: 0..1, 0, yes |
| VFX-12 | `fx.stylize.mosaic` | Mosaic | Stylize | 1.0 | `cellSize`: 0.002..0.2, 0.02, yes |
| VFX-13 | `fx.key.chroma` | Chroma Key | Keying | 1.0 | `keyColor`: color, `#00B140`, no · `tolerance`: 0..100, 30, yes · `softness`: 0..100, 10, yes · `spill`: 0..100, 50, yes · `choke`: −0.02..0.02, 0, yes · `showMatte`: bool, false, no |
| VFX-14 | `fx.key.luma` | Luma Key | Keying | 1.0 | `threshold`: 0..1, 0.1, yes · `softness`: 0..1, 0.05, yes · `invert`: bool, false, no · `showMatte`: bool, false, no |
| VFX-15 | `fx.stylize.vignette` | Vignette | Stylize | 1.0 | `amount`: 0..100, 40, yes · `midpoint`: 0..1, 0.5, yes · `roundness`: 0..1, 0.5, yes · `feather`: 0..1, 0.5, yes · `color`: color, `#000000`, yes |
| VFX-16 | `fx.stylize.glow` | Glow | Stylize | 1.0 | `threshold`: 0..1, 0.7, yes · `radius`: 0..0.1, 0.02, yes · `intensity`: 0..400 %, 100, yes · `tint`: color, `#FFFFFF`, yes |
| VFX-17 | `fx.stylize.grain` | Film Grain | Stylize | 1.0 | `amount`: 0..100, 20, yes · `size`: 0.5..4, 1, yes · `monochrome`: bool, true, no · `seed`: int 0..65535, 1, no · `animated`: bool, true, no |
| VFX-18 | `fx.stylize.shadow` | Drop Shadow | Stylize | 1.0 | `color`: color, `#000000`, yes · `opacity`: 0..100 %, 60, yes · `angle`: 0..360, 135, yes · `distance`: 0..0.1, 0.01, yes · `softness`: 0..0.1, 0.01, yes |
| VFX-19 | `fx.stylize.letterbox` | Letterbox | Stylize | 1.0 | `aspect`: enum `2.39` \| `2.00` \| `1.85` \| `1.33` \| `1.00` \| `custom`, `2.39`, no · `customAspect`: 0.5..4, 2.39, yes · `color`: color, `#000000`, yes · `opacity`: 0..100 %, 100, yes |
| VFX-20 | `fx.distort.mirror` | Mirror | Distort | 1.0 | `mode`: enum `leftToRight` \| `rightToLeft` \| `topToBottom` \| `bottomToTop` \| `quad`, `leftToRight`, no · `center`: 0..1, 0.5, yes |
| VFX-21 | `fx.distort.cornerPin` | Corner Pin | Distort | 1.0 | `topLeft` (0,0), `topRight` (1,0), `bottomRight` (1,1), `bottomLeft` (0,1): point, yes |
| VFX-22 | `fx.distort.lens` | Lens Distortion | Distort | 1.0 (P1) | `amount`: −100..100, 0, yes · `scale`: 50..150 %, 100, yes |

Effect semantics (shader math) are specified in 02 §2.6.6; each effect's manifest (04 §4.8) carries these exact parameter definitions.

## 12.2 Audio effects (AFX)

Clip gain, pan, fades, and channel mode are intrinsic clip audio properties (03 §3.4.6). The effects below can be added to clips, tracks, and the master.

| ID | Effect ID | Name | Rel | Parameters (`name`: range, default) — all automatable unless marked |
|---|---|---|---|---|
| AFX-01 | `afx.eq.parametric` | Parametric EQ | 1.0 | `hpfOn` (bool, false, no) · `hpfFreq` 20..1000 Hz, 80 · `lowShelfFreq` 20..1000 Hz, 120 · `lowShelfGain` −18..18 dB, 0 · `peak1Freq` 40..16000 Hz, 250 · `peak2Freq` …, 1000 · `peak3Freq` …, 3000 · `peak4Freq` …, 8000 · `peakNGain` −18..18 dB, 0 · `peakNQ` 0.3..10, 1.0 · `highShelfFreq` 2000..20000 Hz, 10000 · `highShelfGain` −18..18 dB, 0 · `lpfOn` (bool, false, no) · `lpfFreq` 1000..20000 Hz, 16000 |
| AFX-02 | `afx.dyn.compressor` | Compressor | 1.0 | `threshold` −60..0 dB, −18 · `ratio` 1..20, 3 · `attack` 0.1..200 ms, 10 · `release` 10..2000 ms, 120 · `knee` 0..24 dB, 6 · `makeup` −12..24 dB, 0 · `autoMakeup` (bool, false, no) · `mix` 0..100 %, 100 |
| AFX-03 | `afx.dyn.limiter` | Limiter | 1.0 | `ceiling` −12..0 dBTP, −1.0 · `release` 1..1000 ms, 50 · `truePeak` (bool, true, no); fixed 5 ms look-ahead |
| AFX-04 | `afx.dyn.gate` | Noise Gate | 1.0 | `threshold` −80..0 dB, −45 · `range` −80..0 dB, −40 · `attack` 0.1..50 ms, 2 · `hold` 0..500 ms, 50 · `release` 5..2000 ms, 150 |
| AFX-05 | `afx.dyn.deesser` | De-esser | 1.0 | `frequency` 3000..12000 Hz, 6500 · `threshold` −60..0 dB, −25 · `maxReduction` 0..24 dB, 8 |
| AFX-06 | `afx.restore.denoise` | Voice Denoise | 1.0 | `mix` 0..100 %, 100 (neural noise suppression, 48 kHz internal) |
| AFX-07 | `afx.space.reverb` | Reverb | 1.0 | `roomSize` 0..1, 0.5 · `damping` 0..1, 0.5 · `width` 0..1, 1 · `preDelay` 0..200 ms, 10 · `wet` 0..100 %, 25 · `dry` 0..100 %, 100 |
| AFX-08 | `afx.space.delay` | Delay | 1.0 | `time` 1..2000 ms, 350 · `feedback` 0..95 %, 35 · `wet` 0..100 %, 30 · `dry` 0..100 %, 100 · `pingPong` (bool, false, no) |
| AFX-09 | `afx.util.gain` | Gain | 1.0 | `gain` −60..24 dB, 0 |
| AFX-10 | `afx.util.channels` | Channel Tools | 1.0 | `mode` (enum `stereo` \| `swap` \| `monoMix`, `stereo`, no) · `invertLeft`, `invertRight` (bool, false, no) |
| AFX-11 | `afx.pitch.shift` | Pitch Shift | 1.0 (P1) | `semitones` −12..12, 0 · `cents` −100..100, 0 |

DSP algorithms are specified in 02 §2.7.3.

## 12.3 Transitions (TRS)

All video transitions take `progress` p ∈ [0,1] from outgoing clip A to incoming clip B. Default duration comes from `alvs.timeline.defaultTransitionSec`.

| ID | Transition ID | Name | Kind | Parameters (`name`: range, default) |
|---|---|---|---|---|
| TRS-01 | `tr.dissolve.cross` | Cross Dissolve | Video | `easing`: enum `linear` \| `smooth`, `linear` |
| TRS-02 | `tr.dissolve.dipBlack` | Dip to Black | Video | `hold`: 0..50 % of duration at full black, 0 |
| TRS-03 | `tr.dissolve.dipWhite` | Dip to White | Video | `hold`: 0..50 %, 0 |
| TRS-04 | `tr.dissolve.dipColor` | Dip to Color | Video | `color`: `#000000` · `hold`: 0..50 %, 0 |
| TRS-05 | `tr.dissolve.additive` | Additive Dissolve | Video | — |
| TRS-06 | `tr.dissolve.blur` | Blur Dissolve | Video | `maxBlur`: 0..0.1, 0.03 |
| TRS-07 | `tr.wipe.linear` | Linear Wipe | Video | `angle`: 0..360, 0 (reveals from left to right) · `softness`: 0..1, 0.1 |
| TRS-08 | `tr.wipe.clock` | Clock Wipe | Video | `startAngle`: 0..360, 270 (12 o'clock) · `direction`: enum `clockwise` \| `counterclockwise`, `clockwise` · `softness`: 0..1, 0.05 |
| TRS-09 | `tr.wipe.iris` | Iris | Video | `shape`: enum `circle` \| `box` \| `diamond`, `circle` · `center`: point, (0.5, 0.5) · `softness`: 0..1, 0.05 |
| TRS-10 | `tr.wipe.split` | Split | Video | `orientation`: enum `horizontal` \| `vertical`, `vertical` · `softness`: 0..1, 0.05 |
| TRS-11 | `tr.move.slide` | Slide | Video | `direction`: enum `left` \| `right` \| `up` \| `down`, `left` · `easing`: enum `linear` \| `smooth`, `smooth` |
| TRS-12 | `tr.move.push` | Push | Video | `direction`: same as TRS-11, `left` · `easing`: same, `smooth` |
| TRS-13 | `tr.move.zoom` | Zoom | Video | `mode`: enum `in` \| `out`, `in` · `center`: point, (0.5, 0.5) · `amount`: 1..4, 2 |
| TRS-14 | `tr.audio.constantPower` | Constant Power | Audio | — (gainA = cos(p·π/2), gainB = sin(p·π/2)); default audio crossfade |
| TRS-15 | `tr.audio.constantGain` | Constant Gain | Audio | — (gainA = 1 − p, gainB = p) |
| TRS-16 | `tr.audio.exponential` | Exponential | Audio | — (gainA = (1 − p)², gainB = 1 − (1 − p)²) |

## 12.4 Blend modes (BLM)

Notation: `Cb` backdrop color, `Cs` source color, non-premultiplied, display-referred values in [0,1], per channel unless marked non-separable. The blended color B(Cb,Cs) is composited with the W3C *Compositing and Blending Level 1* general formula: `Cs' = (1 − αb)·Cs + αb·B(Cb, Cs)`, then premultiplied source-over: `co = αs·Cs' + αb·Cb·(1 − αs)`, `αo = αs + αb·(1 − αs)`. Results are clamped to [0,1] before compositing.

| ID | Mode ID | Name | B(Cb, Cs) |
|---|---|---|---|
| BLM-01 | `normal` | Normal | Cs |
| BLM-02 | `darken` | Darken | min(Cb, Cs) |
| BLM-03 | `multiply` | Multiply | Cb·Cs |
| BLM-04 | `colorBurn` | Color Burn | Cb = 1 → 1; Cs = 0 → 0; else 1 − min(1, (1 − Cb)/Cs) |
| BLM-05 | `linearBurn` | Linear Burn | max(0, Cb + Cs − 1) |
| BLM-06 | `lighten` | Lighten | max(Cb, Cs) |
| BLM-07 | `screen` | Screen | Cb + Cs − Cb·Cs |
| BLM-08 | `colorDodge` | Color Dodge | Cb = 0 → 0; Cs = 1 → 1; else min(1, Cb/(1 − Cs)) |
| BLM-09 | `add` | Add (Linear Dodge) | min(1, Cb + Cs) |
| BLM-10 | `overlay` | Overlay | HardLight(Cs, Cb) (operands swapped) |
| BLM-11 | `softLight` | Soft Light | Cs ≤ 0.5: Cb − (1 − 2Cs)·Cb·(1 − Cb); else Cb + (2Cs − 1)·(D(Cb) − Cb), where D(x) = x ≤ 0.25 ? ((16x − 12)x + 4)x : √x |
| BLM-12 | `hardLight` | Hard Light | Cs ≤ 0.5: Cb·2Cs; else Screen(Cb, 2Cs − 1) |
| BLM-13 | `vividLight` | Vivid Light | Cs ≤ 0.5: ColorBurn(Cb, 2Cs); else ColorDodge(Cb, 2(Cs − 0.5)) |
| BLM-14 | `linearLight` | Linear Light | clamp(Cb + 2Cs − 1, 0, 1) |
| BLM-15 | `pinLight` | Pin Light | Cs ≤ 0.5: min(Cb, 2Cs); else max(Cb, 2Cs − 1) |
| BLM-16 | `hardMix` | Hard Mix | VividLight(Cb, Cs) < 0.5 → 0, else 1 |
| BLM-17 | `difference` | Difference | \|Cb − Cs\| |
| BLM-18 | `exclusion` | Exclusion | Cb + Cs − 2·Cb·Cs |
| BLM-19 | `subtract` | Subtract | max(0, Cb − Cs) |
| BLM-20 | `divide` | Divide | Cs = 0 → 1; else min(1, Cb/Cs) |
| BLM-21 | `hue` | Hue | Non-separable: SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb)) |
| BLM-22 | `saturation` | Saturation | Non-separable: SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb)) |
| BLM-23 | `color` | Color | Non-separable: SetLum(Cs, Lum(Cb)) |
| BLM-24 | `luminosity` | Luminosity | Non-separable: SetLum(Cb, Lum(Cs)) |

`Lum`, `SetLum`, `ClipColor`, `Sat`, `SetSat` follow the W3C definitions with Lum(C) = 0.3·R + 0.59·G + 0.11·B.

## 12.5 Keyframe easings (EAS)

Easing applies to the segment from a keyframe to the next one; t ∈ [0,1] is normalized segment time; output e(t) blends values v0 → v1 as v0 + (v1 − v0)·e(t). Overshooting easings (Back, Elastic) are clamped to the parameter range.

| ID | Easing ID | Name | e(t) |
|---|---|---|---|
| EAS-01 | `hold` | Hold | 0 for t < 1 (value jumps at the next keyframe) |
| EAS-02 | `linear` | Linear | t |
| EAS-03 | `auto` | Auto (smooth) | Monotone cubic Hermite through neighboring keyframes (Fritsch–Carlson tangents); never overshoots |
| EAS-04 | `bezier` | Bezier (manual) | Cubic Bezier in (time, value) with handles (x1, y1, x2, y2), x ∈ [0,1], y unbounded; default (0.33, 0, 0.67, 1) |
| EAS-05 | `sineIn` | Sine In | 1 − cos(πt/2) |
| EAS-06 | `sineOut` | Sine Out | sin(πt/2) |
| EAS-07 | `sineInOut` | Sine In-Out | −(cos(πt) − 1)/2 |
| EAS-08 | `quadIn` | Quad In | t² |
| EAS-09 | `quadOut` | Quad Out | 1 − (1 − t)² |
| EAS-10 | `quadInOut` | Quad In-Out | t < 0.5 ? 2t² : 1 − (−2t + 2)²/2 |
| EAS-11 | `cubicIn` | Cubic In | t³ |
| EAS-12 | `cubicOut` | Cubic Out | 1 − (1 − t)³ |
| EAS-13 | `cubicInOut` | Cubic In-Out | t < 0.5 ? 4t³ : 1 − (−2t + 2)³/2 |
| EAS-14 | `quartIn` | Quart In | t⁴ |
| EAS-15 | `quartOut` | Quart Out | 1 − (1 − t)⁴ |
| EAS-16 | `quartInOut` | Quart In-Out | t < 0.5 ? 8t⁴ : 1 − (−2t + 2)⁴/2 |
| EAS-17 | `quintIn` | Quint In | t⁵ |
| EAS-18 | `quintOut` | Quint Out | 1 − (1 − t)⁵ |
| EAS-19 | `quintInOut` | Quint In-Out | t < 0.5 ? 16t⁵ : 1 − (−2t + 2)⁵/2 |
| EAS-20 | `expoIn` | Expo In | t = 0 ? 0 : 2^(10t − 10) |
| EAS-21 | `expoOut` | Expo Out | t = 1 ? 1 : 1 − 2^(−10t) |
| EAS-22 | `expoInOut` | Expo In-Out | 0 / 1 at ends; t < 0.5 ? 2^(20t − 10)/2 : (2 − 2^(−20t + 10))/2 |
| EAS-23 | `circIn` | Circ In | 1 − √(1 − t²) |
| EAS-24 | `circOut` | Circ Out | √(1 − (t − 1)²) |
| EAS-25 | `circInOut` | Circ In-Out | t < 0.5 ? (1 − √(1 − (2t)²))/2 : (√(1 − (−2t + 2)²) + 1)/2 |
| EAS-26 | `backIn` | Back In | c3·t³ − c1·t², c1 = 1.70158, c3 = c1 + 1 |
| EAS-27 | `backOut` | Back Out | 1 + c3·(t − 1)³ + c1·(t − 1)² |
| EAS-28 | `backInOut` | Back In-Out | c2 = c1·1.525; t < 0.5 ? ((2t)²·((c2 + 1)·2t − c2))/2 : ((2t − 2)²·((c2 + 1)(2t − 2) + c2) + 2)/2 |
| EAS-29 | `elasticIn` | Elastic In | 0/1 at ends; −2^(10t − 10)·sin((10t − 10.75)·c4), c4 = 2π/3 |
| EAS-30 | `elasticOut` | Elastic Out | 0/1 at ends; 2^(−10t)·sin((10t − 0.75)·c4) + 1 |
| EAS-31 | `elasticInOut` | Elastic In-Out | 0/1 at ends; c5 = 2π/4.5; t < 0.5 ? −(2^(20t − 10)·sin((20t − 11.125)c5))/2 : (2^(−20t + 10)·sin((20t − 11.125)c5))/2 + 1 |
| EAS-32 | `bounceIn` | Bounce In | 1 − BounceOut(1 − t) |
| EAS-33 | `bounceOut` | Bounce Out | n1 = 7.5625, d1 = 2.75: t < 1/d1 ? n1t²; t < 2/d1 ? n1(t − 1.5/d1)² + 0.75; t < 2.5/d1 ? n1(t − 2.25/d1)² + 0.9375; else n1(t − 2.625/d1)² + 0.984375 |
| EAS-34 | `bounceInOut` | Bounce In-Out | t < 0.5 ? (1 − BounceOut(1 − 2t))/2 : (1 + BounceOut(2t − 1))/2 |

Default easing for new keyframes: `alvs.timeline.defaultEasing` (default `linear`). Boolean and enum parameters always use `hold`.

## 12.6 Masks (MSK)

| ID | Mask ID | Shape parameters (all anim) |
|---|---|---|
| MSK-01 | `mask.rect` | `center` point (0.5, 0.5) · `size` (w, h) (0.5, 0.5) · `cornerRadius` 0..0.5, 0 · `rotation` −3600..3600, 0 |
| MSK-02 | `mask.ellipse` | `center` (0.5, 0.5) · `size` (0.5, 0.5) · `rotation` 0 |
| MSK-03 | `mask.path` | `points`: list of {anchor, inTangent, outTangent} (3–256 points), `closed` = true; all keyframes of a path have the same point count |

Common to all masks: `feather` 0..0.2 (0.01), `expansion` −0.1..0.1 (0), `opacity` 0..100 % (100), `invert` bool (false, not anim), `mode` enum `add` | `subtract` | `intersect` (`add`, not anim). Masks are evaluated in order; the first mask's mode is treated as `add`.

## 12.7 Export presets (XP)

`seq` = sequence resolution and frame rate. "SW" = software encoder, "HW" = hardware alternative chosen when `Use hardware encoding` is on and available (12 §12.19). Encoder option names MUST be verified against `ffmpeg -h encoder=<name>` of the built FFmpeg before use (G-10).

| ID | Preset ID | Name | Container | Video (SW) | Video quality | Audio | Notes |
|---|---|---|---|---|---|---|---|
| XP-01 | `xp.h264.high` | H.264 High Quality | MP4 | libx264, High, yuv420p, preset `medium`, keyint 2 s, 3 B-frames | CRF 18 | AAC-LC 320 kbps, 48 kHz | **Default**; HW: H.264 family |
| XP-02 | `xp.h264.small` | H.264 Small File | MP4 | libx264, High, yuv420p, preset `fast` | CRF 23 | AAC-LC 192 kbps | HW: H.264 family |
| XP-03 | `xp.hevc.10bit` | HEVC 10-bit High Quality | MP4 | libx265, Main10, yuv420p10le, preset `medium`, tag `hvc1` | CRF 20 | AAC-LC 320 kbps | HW: HEVC family |
| XP-04 | `xp.av1` | AV1 Efficient | MP4 | libsvtav1, preset 6, yuv420p10le | CRF 30 | AAC-LC 256 kbps | HW: AV1 family where supported |
| XP-05 | `xp.vp9.webm` | VP9 WebM | WebM | libvpx-vp9, row-mt, yuv420p | CRF 31 (constant quality) | Opus 160 kbps | |
| XP-06 | `xp.h264.compat` | H.264 Maximum Compatibility | MP4 | libx264, Main, level 4.0, yuv420p, max 1920×1080 | VBR 8 Mbps (max 10) | AAC-LC 192 kbps | Downscales larger sequences to fit 1920×1080 |
| XP-07 | `xp.prores.hq` | ProRes 422 HQ | MOV | prores_ks profile `hq`, yuv422p10le | intra | PCM s24le 48 kHz | Mastering |
| XP-08 | `xp.prores.422` | ProRes 422 | MOV | prores_ks profile `standard`, yuv422p10le | intra | PCM s24le | |
| XP-09 | `xp.prores.lt` | ProRes 422 LT | MOV | prores_ks profile `lt`, yuv422p10le | intra | PCM s16le | Used by "Optimize media" (REQ-MED-011) |
| XP-10 | `xp.prores.4444` | ProRes 4444 (alpha) | MOV | prores_ks profile `4444`, yuva444p10le, alpha bits 16 | intra | PCM s24le | Alpha |
| XP-11 | `xp.dnxhr.hq` | DNxHR HQ | MOV | dnxhd profile `dnxhr_hq`, yuv422p | intra | PCM s24le | |
| XP-12 | `xp.ffv1.archive` | FFV1 Lossless Archive | MKV | ffv1 level 3, 16 slices, CRC on, gbrp10le | lossless | FLAC | |
| XP-13 | `xp.gif` | Animated GIF | GIF | two-pass palette (stats from whole range), max 30 fps, width ≤ 960 (option) | — | none | |
| XP-14 | `xp.png.seq` | PNG Sequence (alpha) | folder of `.png` | RGBA 8-bit (option 16-bit) | lossless | none (optional WAV sidecar) | Alpha |
| XP-15 | `xp.jpeg.seq` | JPEG Sequence | folder of `.jpg` | 4:2:0, quality 90 | — | none | |
| XP-16 | `xp.audio.wav` | WAV 24-bit | WAV | — | — | PCM s24le 48 kHz | Audio only |
| XP-17 | `xp.audio.mp3` | MP3 320 kbps | MP3 | — | — | libmp3lame CBR 320 kbps | Audio only |
| XP-18 | `xp.audio.aac` | AAC 256 kbps | M4A | — | — | AAC-LC 256 kbps | Audio only |
| XP-19 | `xp.audio.flac` | FLAC | FLAC | — | — | FLAC level 5 | Audio only |
| XP-20 | `xp.audio.opus` | Opus 160 kbps | Ogg (`.opus`) | — | — | libopus 160 kbps | Audio only |

Image-sequence file names: `<name>_%06d.<ext>` starting at 000000.

## 12.8 Sequence presets (SQP)

| ID | Name | Width × Height | Aspect |
|---|---|---|---|
| SQP-01 | Auto — match first clip (fallback SQP-02 at 30 fps) | from the first video clip added | — |
| SQP-02 | HD 1080p | 1920 × 1080 | 16:9 |
| SQP-03 | HD 720p | 1280 × 720 | 16:9 |
| SQP-04 | QHD 1440p | 2560 × 1440 | 16:9 |
| SQP-05 | UHD 4K | 3840 × 2160 | 16:9 |
| SQP-06 | Vertical 1080 | 1080 × 1920 | 9:16 |
| SQP-07 | Vertical 4K | 2160 × 3840 | 9:16 |
| SQP-08 | Square | 1080 × 1080 | 1:1 |
| SQP-09 | Portrait 4:5 | 1080 × 1350 | 4:5 |
| SQP-10 | Cinema 2.39 UHD | 3840 × 1608 | 2.39:1 |
| SQP-11 | DCI 2K | 2048 × 1080 | ≈1.9:1 |
| SQP-12 | DCI 4K | 4096 × 2160 | ≈1.9:1 |
| SQP-13 | Custom | even integers 16..8192 each | any |

## 12.9 Frame rates (FPS)

| ID | Display | Rational | Timecode |
|---|---|---|---|
| FPS-01 | 23.976 | 24000/1001 | non-drop |
| FPS-02 | 24 | 24/1 | non-drop |
| FPS-03 | 25 | 25/1 | non-drop |
| FPS-04 | 29.97 | 30000/1001 | drop-frame (default) or non-drop |
| FPS-05 | 30 | 30/1 | non-drop |
| FPS-06 | 50 | 50/1 | non-drop |
| FPS-07 | 59.94 | 60000/1001 | drop-frame (default) or non-drop |
| FPS-08 | 60 | 60/1 | non-drop |

## 12.10 Commands and default shortcuts (KB)

Shortcuts bind to physical keys (`KeyboardEvent.code`, REQ-I18N-005); the key names below are the US-layout labels. Contexts: `global`, `timeline`, `viewer`, `media`. While an editable text field has focus, the field receives every key; only these commands still fire: Ctrl+S, Ctrl+Shift+S, Ctrl+E, Ctrl+Shift+P, F1, F6/Shift+F6, F11 (07 §7.9.2). Shuttle speeds for J/L: each press steps through ±1, ±2, ±4, ±8; K+J / K+L hold = slow play (±0.5×).

| ID | Command ID | Label | Menu | Default shortcut | Context |
|---|---|---|---|---|---|
| KB-001 | `file.newProject` | New Project… | File | Ctrl+N | global |
| KB-002 | `file.openProject` | Open Project… | File | Ctrl+O | global |
| KB-003 | `file.save` | Save | File | Ctrl+S | global |
| KB-004 | `file.saveAs` | Save As… | File | Ctrl+Shift+S | global |
| KB-005 | `file.closeProject` | Close Project | File | Ctrl+Shift+W | global |
| KB-006 | `file.importMedia` | Import Media… | File | Ctrl+I | global |
| KB-007 | `file.importCaptions` | Import Captions… | File | — | global |
| KB-008 | `file.export` | Export… | File | Ctrl+E | global |
| KB-009 | `file.exportFrame` | Export Frame… | File | Ctrl+Shift+E | global |
| KB-010 | `file.exportMarkers` | Export Markers › Chapter List / CSV… | File | — | global |
| KB-011 | `file.exportEdl` | Export EDL… | File | — | global |
| KB-012 | `file.collectFiles` | Collect Files… | File | — | global |
| KB-013 | `file.restoreBackup` | Restore from Backup… | File | — | global |
| KB-014 | `file.projectSettings` | Project Settings… | File | — | global |
| KB-015 | `file.settings` | Settings… | File | Ctrl+, | global |
| KB-016 | `file.exit` | Exit | File | Alt+F4 | global |
| KB-017 | `edit.undo` | Undo | Edit | Ctrl+Z | global |
| KB-018 | `edit.redo` | Redo | Edit | Ctrl+Y (also Ctrl+Shift+Z) | global |
| KB-019 | `edit.cut` | Cut | Edit | Ctrl+X | global |
| KB-020 | `edit.copy` | Copy | Edit | Ctrl+C | global |
| KB-021 | `edit.paste` | Paste | Edit | Ctrl+V | global |
| KB-022 | `edit.pasteInsert` | Paste Insert | Edit | Ctrl+Shift+V | timeline |
| KB-023 | `edit.pasteAttributes` | Paste Attributes… | Edit | Ctrl+Alt+V | timeline |
| KB-024 | `edit.duplicate` | Duplicate | Edit | Ctrl+D | timeline |
| KB-025 | `edit.delete` | Delete | Edit | Delete | global |
| KB-026 | `edit.rippleDelete` | Ripple Delete | Edit | Shift+Delete | timeline |
| KB-027 | `edit.selectAll` | Select All | Edit | Ctrl+A | global |
| KB-028 | `edit.deselectAll` | Deselect All | Edit | Ctrl+Shift+A | global |
| KB-029 | `edit.rename` | Rename | Edit | F2 | global |
| KB-030 | `edit.find` | Find | Edit | Ctrl+F | global |
| KB-031 | `edit.keyboardShortcuts` | Keyboard Shortcuts… | Edit | Ctrl+Alt+K | global |
| KB-032 | `clip.split` | Split at Playhead | Clip | Ctrl+K | timeline |
| KB-033 | `clip.splitAll` | Split All Tracks at Playhead | Clip | Ctrl+Shift+K | timeline |
| KB-034 | `clip.toggleEnabled` | Enable/Disable Clip | Clip | Shift+E | timeline |
| KB-035 | `clip.toggleLink` | Link/Unlink | Clip | Ctrl+L | timeline |
| KB-036 | `clip.group` | Group | Clip | Ctrl+G | timeline |
| KB-037 | `clip.ungroup` | Ungroup | Clip | Ctrl+Shift+G | timeline |
| KB-038 | `clip.speedDuration` | Speed/Duration… | Clip | Ctrl+R | timeline |
| KB-039 | `clip.reverse` | Reverse | Clip | — | timeline |
| KB-040 | `clip.freezeFrame` | Freeze Frame… | Clip | — | timeline |
| KB-041 | `clip.nest` | Nest Selection | Clip | — | timeline |
| KB-042 | `clip.matchFrame` | Match Frame (show in Source) | Clip | F | timeline |
| KB-043 | `clip.addMaskRect` | Add Rectangle Mask | Clip › Mask | — | timeline |
| KB-044 | `clip.addMaskEllipse` | Add Ellipse Mask | Clip › Mask | — | timeline |
| KB-045 | `clip.addMaskPath` | Draw Path Mask | Clip › Mask | — | timeline |
| KB-046 | `clip.labelColor` | Label Color › CLR-01..08 | Clip | — | timeline |
| KB-047 | `clip.normalizeLoudness` | Normalize Loudness… | Clip | — | timeline |
| KB-048 | `clip.stabilize` | Basic Stabilize… | Clip | — | timeline |
| KB-049 | `clip.transcribe` | Transcribe to Captions… | Clip | — | timeline |
| KB-050 | `clip.copyGrade` | Copy Grade | Clip | Ctrl+Alt+C | timeline |
| KB-051 | `clip.pasteGrade` | Paste Grade | Clip | Ctrl+Alt+Shift+V | timeline |
| KB-052 | `media.createProxy` | Create Proxies | Clip | — | media |
| KB-053 | `media.optimize` | Optimize Media… | Clip | — | media |
| KB-054 | `media.relink` | Relink Media… | Clip | — | media |
| KB-055 | `media.revealInExplorer` | Show in Explorer | Clip | Ctrl+Shift+R | media |
| KB-056 | `timeline.newSequence` | New Sequence… | Timeline | Ctrl+Alt+N | global |
| KB-057 | `timeline.sequenceSettings` | Sequence Settings… | Timeline | — | global |
| KB-058 | `timeline.addVideoTrack` | Add Video Track | Timeline | — | timeline |
| KB-059 | `timeline.addAudioTrack` | Add Audio Track | Timeline | — | timeline |
| KB-060 | `timeline.addCaptionTrack` | Add Caption Track | Timeline | — | timeline |
| KB-061 | `timeline.insert` | Insert from Source | Timeline | , (Comma) | global |
| KB-062 | `timeline.overwrite` | Overwrite from Source | Timeline | . (Period) | global |
| KB-063 | `timeline.lift` | Lift In–Out | Timeline | ; (Semicolon) | timeline |
| KB-064 | `timeline.extract` | Extract In–Out | Timeline | ' (Quote) | timeline |
| KB-065 | `timeline.trimPrevToPlayhead` | Ripple Trim Previous Edit to Playhead | Timeline | Q | timeline |
| KB-066 | `timeline.trimNextToPlayhead` | Ripple Trim Next Edit to Playhead | Timeline | W | timeline |
| KB-067 | `timeline.extendEdit` | Extend Edit to Playhead | Timeline | E | timeline |
| KB-068 | `timeline.addVideoTransition` | Add Default Video Transition | Timeline | Ctrl+T | timeline |
| KB-069 | `timeline.addAudioTransition` | Add Default Audio Crossfade | Timeline | Ctrl+Shift+T | timeline |
| KB-070 | `timeline.toggleSnapping` | Snapping | Timeline | S | timeline |
| KB-071 | `timeline.toggleLinkedSelection` | Linked Selection | Timeline | Ctrl+Shift+L | timeline |
| KB-072 | `timeline.closeGap` | Close Gap | Timeline | — | timeline |
| KB-073 | `timeline.deleteAllGaps` | Delete All Gaps on Track | Timeline | — | timeline |
| KB-074 | `timeline.nudgeLeft` | Nudge Left 1 Frame | Timeline | Alt+Left | timeline |
| KB-075 | `timeline.nudgeRight` | Nudge Right 1 Frame | Timeline | Alt+Right | timeline |
| KB-076 | `timeline.nudgeLeft10` | Nudge Left 10 Frames | Timeline | Alt+Shift+Left | timeline |
| KB-077 | `timeline.nudgeRight10` | Nudge Right 10 Frames | Timeline | Alt+Shift+Right | timeline |
| KB-078 | `timeline.audioSubframeNudge` | Nudge Audio by 1/4 Frame (Left/Right) | Timeline | Ctrl+Alt+Left / Ctrl+Alt+Right | timeline |
| KB-079 | `timeline.zoomIn` | Zoom In | Timeline | Ctrl+= | timeline |
| KB-080 | `timeline.zoomOut` | Zoom Out | Timeline | Ctrl+- | timeline |
| KB-081 | `timeline.zoomFit` | Zoom to Fit | Timeline | Shift+Z | timeline |
| KB-082 | `timeline.addMarker` | Add Marker | Timeline | M | global |
| KB-083 | `timeline.editMarker` | Edit Marker… | Timeline | Shift+M | global |
| KB-084 | `timeline.nextMarker` | Next Marker | Timeline | Ctrl+Shift+Right | global |
| KB-085 | `timeline.prevMarker` | Previous Marker | Timeline | Ctrl+Shift+Left | global |
| KB-086 | `timeline.addCaption` | Add Caption at Playhead | Timeline | Ctrl+Alt+A | global |
| KB-087 | `timeline.listView` | Timeline List View | Timeline | Ctrl+Alt+L | global |
| KB-088 | `timeline.recordVoiceOver` | Record Voice-over… | Timeline | Ctrl+Alt+R | global |
| KB-089 | `timeline.autoDuck` | Auto-Duck Music… | Timeline | — | global |
| KB-090 | `timeline.transcribeSequence` | Transcribe Sequence… | Timeline | — | global |
| KB-091 | `tool.select` | Selection Tool | Timeline › Tools | V | timeline |
| KB-092 | `tool.blade` | Blade Tool | Timeline › Tools | B | timeline |
| KB-093 | `tool.trim` | Trim Tool (ripple at edge, roll at edit point) | Timeline › Tools | T | timeline |
| KB-094 | `tool.slip` | Slip Tool | Timeline › Tools | Y | timeline |
| KB-095 | `tool.slide` | Slide Tool | Timeline › Tools | U | timeline |
| KB-096 | `tool.hand` | Hand Tool | Timeline › Tools | H | timeline |
| KB-097 | `tool.zoom` | Zoom Tool | Timeline › Tools | Z | timeline |
| KB-098 | `playback.playPause` | Play/Pause | Playback | Space | global |
| KB-099 | `playback.shuttleReverse` | Shuttle Reverse | Playback | J | global |
| KB-100 | `playback.pause` | Pause | Playback | K | global |
| KB-101 | `playback.shuttleForward` | Shuttle Forward | Playback | L | global |
| KB-102 | `playback.stepBack` | Previous Frame | Playback | Left | global |
| KB-103 | `playback.stepForward` | Next Frame | Playback | Right | global |
| KB-104 | `playback.stepBack10` | Back 10 Frames | Playback | Shift+Left | global |
| KB-105 | `playback.stepForward10` | Forward 10 Frames | Playback | Shift+Right | global |
| KB-106 | `playback.prevEdit` | Previous Edit Point | Playback | Up | global |
| KB-107 | `playback.nextEdit` | Next Edit Point | Playback | Down | global |
| KB-108 | `playback.goStart` | Go to Start | Playback | Home | global |
| KB-109 | `playback.goEnd` | Go to End | Playback | End | global |
| KB-110 | `playback.markIn` | Mark In | Playback | I | global |
| KB-111 | `playback.markOut` | Mark Out | Playback | O | global |
| KB-112 | `playback.clearIn` | Clear In | Playback | Alt+I | global |
| KB-113 | `playback.clearOut` | Clear Out | Playback | Alt+O | global |
| KB-114 | `playback.clearInOut` | Clear In and Out | Playback | Alt+X | global |
| KB-115 | `playback.goIn` | Go to In | Playback | Shift+I | global |
| KB-116 | `playback.goOut` | Go to Out | Playback | Shift+O | global |
| KB-117 | `playback.playInToOut` | Play In to Out | Playback | Ctrl+Shift+Space | global |
| KB-118 | `playback.toggleLoop` | Loop Playback | Playback | Shift+L | global |
| KB-119 | `playback.goToTimecode` | Go to Timecode… | Playback | Ctrl+J | global |
| KB-120 | `playback.toggleAudioScrub` | Audio Scrubbing | Playback | Shift+S | global |
| KB-121 | `playback.previewQuality` | Preview Resolution › Auto/Full/Half/Quarter | Playback | — | global |
| KB-122 | `playback.playEveryFrame` | Play Every Frame | Playback | — | global |
| KB-123 | `playback.fullscreen` | Full Screen | Playback | F11 | global |
| KB-124 | `playback.externalPreview` | External Preview Window… | Playback | Ctrl+Shift+F11 | global |
| KB-125 | `view.workspaceEdit` | Workspace › Edit | View | Alt+1 | global |
| KB-126 | `view.workspaceColor` | Workspace › Color | View | Alt+2 | global |
| KB-127 | `view.workspaceAudio` | Workspace › Audio | View | Alt+3 | global |
| KB-128 | `view.workspaceCaptions` | Workspace › Captions | View | Alt+4 | global |
| KB-129 | `view.saveWorkspace` | Save Workspace As… | View | — | global |
| KB-130 | `view.resetWorkspace` | Reset Workspace | View | — | global |
| KB-131 | `view.commandPalette` | Command Palette | View | Ctrl+Shift+P | global |
| KB-132 | `view.maximizePanel` | Maximize Panel | View | ` (Backquote) | global |
| KB-133 | `view.nextPanel` | Next Panel | View | F6 | global |
| KB-134 | `view.prevPanel` | Previous Panel | View | Shift+F6 | global |
| KB-135 | `view.focusMedia` | Media Panel | View › Panels | Shift+1 | global |
| KB-136 | `view.focusSource` | Source Viewer | View › Panels | Shift+2 | global |
| KB-137 | `view.focusTimeline` | Timeline | View › Panels | Shift+3 | global |
| KB-138 | `view.focusProgram` | Program Viewer | View › Panels | Shift+4 | global |
| KB-139 | `view.focusInspector` | Inspector | View › Panels | Shift+5 | global |
| KB-140 | `view.focusEffects` | Effects | View › Panels | Shift+6 | global |
| KB-141 | `view.focusCaptions` | Captions | View › Panels | Shift+7 | global |
| KB-142 | `view.focusMixer` | Mixer | View › Panels | Shift+8 | global |
| KB-143 | `view.focusScopes` | Scopes | View › Panels | Shift+9 | global |
| KB-144 | `view.uiZoomIn` | Interface Zoom In | View | Ctrl+Alt+= | global |
| KB-145 | `view.uiZoomOut` | Interface Zoom Out | View | Ctrl+Alt+- | global |
| KB-146 | `view.uiZoomReset` | Interface Zoom Reset | View | Ctrl+Alt+0 | global |
| KB-147 | `view.toggleSafeAreas` | Safe Areas | View › Overlays | — | viewer |
| KB-148 | `view.toggleGrid` | Grid | View › Overlays | — | viewer |
| KB-149 | `view.compareSplit` | Before/After Split | View › Overlays | — | viewer |
| KB-150 | `view.theme` | Theme › Dark/Light/High Contrast | View | — | global |
| KB-151 | `help.gettingStarted` | Getting Started | Help | — | global |
| KB-152 | `help.userGuide` | User Guide | Help | F1 | global |
| KB-153 | `help.shortcutSheet` | Keyboard Shortcuts Sheet | Help | Ctrl+/ | global |
| KB-154 | `help.replayTour` | Replay Tour | Help | — | global |
| KB-155 | `help.reportProblem` | Report a Problem… | Help | — | global |
| KB-156 | `help.diagnostics` | Create Diagnostics Bundle… | Help | — | global |
| KB-157 | `help.checkUpdates` | Check for Updates… | Help | — | global |
| KB-158 | `help.about` | About AfterLeads-Video-Spirits | Help | — | global |
| KB-159 | `playback.announceTimecode` | Announce Timecode (screen readers) | Playback | Ctrl+Alt+T | global |
| KB-160 | `timeline.selectPrevClip` | Select Previous Clip on Track | Timeline › Navigate | Ctrl+Left | timeline |
| KB-161 | `timeline.selectNextClip` | Select Next Clip on Track | Timeline › Navigate | Ctrl+Right | timeline |
| KB-162 | `timeline.focusTrackUp` | Focus Track Above | Timeline › Navigate | Ctrl+Up | timeline |
| KB-163 | `timeline.focusTrackDown` | Focus Track Below | Timeline › Navigate | Ctrl+Down | timeline |
| KB-164 | `timeline.trimEdgeLeft` | Trim Selected Edge Left 1 Frame | Timeline › Trim | [ (BracketLeft) | timeline |
| KB-165 | `timeline.trimEdgeRight` | Trim Selected Edge Right 1 Frame | Timeline › Trim | ] (BracketRight) | timeline |
| KB-166 | `timeline.trimEdgeLeft10` | Trim Selected Edge Left 10 Frames | Timeline › Trim | Shift+[ | timeline |
| KB-167 | `timeline.trimEdgeRight10` | Trim Selected Edge Right 10 Frames | Timeline › Trim | Shift+] | timeline |
| KB-168 | `timeline.inspectSelection` | Show Selection in Inspector | — (palette only) | Enter | timeline |

Commands whose Menu column is "— (palette only)" appear only in the command palette and keyboard shortcut editor. The default map above contains no conflicts within any context; the shortcut editor's conflict check (REQ-SET-002) MUST report zero conflicts for it, and T-SET-002 verifies this.

## 12.11 Menus

The menu bar order is File, Edit, Clip, Timeline, Playback, View, Help. Each menu lists its commands in KB order from 12.10, with separators between these groups:

| Menu | Groups (separated) |
|---|---|
| File | New/Open/Open Recent ▸ · Save/Save As/Close · Import Media/Import Captions · Export/Export Frame/Export Markers ▸/Export EDL · Collect Files/Restore from Backup · Project Settings/Settings · Exit |
| Edit | Undo/Redo · Cut/Copy/Paste/Paste Insert/Paste Attributes/Duplicate · Delete/Ripple Delete · Select All/Deselect All · Rename/Find · Keyboard Shortcuts |
| Clip | Split/Split All · Enable/Link/Group/Ungroup · Speed-Duration/Reverse/Freeze Frame/Nest · Match Frame · Mask ▸ · Label Color ▸ · Normalize Loudness/Basic Stabilize/Transcribe · Copy Grade/Paste Grade · Create Proxies/Optimize Media/Relink Media/Show in Explorer |
| Timeline | New Sequence/Sequence Settings · Add Video/Audio/Caption Track · Insert/Overwrite/Lift/Extract · Trim Previous/Next/Extend · Add Video Transition/Add Audio Crossfade · Snapping ✓/Linked Selection ✓ · Close Gap/Delete All Gaps · Nudge ▸ · Zoom In/Out/Fit · Markers ▸ (Add/Edit/Next/Previous) · Add Caption · Record Voice-over/Auto-Duck/Transcribe Sequence · Navigate ▸ (KB-160..KB-163) · Trim ▸ (KB-164..KB-167) · Tools ▸ · List View ✓ |
| Playback | Play-Pause/Shuttle ▸ · Frame/Edit navigation ▸ · Mark In/Out/Clear ▸/Go to In/Out · Play In to Out/Loop ✓ · Go to Timecode · Audio Scrubbing ✓/Preview Resolution ▸/Play Every Frame ✓ · Full Screen/External Preview |
| View | Workspaces ▸ (built-in + saved + Save/Reset) · Panels ▸ · Command Palette · Maximize Panel/Next/Previous Panel · Overlays ▸ · Interface Zoom ▸ · Theme ▸ · Language ▸ (LOC-01..06) |
| Help | Getting Started/User Guide/Keyboard Shortcuts Sheet/Replay Tour · Report a Problem/Create Diagnostics Bundle · Check for Updates · About |

Context menus are specified per panel in 07 §7.8.

## 12.12 Title animations, title templates, generators

### 12.12.1 Title animations (TAN)

Applied at the start (in) or end (out) of a title clip; `duration` 0.1..5 s (default 0.5 s); out animations play the in animation mirrored in time.

| ID | Animation ID | Name | Behavior |
|---|---|---|---|
| TAN-01 | `tan.fade` | Fade | opacity 0 → 1 |
| TAN-02 | `tan.slideUp` | Slide Up | translate y +0.05 → 0 with fade |
| TAN-03 | `tan.slideDown` | Slide Down | translate y −0.05 → 0 with fade |
| TAN-04 | `tan.slideLeft` | Slide from Right | translate x +0.05 → 0 with fade |
| TAN-05 | `tan.slideRight` | Slide from Left | translate x −0.05 → 0 with fade |
| TAN-06 | `tan.pop` | Scale Pop | scale 60% → 105% → 100% (backOut) with fade |
| TAN-07 | `tan.blurIn` | Blur In | blur 0.02 → 0 with fade |
| TAN-08 | `tan.typewriter` | Typewriter | reveals grapheme clusters in logical order (RTL-correct) |
| TAN-09 | `tan.wipe` | Wipe Reveal | linear mask wipe in reading direction of the first strong character |
| TAN-10 | `tan.riseLines` | Rise by Line | each line rises 0.03 with 80 ms stagger |

### 12.12.2 Title templates (TPL)

Original designs created for this project (fonts FNT-04..FNT-08 only). Each template defines text layers, styles, and animations; all text is editable after insert. Default duration 5 s.

| ID | Template ID | Name | Layout |
|---|---|---|---|
| TPL-01 | `tpl.lowerThird.clean` | Clean Lower Third | Name (bold) + role (regular) on a translucent bar, bottom-start |
| TPL-02 | `tpl.lowerThird.accent` | Accent Bar Lower Third | Accent-colored vertical bar + two lines |
| TPL-03 | `tpl.title.bold` | Bold Center Title | Large single line, centered, TAN-06 |
| TPL-04 | `tpl.title.cinematic` | Cinematic Title | Wide letter-spaced serif title + small subtitle, TAN-01 |
| TPL-05 | `tpl.chapter` | Chapter Card | Number + chapter name, full-frame dark background |
| TPL-06 | `tpl.quote` | Quote Card | Quotation text + author, serif |
| TPL-07 | `tpl.location` | Location Tag | Pin glyph + place + date, top-start |
| TPL-08 | `tpl.bigNumber` | Big Number | Large numeral + caption, TAN-06 |
| TPL-09 | `tpl.callout` | Callout | Rounded label with arrow line |
| TPL-10 | `tpl.endCard` | End Card | Thank-you line + two placeholder boxes for links |
| TPL-11 | `tpl.subtitleStyle` | Subtitle Style Text | Caption-like text with background box |
| TPL-12 | `tpl.arabic.elegant` | Arabic Elegant Title | Amiri headline + Cairo subtitle, RTL alignment |

### 12.12.3 Generators (GEN)

| ID | Generator ID | Name | Parameters |
|---|---|---|---|
| GEN-01 | `gen.solid` | Solid Color | `color` `#000000` |
| GEN-02 | `gen.gradient` | Gradient | `type` linear/radial · `colorA` `#3E63DD` · `colorB` `#8E4EC6` · `angle` 90 · `center` (0.5, 0.5) |
| GEN-03 | `gen.bars` | Color Bars | 75% bars with PLUGE row; no parameters |
| GEN-04 | `gen.adjustment` | Adjustment Layer | none (holds effects, REQ-FX-009) |
| GEN-05 | `gen.tone` | Tone | `frequency` 20..20000 Hz, 1000 · `level` −60..0 dBFS, −20 (audio clip) |

## 12.13 Error codes (E)

Envelope and severity semantics: 04 §4.9. "Rec." = recovery action offered in the UI.

| Code | Severity | Message (en) | Rec. |
|---|---|---|---|
| E0001 | error | An unexpected internal error occurred. | Save a copy; create diagnostics |
| E0002 | error | An operation was rolled back because it would corrupt the project. | Report a problem |
| E1001 | error | The project file was not found. | Locate file |
| E1002 | error | The project file is damaged (invalid JSON). | Restore from backup |
| E1003 | error | The project file failed validation. | Restore from backup |
| E1004 | warning | This project was created by a newer version and opens read-only. | Check for updates |
| E1005 | error | The project could not be upgraded; the original is unchanged. | Report a problem |
| E1006 | warning | A sequence cannot contain itself. | — |
| E1007 | warning | The project is open in another window or computer. | Open read-only / take over |
| E1008 | error | The project could not be saved. | Save As elsewhere |
| E1009 | warning | Autosave failed. | Check disk space |
| E1010 | error | The backup could not be restored. | Choose another backup |
| E1101 | info | The transition was shortened to fit the available media. | — |
| E1102 | info | That edit is not possible here (locked track or overlap). | — |
| E1103 | info | Not enough source media to trim further. | — |
| E1104 | warning | Some items were moved to match the new frame rate. | Show items |
| E2001 | warning | Media is offline. | Relink |
| E2002 | error | This file format or codec is not supported. | Learn more |
| E2003 | error | The file could not be read (it may be damaged). | — |
| E2004 | error | Reading this file crashed a helper process; the file was skipped. | Report a problem |
| E2005 | warning | A damaged frame was skipped. | — |
| E2006 | info | Hardware decoding is unavailable; using software decoding. | — |
| E2007 | error | The proxy could not be created. | Retry |
| E2008 | info | A very large image is shown at reduced size in previews. | — |
| E2009 | warning | The relinked file differs from the original (size or duration). | Confirm / cancel |
| E2010 | error | The LUT file is invalid (line {line}, column {column}). | — |
| E2011 | error | The caption file is invalid (line {line}). | — |
| E2012 | warning | The caption file's text encoding is uncertain. | Choose encoding |
| E3001 | warning | No compatible graphics adapter; using software rendering (slower). | Update graphics driver |
| E3002 | warning | The graphics driver was reset; rendering resumed. | — |
| E3003 | warning | Graphics memory is low; preview quality was reduced. | Lower preview resolution |
| E3004 | error | An effect could not be prepared and was bypassed. | Report a problem |
| E3005 | error | An effect definition is invalid and is unavailable. | Report a problem |
| E3006 | info | Font "{font}" is missing; a fallback font is used. | Choose font |
| E4001 | warning | The audio device was removed; switched to the default device. | Audio settings |
| E4002 | info | The audio device uses a different sample rate; audio is resampled. | — |
| E4003 | error | The recording input is unavailable. | Audio settings |
| E4004 | warning | Audio playback glitched (buffer underrun). | Increase buffer |
| E5001 | error | Not enough disk space for this export (needs about {size}). | Choose folder |
| E5002 | error | The export folder is not writable. | Choose folder |
| E5003 | error | Writing the export failed (disk full or removed). | Retry |
| E5004 | info | Hardware encoding is unavailable; using software encoding. | — |
| E5005 | error | The encoder rejected these settings. | Use a preset |
| E5006 | error | The exported file did not pass verification. | Retry / report |
| E5007 | warning | The export range contains offline media. | Relink / continue |
| E5008 | error | Width and height must be even for this format. | Adjust resolution |
| E6001 | warning | A background task stopped unexpectedly and will retry. | — |
| E6002 | warning | A background task stopped responding and was restarted. | — |
| E6003 | error | A background task failed after retries. | View log |
| E6004 | info | The task was cancelled. | — |
| E6005 | error | A background task sent an invalid message. | Report a problem |
| E7001 | info | The speech model is not installed. | Open model manager |
| E7002 | warning | The model download failed; it can resume. | Retry |
| E7003 | error | The model file failed verification and was deleted. | Download again |
| E7004 | error | Transcription failed. | View log |
| E7005 | error | Not enough memory for this model. | Choose a smaller model |
| E7006 | info | GPU transcription is unavailable; using the CPU. | — |
| E8001 | error | The disk is full. | Free space |
| E8002 | warning | Settings were damaged and have been restored. | — |
| E8003 | warning | The cache was damaged and has been rebuilt. | — |
| E8004 | error | Access to "{path}" was denied. | Choose another location |
| E8005 | warning | The cache folder is unavailable; using the default folder. | Settings |
| E9001 | info | Could not check for updates. | Retry later |
| E9002 | error | The update package signature is invalid; it was not installed. | Report a problem |
| E9003 | info | The update will install after the export finishes and the project is saved. | — |
| E9004 | warning | A network request was blocked by privacy settings. | Settings |
| E9005 | error | A file path was rejected for security reasons. | — |
| E9006 | error | An internal request was rejected (invalid data). | Report a problem |

## 12.14 Locales (LOC)

| ID | Tag | Language | Direction | Status |
|---|---|---|---|---|
| LOC-01 | `en` | English | LTR | Source locale (keys authored here) |
| LOC-02 | `ar` | Arabic (Modern Standard) | RTL | Shipped |
| LOC-03 | `fr` | French | LTR | Shipped |
| LOC-04 | `es` | Spanish | LTR | Shipped |
| LOC-05 | `de` | German | LTR | Shipped |
| LOC-06 | `pt-BR` | Portuguese (Brazil) | LTR | Shipped |
| LOC-90 | `en-XA` | Pseudo-locale (accented, +40% length, brackets) | LTR | Tests only |
| LOC-91 | `ar-XB` | Pseudo-locale (RTL, mirrored, bidi marks) | RTL | Tests only |

Adding a locale: add the catalog file, translate 100% of keys, add a row here through a docs pull request (G-19). Rules for translators: 07 §7.12.

## 12.15 Speech-to-text models (MDL)

Files come from the `ggerganov/whisper.cpp` model repository on Hugging Face (MIT license). The download URL pattern is `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/<file>`. The SHA-256 column lists expected values; WP-21 MUST recompute every hash from the downloaded file with `tools/models/hash-models.py` and stop if a value differs (G-10). The manifest file `assets/models/whisper.json` is generated from this table.

| ID | Model | File | Size (approx.) | Expected SHA-256 | Guidance |
|---|---|---|---|---|---|
| MDL-01 | Tiny (quantized) | `ggml-tiny-q5_1.bin` | 31 MB | `818710568da3ca15689e31a743197b520007872ff9576237bda97bd1b469c3d7` | Fastest, lowest accuracy |
| MDL-02 | Base (quantized) | `ggml-base-q5_1.bin` | 57 MB | `422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898` | Fast drafts |
| MDL-03 | Small (quantized) | `ggml-small-q5_1.bin` | 181 MB | `ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb` | **Default** (OD-09); good for Arabic and French |
| MDL-04 | Medium (quantized) | `ggml-medium-q5_0.bin` | 514 MB | `19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f` | High accuracy, slower |
| MDL-05 | Large v3 Turbo (quantized) | `ggml-large-v3-turbo-q5_0.bin` | ≈ 550 MB | recorded at WP-21 by `hash-models.py` | Best accuracy per speed; GPU recommended |

## 12.16 Fonts (FNT)

All fonts are SIL Open Font License 1.1, bundled as separate files under `assets/fonts/` with their license files, and listed in the notices manifest. Source: the official upstream repositories of each family (the exact URL and SHA-256 of each file are recorded in `assets/fonts/fonts.json` at WP-04/WP-17).

| ID | Family | Use |
|---|---|---|
| FNT-01 | Inter (variable) | UI Latin text |
| FNT-02 | Noto Sans Arabic (variable) | UI Arabic text |
| FNT-03 | JetBrains Mono | Timecode and numeric readouts |
| FNT-04 | Montserrat | Titles |
| FNT-05 | Playfair Display | Titles (serif) |
| FNT-06 | Bebas Neue | Titles (display) |
| FNT-07 | Cairo | Titles (Arabic + Latin) and default caption font |
| FNT-08 | Amiri | Titles (Arabic, classical) |

## 12.17 Colors (CLR)

Label and marker colors (same palette in all themes; text on them uses the contrasting color in the last column):

| ID | Name | Hex | Text |
|---|---|---|---|
| CLR-01 | Red | `#E5484D` | white |
| CLR-02 | Orange | `#F76B15` | black |
| CLR-03 | Yellow | `#FFC53D` | black |
| CLR-04 | Green | `#30A46C` | white |
| CLR-05 | Cyan | `#05A2C2` | black |
| CLR-06 | Blue | `#3E63DD` | white |
| CLR-07 | Purple | `#8E4EC6` | white |
| CLR-08 | Pink | `#D6409F` | white |

Accent colors offered in Appearance settings: CLR-06 (default), CLR-07, CLR-04, CLR-02, CLR-08. Theme tokens derived from the accent are defined in 07 §7.4.

## 12.18 Built-in LUT looks (LUT)

Generated by `tools/assets/generate_luts.py` (33³ `.cube`, input/output Rec.709 display-referred) from the formulas below; they are original works of this project.

| ID | Name | Formula (applied per pixel, then clamped) |
|---|---|---|
| LUT-01 | Warm Lift | R×1.04, B×0.94, then lift shadows by +0.02 on all channels |
| LUT-02 | Cool Breeze | R×0.95, B×1.05, contrast S-curve strength 0.10 |
| LUT-03 | Teal & Orange | push hues near 25° (skin) +8% saturation; shift shadows toward teal (−0.03 R, +0.02 B) |
| LUT-04 | Faded Film | out = 0.06 + 0.88·in, saturation ×0.85 |
| LUT-05 | Mono Contrast | Rec.709 luma, S-curve strength 0.20 |

## 12.19 Hardware encoder mapping

| Codec family | NVIDIA | AMD | Intel | Software |
|---|---|---|---|---|
| H.264 | `h264_nvenc` | `h264_amf` | `h264_qsv` | `libx264` |
| HEVC | `hevc_nvenc` | `hevc_amf` | `hevc_qsv` | `libx265` |
| AV1 | `av1_nvenc` (GPU generation dependent) | `av1_amf` (generation dependent) | `av1_qsv` (generation dependent) | `libsvtav1` |

Quality mapping from a preset's CRF to hardware constant-quality values is defined in 02 §2.9.2 and validated by T-EXP-002.

## 12.20 Guaranteed input formats

The FFmpeg build (02 ADR-004) MUST decode at least the following; T-MED-001 generates or uses a sample for each.

| Kind | Formats |
|---|---|
| Containers | MP4, MOV, MKV, WebM, AVI, MXF, MPEG-TS (`.ts`, `.mts`, `.m2ts`), MPEG-PS (`.mpg`), WMV/ASF, FLV, 3GP, Ogg |
| Video codecs | H.264, HEVC (8/10-bit), AV1, VP8, VP9, ProRes (all profiles), DNxHD/DNxHR, MPEG-2, MPEG-4 Part 2, Motion JPEG, FFV1, VC-1/WMV3 |
| Images | PNG (8/16-bit, alpha), JPEG, WebP, TIFF, BMP, GIF (animated as clip), TGA, OpenEXR (half/float) |
| Audio | WAV (PCM 16/24/32f), AIFF, MP3, AAC/M4A, FLAC, Ogg Vorbis, Opus, WMA, AC-3/E-AC-3 |
| Captions | SRT, WebVTT, ASS/SSA (REQ-CAP-003) |
| LUTs | `.cube` (REQ-INT-003) |
