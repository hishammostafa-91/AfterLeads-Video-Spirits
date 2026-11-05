// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Phase-1 smoke test: the Vitest pipeline renders the app root.
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../App";

describe("app root", () => {
  it("renders the main container", () => {
    const { container } = render(<App />);
    const main = container.querySelector("main.container");
    expect(main).not.toBeNull();
  });
});
