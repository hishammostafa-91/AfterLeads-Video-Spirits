// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-I18N-003 (09 §9.10): ICU MessageFormat including Arabic's
 * zero/one/two/few/many/other; numbers and dates via `Intl` with
 * `numberingSystem: "latn"`; timecode in an LTR isolate inside RTL
 * prose; runtime language switch without reload; the default follows
 * the Windows display language when supported, else English
 * (REQ-I18N-001/003; 07 §7.11–7.12).
 */
import { act, render, screen, waitFor } from "@testing-library/react";
import type { JSX } from "react";
import { FormattedMessage } from "react-intl";
import { afterEach, describe, expect, it } from "vitest";
import { formatNumber, ltrIsolate } from "./format";
import { I18n, resolveDefaultLocale, useLocaleStore } from "./runtime";

const root = () => document.documentElement;

function StoryCount({ count }: { count: number }): JSX.Element {
  return <FormattedMessage id="harness.stories.count" values={{ count }} />;
}

afterEach(() => {
  act(() => {
    useLocaleStore.getState().setLocale("en");
  });
});

describe("t_i18n_003 ICU and formatting (REQ-I18N-003)", () => {
  it("selects Arabic's six plural categories", () => {
    const cases: [number, RegExp][] = [
      [0, /لا قصص/],
      [1, /قصة واحدة/],
      [2, /قصتان/],
      [3, /3 قصص/],
      [11, /11 قصة/],
      [100, /100 قصة/],
    ];
    for (const [count, pattern] of cases) {
      act(() => {
        useLocaleStore.getState().setLocale("ar");
      });
      const { unmount } = render(
        <I18n>
          <StoryCount count={count} />
        </I18n>,
      );
      expect(screen.getByText(pattern)).toBeTruthy();
      unmount();
    }
  });

  it("formats numbers with Western digits in every locale (latn)", () => {
    const arabic = formatNumber(12345.6, "ar");
    expect(arabic).not.toMatch(/[\u0660-\u0669]/);
    expect(arabic).toMatch(/12/);
    expect(formatNumber(1234, "en")).toBe("1,234");
  });

  it("wraps timecode in an LTR isolate for RTL prose", () => {
    const { container } = render(
      <I18n>
        <p>
          <FormattedMessage id="harness.title" /> {ltrIsolate("00:01:02:10")}
        </p>
      </I18n>,
    );
    const isolate = container.querySelector("bdi");
    expect(isolate).not.toBeNull();
    expect(isolate?.getAttribute("dir")).toBe("ltr");
    expect(isolate?.textContent).toBe("00:01:02:10");
  });
});

describe("t_i18n_003 runtime switching (REQ-I18N-001)", () => {
  it("switches language without reload and sets dir/lang", async () => {
    render(
      <I18n>
        <FormattedMessage id="harness.title" />
      </I18n>,
    );
    expect(screen.getByText("Design system harness")).toBeTruthy();
    act(() => {
      useLocaleStore.getState().setLocale("ar");
    });
    await waitFor(() => {
      expect(screen.getByText("حاضنة نظام التصميم")).toBeTruthy();
    });
    expect(root().dir).toBe("rtl");
    expect(root().lang).toBe("ar");
  });

  it("defaults to the Windows display language when supported, else en", () => {
    expect(resolveDefaultLocale(["ar-SA", "ar"])).toBe("ar");
    expect(resolveDefaultLocale(["fr-FR"])).toBe("fr");
    expect(resolveDefaultLocale(["pt-BR"])).toBe("pt-BR");
    expect(resolveDefaultLocale(["ja-JP"])).toBe("en");
    expect(resolveDefaultLocale([])).toBe("en");
  });
});
