// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The sequence-preset and frame-rate catalogs (12 s12.8/s12.9) as
 * typed accessors over doc-derived fixture files (the D1 pattern:
 * the fixture is regenerated from the doc, never hand-edited).
 * DLG-01 offers SQP-01..12; SQP-13 (custom) needs the explicit
 * format of the sequence-settings dialog and is not offered here.
 */
import sqpFixture from "./presets.sqp.fixture.json";
import fpsFixture from "./presets.fps.fixture.json";

/** One sequence preset row (12 s12.8). SQP-01 is auto-match: its
 * dimensions are null until the first clip sets them (the SQP-02
 * fallback applies on creation). */
export interface SequencePreset {
  id: string;
  width: number | null;
  height: number | null;
  /** Aspect label, catalog spelling (not translated). */
  aspect: string | null;
  autoMatch: boolean;
}

/** One frame-rate row (12 s12.9). */
export interface FrameRate {
  id: string;
  /** Display string, catalog spelling (not translated). */
  display: string;
  num: number;
  den: number;
  dropFrameDefault: boolean;
}

/** The SQP-01..12 list in catalog order. */
export const SEQUENCE_PRESETS: readonly SequencePreset[] = sqpFixture.map(
  (row) => ({ ...row }),
);

/** The FPS-01..08 list in catalog order. */
export const FRAME_RATES: readonly FrameRate[] = fpsFixture.map((row) => ({
  ...row,
}));

/** The DLG-01 defaults (12 s12.8: auto preset; 48 kHz audio). */
export const DEFAULT_PRESET_ID = "SQP-01";
export const DEFAULT_FRAME_RATE_ID = "FPS-05";
export const DEFAULT_SAMPLE_RATE = 48_000;
export const SAMPLE_RATES: readonly number[] = [44_100, DEFAULT_SAMPLE_RATE];

export function presetById(id: string): SequencePreset | undefined {
  return SEQUENCE_PRESETS.find((preset) => preset.id === id);
}

export function frameRateById(id: string): FrameRate | undefined {
  return FRAME_RATES.find((rate) => rate.id === id);
}

/** The locale key of a preset's name. */
export function presetLabelKey(id: string): string {
  return `presets.sqp.${id}`;
}

/** The locale key of a frame rate's name. */
export function frameRateLabelKey(id: string): string {
  return `presets.fps.${id}`;
}
