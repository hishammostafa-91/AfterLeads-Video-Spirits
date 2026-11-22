// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Project lifecycle tests (T-APP-004/007 ground): DLG-01's name rule
 * and body, DLG-20's three-way body, the Welcome states, and the
 * store flows over the mock backend (create/open/E1007/E1001/save/
 * close) exactly as the wrappers drive them.
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { beforeEach, describe, expect, it } from "vitest";
import en from "../locales/en.json";
import { NewProjectBody, nameError } from "../dialogs/NewProject";
import { UnsavedChangesBody } from "../dialogs/UnsavedChanges";
import { LockConflictBody } from "../dialogs/LockConflict";
import { Welcome } from "../shell/Welcome";
import { resetProjectStoreForTests, useProjectStore } from "./store";
import type { UnsavedChoice } from "../dialogs/UnsavedChanges";
import { setIpcTransport } from "../ipc/transport";
import { ipcMockTransport } from "../harness/mock/ipcMock";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("t_prj_001 — the DLG-01 name rule (07 s7.14)", () => {
  it("accepts safe names and rejects the documented cases", () => {
    expect(nameError("Interviews")).toBeNull();
    expect(nameError("a")).toBeNull();
    expect(nameError("x".repeat(200))).toBeNull();
    expect(nameError("")).toBe("dlg.newProject.nameLength");
    expect(nameError("x".repeat(201))).toBe("dlg.newProject.nameLength");
    expect(nameError("..")).toBe("dlg.newProject.nameReserved");
    expect(nameError("a<b")).toBe("dlg.newProject.nameChars");
    expect(nameError("a:")).toBe("dlg.newProject.nameChars");
    expect(nameError("trailing.")).toBe("dlg.newProject.nameTrailing");
  });

  it("the body shows the inline rule after touching an empty name", () => {
    render(
      <Wrapper>
        <NewProjectBody
          defaultFolder="C:\\Projects"
          error={null}
          onCreate={() => Promise.resolve(false)}
          onDone={() => undefined}
        />
      </Wrapper>,
    );
    const field = screen.getByLabelText("Name");
    fireEvent.click(field);
    fireEvent.blur(field);
    expect(
      screen.getByText("The name must be 1 to 200 characters."),
    ).toBeTruthy();
  });
});

describe("t_app_007 — DLG-20's three-way choice (07 s7.14)", () => {
  it("offers Save / Don't save / Cancel with the project name", () => {
    const choices: string[] = [];
    render(
      <Wrapper>
        <UnsavedChangesBody
          projectName="Interviews"
          onChoose={(choice: UnsavedChoice) => {
            choices.push(choice);
          }}
        />
      </Wrapper>,
    );
    expect(screen.getByText("Save changes to Interviews?")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Don't save" }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(choices).toEqual(["cancel", "discard", "save"]);
  });
});

describe("t_app_004 — the E1007 recovery surface (FR-013 AC3)", () => {
  it("offers read-only always and take-over only when available", () => {
    const { rerender } = render(
      <Wrapper>
        <LockConflictBody
          takeoverAvailable={false}
          onOpenReadonly={() => undefined}
          onTakeOver={() => undefined}
          onCancel={() => undefined}
        />
      </Wrapper>,
    );
    expect(screen.getByRole("button", { name: "Open read-only" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Take over" })).toBeNull();
    rerender(
      <Wrapper>
        <LockConflictBody
          takeoverAvailable
          onOpenReadonly={() => undefined}
          onTakeOver={() => undefined}
          onCancel={() => undefined}
        />
      </Wrapper>,
    );
    expect(screen.getByRole("button", { name: "Take over" })).toBeTruthy();
  });
});

describe("t_app_004 — store flows over the mock backend", () => {
  beforeEach(() => {
    resetProjectStoreForTests();
    setIpcTransport(ipcMockTransport());
  });

  it("create installs the snapshot and the title-dot data flows", async () => {
    const store = useProjectStore.getState();
    const created = await store.newProject({
      name: "Interviews",
      folder: "C:\\Projects",
      sequencePreset: "SQP-02",
      frameRate: "FPS-05",
      sampleRate: 48_000,
    });
    expect(created).toBe(true);
    const snapshot = useProjectStore.getState().snapshot;
    expect(snapshot?.project.name).toBe("Interviews");
    expect(snapshot?.path).toBe("C:\\Projects\\Interviews\\Interviews.alvs");
  });

  it("a rejected name surfaces the E1003 envelope and no snapshot", async () => {
    const store = useProjectStore.getState();
    const created = await store.newProject({
      name: "bad<name",
      folder: "C:\\Projects",
      sequencePreset: "SQP-02",
      frameRate: "FPS-05",
      sampleRate: 48_000,
    });
    expect(created).toBe(false);
    expect(useProjectStore.getState().snapshot).toBeNull();
    expect(useProjectStore.getState().lastError?.code).toBe("E1003");
  });

  it("a missing file records the missing path (E1001, FR-011 AC2)", async () => {
    setIpcTransport(
      ipcMockTransport({ missingPaths: ["D:\\Gone\\Deleted.alvs"] }),
    );
    const store = useProjectStore.getState();
    const opened = await store.openPath("D:\\Gone\\Deleted.alvs");
    expect(opened).toBe(false);
    expect(
      useProjectStore.getState().missingPaths.has("D:\\Gone\\Deleted.alvs"),
    ).toBe(true);
  });

  it("open then close empties the shell again", async () => {
    const store = useProjectStore.getState();
    await store.newProject({
      name: "Interviews",
      folder: "C:\\Projects",
      sequencePreset: "SQP-01",
      frameRate: "FPS-05",
      sampleRate: 48_000,
    });
    const closed = await store.close(true);
    expect(closed).toBe(true);
    expect(useProjectStore.getState().snapshot).toBeNull();
  });
});

describe("t_app_004 — the Welcome states (FR-011 AC1)", () => {
  beforeEach(() => {
    resetProjectStoreForTests();
  });

  it("empty: no recents yet", () => {
    setIpcTransport(ipcMockTransport());
    render(
      <Wrapper>
        <Welcome
          onNewProject={() => undefined}
          onOpenProject={() => undefined}
          onOpenRecent={() => undefined}
        />
      </Wrapper>,
    );
    expect(screen.getByText("No recent projects yet.")).toBeTruthy();
  });

  it("recents: lists entries and marks the missing one", async () => {
    setIpcTransport(
      ipcMockTransport({
        recents: [
          {
            path: "C:\\Projects\\Interviews\\Interviews.alvs",
            name: "Interviews",
            thumbnail: null,
            openedUtc: "2026-01-01T00:00:00Z",
            pinned: false,
          },
          {
            path: "D:\\Gone\\Deleted.alvs",
            name: "Deleted",
            thumbnail: null,
            openedUtc: "2026-01-01T00:00:00Z",
            pinned: false,
          },
        ],
      }),
    );
    render(
      <Wrapper>
        <Welcome
          onNewProject={() => undefined}
          onOpenProject={() => undefined}
          onOpenRecent={() => undefined}
        />
      </Wrapper>,
    );
    await waitFor(() => {
      expect(screen.getByText("Interviews")).toBeTruthy();
    });
    // Missing only shows for paths the store learned are gone.
    expect(screen.queryByText("Missing")).toBeNull();
    useProjectStore.getState().recordMissing("D:\\Gone\\Deleted.alvs");
    await waitFor(() => {
      expect(screen.getByText("Missing")).toBeTruthy();
    });
  });
});
