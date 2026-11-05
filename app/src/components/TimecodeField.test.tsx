// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * TimecodeField tests (07 §7.6): segment conversion round-trips,
 * typed absolute/relative entry parsing, arrow stepping with
 * rollover, and the pinned `dir="ltr"` (non-mirrored surface,
 * docs/07 §7.11).
 */
import { fireEvent, render } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import {
  framesToTimecode,
  parseTimecodeEntry,
  TimecodeField,
  timecodeToFrames,
} from "./TimecodeField";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

const FPS = 24;

describe("conversion helpers (07 s7.6)", () => {
  it("round-trips segments through frames", () => {
    const tc = { hours: 1, minutes: 2, seconds: 3, frames: 12 };
    expect(framesToTimecode(timecodeToFrames(tc, FPS), FPS)).toEqual(tc);
  });

  it("parses full and partial absolute timecodes", () => {
    expect(parseTimecodeEntry("01:02:03:12", FPS)).toEqual({
      kind: "absolute",
      frames: timecodeToFrames(
        { hours: 1, minutes: 2, seconds: 3, frames: 12 },
        FPS,
      ),
    });
    expect(parseTimecodeEntry("00:00:10:00", FPS)).toEqual({
      kind: "absolute",
      frames: 10 * FPS,
    });
  });

  it("parses +N / -N relative frame entry", () => {
    expect(parseTimecodeEntry("+12", FPS)).toEqual({
      kind: "relative",
      delta: 12,
    });
    expect(parseTimecodeEntry("-3", FPS)).toEqual({
      kind: "relative",
      delta: -3,
    });
  });

  it("rejects invalid text", () => {
    expect(parseTimecodeEntry("abc", FPS)).toBeNull();
    expect(parseTimecodeEntry("00:99:00:00", FPS)).toBeNull();
    expect(parseTimecodeEntry("00:00:00:24", FPS)).toBeNull();
  });
});

describe("TimecodeField (07 s7.6; s7.11 dir=ltr)", () => {
  it("renders four two-digit segments inside a pinned LTR field", () => {
    const { container } = render(
      <Wrapper>
        <TimecodeField
          value={timecodeToFrames(
            { hours: 1, minutes: 2, seconds: 3, frames: 4 },
            FPS,
          )}
          onChange={() => {}}
          fps={FPS}
        />
      </Wrapper>,
    );
    const field = container.querySelector("div[dir='ltr']");
    expect(field).not.toBeNull();
    const inputs = field?.querySelectorAll("input");
    expect(inputs?.length).toBe(4);
    expect(inputs?.[0].value).toBe("01");
    expect(inputs?.[1].value).toBe("02");
    expect(inputs?.[2].value).toBe("03");
    expect(inputs?.[3].value).toBe("04");
  });

  it("steps the focused segment with ArrowUp/ArrowDown", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <TimecodeField
          value={timecodeToFrames(
            { hours: 0, minutes: 0, seconds: 0, frames: 0 },
            FPS,
          )}
          onChange={onChange}
          fps={FPS}
        />
      </Wrapper>,
    );
    const frames = container.querySelectorAll("input")[3];
    fireEvent.keyDown(frames, { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith(1);
    fireEvent.keyDown(frames, { key: "ArrowDown" });
    expect(onChange).toHaveBeenLastCalledWith(23);
  });

  it("commits typed relative entry on Enter", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <TimecodeField value={100} onChange={onChange} fps={FPS} />
      </Wrapper>,
    );
    const first = container.querySelectorAll("input")[0];
    fireEvent.change(first, { target: { value: "+12" } });
    fireEvent.keyDown(first, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith(112);
  });
});
