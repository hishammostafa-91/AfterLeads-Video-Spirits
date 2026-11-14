// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Collections + feedback tests (07 §7.6): the four RAC collections
 * render their semantics, the useWindowing hook virtualizes only
 * beyond the 200-row boundary (200 renders all, 201 windows), the
 * feedback family exposes progress/meter/spinner roles with the
 * spinner determinate only once a value exists, badges pair text with
 * an icon (never color alone), and EmptyState shows icon, sentence,
 * action, and shortcut.
 */
import { fireEvent, render } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { FormattedMessage, IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { Badge } from "./Badge";
import {
  GridList,
  GridListItem,
  ListBox,
  ListBoxOption,
  Table,
  TableCell,
  TableColumn,
  TableHeader,
  TableRow,
  TableBody,
  Tree,
  TreeItem,
  useWindowing,
  WindowedListBox,
} from "./Collections";
import { EmptyState } from "./EmptyState";
import { Meter, ProgressBar, Skeleton, Spinner } from "./Feedback";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("Collections (07 s7.6)", () => {
  it("renders a tree with treeitems and nested levels", () => {
    const { container } = render(
      <Wrapper>
        <Tree aria-label="Media">
          <TreeItem textValue="bin">
            {"Bin"}
            <TreeItem textValue="clip">{"Clip"}</TreeItem>
          </TreeItem>
        </Tree>
      </Wrapper>,
    );
    expect(container.querySelector("[role='tree']")).not.toBeNull();
    const items = container.querySelectorAll("[role='treeitem']");
    expect(items.length).toBe(2);
    expect(items[0].getAttribute("aria-level")).toBe("1");
    expect(items[0].getAttribute("aria-expanded")).toBe("false");
    expect(items[1].getAttribute("aria-level")).toBe("2");
  });

  it("expands and collapses with the arrow keys (APG tree)", () => {
    const { container } = render(
      <Wrapper>
        <Tree aria-label="Media">
          <TreeItem textValue="bin">
            {"Bin"}
            <TreeItem textValue="clip">{"Clip"}</TreeItem>
          </TreeItem>
        </Tree>
      </Wrapper>,
    );
    const tree = container.querySelector("[role='tree']");
    if (!(tree instanceof HTMLElement)) {
      throw new TypeError("expected the tree");
    }
    const parent = container.querySelectorAll("[role='treeitem']")[0];
    if (!(parent instanceof HTMLElement)) {
      throw new TypeError("expected the treeitem");
    }
    // Collapsed: the nested item's group is hidden.
    const visible = (): number =>
      Array.from(container.querySelectorAll("[role='treeitem']")).filter(
        (el) => el.closest("[hidden]") === null,
      ).length;
    expect(visible()).toBe(1);
    parent.focus();
    fireEvent.keyDown(tree, { key: "ArrowRight" });
    expect(parent.getAttribute("aria-expanded")).toBe("true");
    expect(container.querySelectorAll("[role='treeitem']").length).toBe(2);
    fireEvent.keyDown(tree, { key: "ArrowLeft" });
    expect(parent.getAttribute("aria-expanded")).toBe("false");
  });

  it("renders a grid list with rows", () => {
    const { container } = render(
      <Wrapper>
        <GridList aria-label="Bin">
          <GridListItem textValue="Item A">{"Item A"}</GridListItem>
          <GridListItem textValue="Item B">{"Item B"}</GridListItem>
        </GridList>
      </Wrapper>,
    );
    expect(
      container.querySelectorAll("[role='row']").length,
    ).toBeGreaterThanOrEqual(2);
  });

  it("renders a table with columns and rows", () => {
    const { container } = render(
      <Wrapper>
        <Table aria-label="Clips">
          <TableHeader>
            <TableColumn>{"Name"}</TableColumn>
            <TableColumn>{"Duration"}</TableColumn>
          </TableHeader>
          <TableBody>
            <TableRow>
              <TableCell>{"Clip A"}</TableCell>
              <TableCell>{"10s"}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </Wrapper>,
    );
    expect(container.querySelectorAll("th").length).toBe(2);
    expect(container.querySelectorAll("td").length).toBe(2);
  });

  it("renders a listbox with options", () => {
    const { container } = render(
      <Wrapper>
        <ListBox aria-label="Tracks">
          <ListBoxOption id="v1">{"Video 1"}</ListBoxOption>
          <ListBoxOption id="a1">{"Audio 1"}</ListBoxOption>
        </ListBox>
      </Wrapper>,
    );
    expect(container.querySelectorAll("[role='option']").length).toBe(2);
  });
});

describe("useWindowing (200/201 boundary, research D10)", () => {
  const OPTIONS = { rowHeight: 28 };

  function harness(items: readonly number[]) {
    const result = { rendered: 0, windowed: false as boolean, total: 0 };
    function Probe(): JSX.Element {
      const w = useWindowing(items, OPTIONS);
      result.rendered = w.rendered.length;
      result.windowed = w.windowed;
      result.total = w.totalHeight;
      return (
        <div {...w.containerProps}>
          <span>{w.rendered.length}</span>
        </div>
      );
    }
    render(<Probe />);
    return result;
  }

  it("renders every row at the threshold (200)", () => {
    const r = harness(Array.from({ length: 200 }, (_, i) => i));
    expect(r.windowed).toBe(false);
    expect(r.rendered).toBe(200);
  });

  it("windows the row above the threshold (201)", () => {
    const r = harness(Array.from({ length: 201 }, (_, i) => i));
    expect(r.windowed).toBe(true);
    expect(r.rendered).toBeLessThan(201);
    expect(r.rendered).toBeGreaterThan(0);
  });
});

describe("WindowedListBox", () => {
  it("keeps full-list semantics through aria-setsize", () => {
    const items = Array.from({ length: 250 }, (_, i) => `row-${String(i + 1)}`);
    const { container } = render(
      <WindowedListBox
        items={items}
        keyOf={(item) => item}
        renderOption={(item) => item}
        aria-label="Big"
      />,
    );
    const options = container.querySelectorAll("[role='option']");
    expect(options.length).toBeLessThan(250);
    expect(options[0].getAttribute("aria-setsize")).toBe("250");
    expect(options[0].getAttribute("aria-posinset")).toBe("1");
  });

  it("renders all options below the threshold", () => {
    const items = ["a", "b", "c"];
    const { container } = render(
      <WindowedListBox
        items={items}
        keyOf={(item) => item}
        renderOption={(item) => item}
        aria-label="Small"
      />,
    );
    expect(container.querySelectorAll("[role='option']").length).toBe(3);
  });
});

describe("Feedback (07 s7.6)", () => {
  it("exposes a determinate progressbar", () => {
    const { container } = render(
      <Wrapper>
        <ProgressBar label="Render" value={40} />
      </Wrapper>,
    );
    const bar = container.querySelector("[role='progressbar']");
    expect(bar).not.toBeNull();
    expect(bar?.getAttribute("aria-valuenow")).toBe("40");
  });

  it("exposes a meter without progress semantics", () => {
    const { container } = render(
      <Wrapper>
        <Meter label="Cache" value={70} />
      </Wrapper>,
    );
    expect(container.querySelector("[role='meter']")).not.toBeNull();
  });

  it("spinner is indeterminate until a value exists", () => {
    const { container, rerender } = render(
      <Wrapper>
        <Spinner />
      </Wrapper>,
    );
    const spin = () => container.querySelector("[role='progressbar']");
    expect(spin()?.getAttribute("aria-valuenow")).toBeNull();
    rerender(
      <Wrapper>
        <Spinner value={80} />
      </Wrapper>,
    );
    expect(spin()?.getAttribute("aria-valuenow")).toBe("80");
  });

  it("renders skeleton placeholders", () => {
    const { container } = render(
      <Wrapper>
        <Skeleton />
        <Skeleton shape="block" inlineSize={16} blockSize={4} />
      </Wrapper>,
    );
    expect(container.children.length).toBe(2);
  });
});

describe("Badge (never color alone)", () => {
  it("pairs an icon with text for every kind", () => {
    for (const kind of ["proxy", "offline", "tenBit", "hdr", "vfr"] as const) {
      const { container } = render(
        <Wrapper>
          <Badge kind={kind} />
        </Wrapper>,
      );
      expect(container.querySelector("svg")).not.toBeNull();
      expect(container.textContent).not.toBe("");
    }
  });
});

describe("EmptyState (07 s7.6)", () => {
  it("shows icon, sentence, action, and shortcut hint", () => {
    const onAction = vi.fn();
    const { container, getByText } = render(
      <Wrapper>
        <EmptyState
          message={<FormattedMessage id="components.empty.message" />}
          actionMessageId="components.empty.action"
          onAction={onAction}
          shortcut="Ctrl+I"
        />
      </Wrapper>,
    );
    expect(container.querySelector("svg")).not.toBeNull();
    expect(getByText("Import media")).toBeTruthy();
    expect(container.querySelector("kbd")?.textContent).toBe("Ctrl+I");
    const action = container.querySelector("button");
    if (!(action instanceof HTMLButtonElement)) {
      throw new TypeError("expected the empty-state action");
    }
    fireEvent.click(action);
    expect(onAction).toHaveBeenCalled();
  });
});
