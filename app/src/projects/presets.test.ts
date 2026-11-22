// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Catalog tests (T024, D1 pattern): the fixtures carry exactly the
 * 12 s12.8/s12.9 tables - ids in order, exact dimensions and rates -
 * and every title key is seeded in the locales.
 */
import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import {
  DEFAULT_FRAME_RATE_ID,
  DEFAULT_PRESET_ID,
  DEFAULT_SAMPLE_RATE,
  FRAME_RATES,
  SAMPLE_RATES,
  SEQUENCE_PRESETS,
  frameRateById,
  presetById,
} from "./presets";

describe("t_prj_001 — sequence preset catalog (12 s12.8)", () => {
  it("lists SQP-01..12 in order with the doc dimensions", () => {
    expect(SEQUENCE_PRESETS.map((preset) => preset.id)).toEqual([
      "SQP-01",
      "SQP-02",
      "SQP-03",
      "SQP-04",
      "SQP-05",
      "SQP-06",
      "SQP-07",
      "SQP-08",
      "SQP-09",
      "SQP-10",
      "SQP-11",
      "SQP-12",
    ]);
    expect(presetById("SQP-02")).toMatchObject({
      width: 1920,
      height: 1080,
      aspect: "16:9",
      autoMatch: false,
    });
    expect(presetById("SQP-01")?.autoMatch).toBe(true);
    expect(presetById("SQP-01")?.width).toBeNull();
    expect(presetById("SQP-10")).toMatchObject({ width: 3840, height: 1608 });
    expect(presetById("SQP-13")).toBeUndefined();
  });

  it("every preset has a seeded name key", () => {
    for (const preset of SEQUENCE_PRESETS) {
      expect(en[`presets.sqp.${preset.id}` as keyof typeof en]).toBeTruthy();
    }
  });
});

describe("t_prj_001 — frame rate catalog (12 s12.9)", () => {
  it("lists FPS-01..08 with exact rationals", () => {
    expect(FRAME_RATES.map((rate) => rate.id)).toEqual([
      "FPS-01",
      "FPS-02",
      "FPS-03",
      "FPS-04",
      "FPS-05",
      "FPS-06",
      "FPS-07",
      "FPS-08",
    ]);
    expect(frameRateById("FPS-01")).toMatchObject({
      num: 24000,
      den: 1001,
      dropFrameDefault: false,
    });
    expect(frameRateById("FPS-04")?.dropFrameDefault).toBe(true);
    expect(frameRateById("FPS-07")?.dropFrameDefault).toBe(true);
    expect(frameRateById("FPS-09")).toBeUndefined();
  });

  it("the DLG-01 defaults match the doc", () => {
    expect(DEFAULT_PRESET_ID).toBe("SQP-01");
    expect(DEFAULT_FRAME_RATE_ID).toBe("FPS-05");
    expect(DEFAULT_SAMPLE_RATE).toBe(48_000);
    expect(SAMPLE_RATES).toEqual([44_100, 48_000]);
  });

  it("every frame rate has a seeded name key", () => {
    for (const rate of FRAME_RATES) {
      expect(en[`presets.fps.${rate.id}` as keyof typeof en]).toBeTruthy();
    }
  });
});
