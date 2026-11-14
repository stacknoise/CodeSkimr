import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { setLanguage } from "../../shared/i18n/i18n";
import { describe, expect, it, vi } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { ProjectsScreen } from "./ProjectsScreen";
import type { ProjectRow, ToolsInfo } from "./types";

/** What a scan reports when it did not check the tools. */
const UNCHECKED: ToolsInfo = { checked_at: null, languages: [], tools: [] };

const ROW: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: { snapshot: 5, created: 1_800_000_000, mode: "full", files: 3, symbols: 9, references: 4 },
  types: 433,
  loc: 41_208,
  languages: [
    { language: "csharp", files: 3, loc: 38_000 },
    { language: "xaml", files: 1, loc: 3_208 },
  ],
  running: false,
};

useServerSession();

describe("ProjectsScreen", () => {
  it("says how a first project gets here when there is none", async () => {
    serve({ "projects.list": () => answer([]) });
    render(() => <ProjectsScreen />);
    expect(await screen.findByText("No projects yet")).toBeInTheDocument();
    expect(screen.getByLabelText("New project")).toBeInTheDocument();
  });

  it("lists the projects with languages, counts and state", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    expect(await screen.findByText("shop")).toBeInTheDocument();
    expect(screen.getByText("/work/shop")).toBeInTheDocument();
    expect(screen.getByText("C# 92 % · xaml 8 %")).toBeInTheDocument();
    expect(screen.getByText("433")).toBeInTheDocument();
    expect(screen.getByText("41,208")).toBeInTheDocument();
    expect(screen.getByText("current")).toBeInTheDocument();
    expect(screen.getByText(/^1 project · 433 types · Enter opens/)).toBeInTheDocument();
  });

  it("shows a running scan and a project that was never scanned", async () => {
    serve({
      "projects.list": () =>
        answer([
          { ...ROW, running: true },
          { ...ROW, id: 2, name: "new", scan: null, types: 0, loc: 0, languages: [] },
        ]),
    });
    render(() => <ProjectsScreen />);
    expect(await screen.findByText("scan running")).toBeInTheDocument();
    expect(screen.getByText("not scanned")).toBeInTheDocument();
  });

  it("adds a project and reads the list again", async () => {
    let added = false;
    serve({
      "projects.list": () => answer(added ? [ROW] : []),
      "projects.add": () => {
        added = true;
        return answer({
          project: 1,
          snapshot: 5,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 1.4,
          tools: UNCHECKED,
        });
      },
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: " /work/shop " } });
    fireEvent.click(screen.getByRole("button", { name: "Add and scan" }));
    expect(await screen.findByText("shop")).toBeInTheDocument();
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.add"));
    expect(JSON.parse(call?.[1].body)).toEqual({ source: "/work/shop", branch: "main" });
    await waitFor(() => expect((field as HTMLInputElement).value).toBe(""));
  });

  it("tells a refusal in words and keeps what was typed", async () => {
    serve({
      "projects.list": () => answer([]),
      "projects.add": () => answer({ code: "PRJ-101" }, 400),
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "nowhere" } });
    fireEvent.click(screen.getByRole("button", { name: "Add and scan" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("neither an existing folder");
    expect((field as HTMLInputElement).value).toBe("nowhere");
  });

  it("puts a dropped path into the field, and explains a folder without one", async () => {
    serve({ "projects.list": () => answer([]) });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    const drop = (types: Record<string, string>, files: number) => {
      const event = new Event("drop", { cancelable: true });
      Object.assign(event, {
        dataTransfer: {
          getData: (type: string) => types[type] ?? "",
          files: { length: files },
          items: { length: files },
        },
      });
      window.dispatchEvent(event);
    };
    drop({ "text/uri-list": "file:///work/shop" }, 0);
    expect((field as HTMLInputElement).value).toBe("/work/shop");
    drop({}, 1);
    expect(await screen.findByText(/does not reveal the path/)).toBeInTheDocument();
  });

  it("scans the selected project again on R and moves with the arrow keys", async () => {
    serve({
      "projects.list": () => answer([ROW, { ...ROW, id: 2, name: "other" }]),
      "projects.rescan": () =>
        answer({
          project: 2,
          snapshot: 6,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 0.2,
          tools: UNCHECKED,
        }),
    });
    render(() => <ProjectsScreen />);
    await screen.findByText("other");
    fireEvent.keyDown(document, { key: "ArrowDown" });
    fireEvent.keyDown(document, { key: "ArrowDown" });
    fireEvent.keyDown(document, { key: "r" });
    await waitFor(() =>
      expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.rescan"))).toBe(
        true,
      ),
    );
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rescan"));
    expect(JSON.parse(call?.[1].body)).toEqual({ project: 2 });
  });

  it("leaves the keys alone while typing in a field", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.keyDown(field, { key: "r" });
    expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.rescan"))).toBe(
      false,
    );
  });

  it("leaves Enter to a focused button instead of opening the project", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    await screen.findByText("shop");
    fireEvent.keyDown(document, { key: "ArrowDown" });
    const button = screen.getByRole("button", { name: "Add and scan" });
    const allowed = fireEvent.keyDown(button, { key: "Enter", cancelable: true });
    expect(allowed).toBe(true);
  });

  it("speaks German when the language is German", async () => {
    setLanguage("de");
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    expect(await screen.findByText(/^1 Projekt · 433 Typen · Enter öffnet/)).toBeInTheDocument();
    expect(screen.getByText("aktuell")).toBeInTheDocument();
    expect(screen.getByText("41.208")).toBeInTheDocument();
  });
});
