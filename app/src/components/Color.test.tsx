// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Color + curve tests (07 §7.6): the picker renders its HSV area,
 * hue slider, hex field, and recent swatches and reports hex changes;
 * the wheel wraps angular deltas, resets on double-click; the curve
 * editor's point helpers keep x order and clamp, click adds, Delete
 * removes; PointControl rows X/Y + handle.
 */
import { fireEvent, render } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { ColorPicker } from "./ColorPicker";
import { angleDelta, ColorWheel } from "./ColorWheel";
import {
  CurveEditor,
  DEFAULT_CURVE,
  insertPoint,
  movePoint,
  removePoint,
  type Curve,
} from "./CurveEditor";
import { PointControl } from "./PointControl";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("ColorPicker (07 s7.6)", () => {
  it("renders the hex field, hue slider, and recent swatches", () => {
    const { container } = render(
      <Wrapper>
        <ColorPicker
          value="#ff8800"
          onChange={() => {}}
          recent={["#ff8800", "#0033cc"]}
        />
      </Wrapper>,
    );
    const hex = container.querySelector("input");
    expect(hex).not.toBeNull();
    // The hue slider is one of the range inputs (max 360).
    const ranges = container.querySelectorAll("input[type='range']");
    expect(ranges.length).toBeGreaterThanOrEqual(3);
    // Recent swatches render as options of the swatch picker listbox.
    const swatches = container.querySelectorAll("[role='option']");
    expect(swatches.length).toBe(2);
  });

  it("exposes the hue thumb with its value text", () => {
    render(
      <Wrapper>
        <ColorPicker value="#ff8800" onChange={() => {}} />
      </Wrapper>,
    );
    const hue = hueSlider();
    expect(hue.getAttribute("aria-valuetext")).toContain("32");
  });
});

function hueSlider(): HTMLElement {
  const sliders = Array.from(document.querySelectorAll("input")).filter(
    (el) =>
      el.getAttribute("type") === "range" && el.getAttribute("max") === "360",
  );
  const el = sliders[0];
  if (!(el instanceof HTMLElement)) {
    throw new TypeError("expected the hue slider");
  }
  return el;
}

describe("ColorWheel (07 s7.6)", () => {
  it("wraps angular deltas into (-180, 180]", () => {
    expect(angleDelta(0, 90)).toBe(90);
    expect(angleDelta(350, 10)).toBe(20);
    expect(angleDelta(10, 350)).toBe(-20);
  });

  it("resets to the default color on double-click", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <ColorWheel
          value="#ff8800"
          defaultValue="#3366cc"
          onChange={onChange}
        />
      </Wrapper>,
    );
    const wrap = container.firstElementChild?.firstElementChild;
    if (!(wrap instanceof HTMLElement)) {
      throw new TypeError("expected the wheel wrap");
    }
    fireEvent.doubleClick(wrap);
    expect(onChange).toHaveBeenCalledWith("#3366cc");
  });
});

describe("CurveEditor helpers (07 s7.6)", () => {
  it("inserts points sorted by x", () => {
    const base = DEFAULT_CURVE.y;
    const next = insertPoint(base, { x: 0.5, y: 0.25 });
    expect(next.map((p) => p.x)).toEqual([0, 0.5, 1]);
    expect(next[1]).toEqual({ x: 0.5, y: 0.25 });
  });

  it("removes points by index", () => {
    expect(removePoint(DEFAULT_CURVE.y, 0).length).toBe(1);
  });

  it("moves points clamped to [0,1] without crossing neighbours", () => {
    const points = [
      { x: 0, y: 0 },
      { x: 0.5, y: 0.5 },
      { x: 1, y: 1 },
    ];
    const moved = movePoint(points, 1, { x: -10, y: 10 });
    expect(moved[1]).toEqual({ x: 0, y: 1 });
  });
});

describe("CurveEditor component (07 s7.6)", () => {
  it("renders channel tabs and the two default points", () => {
    const { container } = render(
      <Wrapper>
        <CurveEditor value={DEFAULT_CURVE} onChange={() => {}} />
      </Wrapper>,
    );
    expect(container.querySelectorAll("[role='tab']").length).toBe(4);
    const points = container.querySelectorAll("circle");
    expect(points.length).toBe(2);
  });

  it("adds a point on a click on empty curve space", () => {
    const onChange = vi.fn<(curve: Curve) => void>();
    const { container } = render(
      <Wrapper>
        <CurveEditor value={DEFAULT_CURVE} onChange={onChange} />
      </Wrapper>,
    );
    const svg = container.querySelector("svg");
    if (!(svg instanceof SVGSVGElement)) {
      throw new TypeError("expected the curve svg");
    }
    fireEvent.pointerDown(svg, { clientX: 80, clientY: 80 });
    expect(onChange).toHaveBeenCalled();
    const curve = onChange.mock.calls[0]?.[0];
    expect(curve.y.length).toBe(3);
  });

  it("removes the focused point with Delete", () => {
    const onChange = vi.fn<(curve: Curve) => void>();
    const { container } = render(
      <Wrapper>
        <CurveEditor value={DEFAULT_CURVE} onChange={onChange} />
      </Wrapper>,
    );
    const point = container.querySelectorAll("circle")[0];
    fireEvent.focus(point);
    fireEvent.keyDown(point, { key: "Delete" });
    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls[0]?.[0].y.length).toBe(1);
  });
});

describe("PointControl (07 s7.6)", () => {
  it("renders X/Y fields and the handle select", () => {
    const { container } = render(
      <Wrapper>
        <PointControl
          x={25}
          y={75}
          onX={() => {}}
          onY={() => {}}
          handle="smooth"
          onHandle={() => {}}
        />
      </Wrapper>,
    );
    const inputs = Array.from(
      container.querySelectorAll<HTMLInputElement>("input"),
    ).filter((el) => el.getAttribute("type") !== "range");
    expect(inputs.length).toBe(2);
    expect(inputs[0].value).toContain("25");
    expect(inputs[1].value).toContain("75");
  });
});
