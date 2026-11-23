import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ProjectRow } from "../projects/types";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { EXPORT_TIMEOUT_MS } from "./consts";
import { ExportButton } from "./ExportButton";
import { exportFileName } from "./exportHtml";

const SCAN = { snapshot: 3, created: 0, mode: "full", files: 1, symbols: 2, references: 0 };
const PROJECT = {
  id: 7,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: SCAN,
  types: 0,
  loc: 0,
  languages: [],
  running: false,
} as unknown as ProjectRow;

useServerSession();

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("ExportButton", () => {
  it("downloads the page the command returns, named after the project", async () => {
    serve({ "export.html": () => answer({ html: "<html></html>", symbols: 2 }) });
    const created = vi.fn(() => "blob:x");
    vi.stubGlobal("URL", { createObjectURL: created, revokeObjectURL: vi.fn() });
    const names: string[] = [];
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (
      this: HTMLAnchorElement,
    ) {
      names.push(this.download);
    });
    render(() => <ExportButton project={PROJECT} />);
    fireEvent.click(screen.getByRole("button", { name: "Export HTML" }));
    await waitFor(() => expect(names).toEqual(["shop.html"]));
    expect(JSON.parse(fetchMock.mock.calls[0]?.[1].body)).toEqual({ project: 7 });
    click.mockRestore();
  });

  it("is shut without a scan", () => {
    render(() => <ExportButton project={{ ...PROJECT, scan: null }} />);
    expect(screen.getByRole("button", { name: "Export HTML" })).toBeDisabled();
  });

  it("shows the registered message of a failure", async () => {
    serve({ "export.html": () => answer({ code: "EXP-201" }, 500) });
    render(() => <ExportButton project={PROJECT} />);
    fireEvent.click(screen.getByRole("button", { name: "Export HTML" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("forgets the failure when another project is shown", async () => {
    serve({ "export.html": () => answer({ code: "EXP-201" }, 500) });
    const [project, setProject] = createSignal(PROJECT);
    render(() => <ExportButton project={project()} />);
    fireEvent.click(screen.getByRole("button", { name: "Export HTML" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    setProject({ ...PROJECT, id: 8, name: "api" });
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  });
});

/** A server that never answers: the call ends only when its signal aborts it. */
function hang(): void {
  fetchMock.mockImplementation(
    (_path: string, init: RequestInit) =>
      new Promise((_resolve, reject) => {
        init.signal?.addEventListener("abort", () => reject(init.signal?.reason));
      }),
  );
}

describe("an export that does not come back", () => {
  it("says so when the time is up and frees the button", async () => {
    vi.useFakeTimers();
    hang();
    render(() => <ExportButton project={PROJECT} />);
    fireEvent.click(screen.getByRole("button", { name: "Export HTML" }));
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    await vi.advanceTimersByTimeAsync(EXPORT_TIMEOUT_MS + 1);
    expect(screen.getByRole("alert")).toHaveTextContent("took too long");
    expect(screen.getByRole("button", { name: "Export HTML" })).toBeEnabled();
  });

  it("shows the seconds it has waited and stops quietly when cancelled", async () => {
    vi.useFakeTimers();
    hang();
    render(() => <ExportButton project={PROJECT} />);
    fireEvent.click(screen.getByRole("button", { name: "Export HTML" }));
    await vi.advanceTimersByTimeAsync(3000);
    expect(screen.getByText(/Exporting … 3 s/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("button", { name: "Export HTML" })).toBeEnabled();
  });
});

describe("exportFileName", () => {
  it("makes the project name safe and never empty", () => {
    expect(exportFileName("my repo/api")).toBe("my-repo-api.html");
    expect(exportFileName("///")).toBe("codeskimr-export.html");
  });

  it("avoids the device names of Windows and very long names", () => {
    expect(exportFileName("CON")).toBe("project-CON.html");
    expect(exportFileName("nul")).toBe("project-nul.html");
    expect(exportFileName("com1")).toBe("project-com1.html");
    expect(exportFileName("console")).toBe("console.html");
    expect(exportFileName("nul.backup")).toBe("project-nul.backup.html");
    expect(exportFileName("CON.v2")).toBe("project-CON.v2.html");
    expect(exportFileName("a".repeat(300)).length).toBe(100 + ".html".length);
  });
});
