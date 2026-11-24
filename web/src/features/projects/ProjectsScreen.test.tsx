import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { setLanguage } from "../../shared/i18n/i18n";
import { activeProject, openProject } from "../../shared/project/active";
import { afterEach, describe, expect, it, vi } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { currentPath } from "../../app/router";
import { ProjectsScreen } from "./ProjectsScreen";
import { addProject, cancelRename, noteScanReport, refreshProjects } from "./projects";
import { UNCHECKED, rescanAsked, rescanned } from "../../tests/scanned";
import type { ProjectRow } from "./types";

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
    { language: "csharp", files: 3, loc: 38_000, symbols: 10, parse_micros: null, tier: "full" },
    { language: "xaml", files: 1, loc: 3_208, symbols: 10, parse_micros: null, tier: "full" },
  ],
  running: false,
};

useServerSession();
afterEach(cancelRename);

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

  it("fills the display name from the source and sends it only when the user typed one", async () => {
    serve({
      "projects.list": () => answer([]),
      "projects.suggest_name": () => answer({ name: "shop/src" }),
      "projects.add": () =>
        answer({
          project: 1,
          snapshot: 5,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 1,
          tools: UNCHECKED,
        }),
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "/work/shop/src" } });
    const name = await screen.findByLabelText("Display name");
    await waitFor(() => expect((name as HTMLInputElement).value).toBe("shop/src"));
    fireEvent.click(screen.getByRole("button", { name: "Add and scan" }));
    await waitFor(() => expect((field as HTMLInputElement).value).toBe(""));
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.add"));
    expect(JSON.parse(call?.[1].body)).toEqual({ source: "/work/shop/src", branch: "main" });
  });

  it("asks for no name while a network path is typed", async () => {
    serve({
      "projects.list": () => answer([]),
      "projects.suggest_name": () => answer({ name: "x" }),
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "\\\\server\\share\\src" } });
    await new Promise((resolve) => setTimeout(resolve, 450));
    expect(
      fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.suggest_name")),
    ).toBe(false);
  });

  it("renames a project, and keeps the typed name when the list is read again", async () => {
    let name = "shop";
    serve({
      "projects.list": () => answer([{ ...ROW, name }]),
      "projects.rename": () => {
        name = "Webshop";
        return answer({ project: 1, name });
      },
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Rename shop" }));
    const input = screen.getByLabelText("New display name for shop");
    fireEvent.input(input, { target: { value: "Webshop" } });
    await refreshProjects();
    expect((screen.getByLabelText("New display name for shop") as HTMLInputElement).value).toBe(
      "Webshop",
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Webshop")).toBeInTheDocument();
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rename"));
    expect(JSON.parse(call?.[1].body)).toEqual({ project: 1, name: "Webshop" });
  });

  it("shows why a name was refused and keeps the edit open", async () => {
    serve({
      "projects.list": () => answer([ROW]),
      "projects.rename": () => answer({ code: "PRJ-102" }, 400),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Rename shop" }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(screen.getByLabelText("New display name for shop")).toBeInTheDocument();
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
      "projects.rescan": () => answer(rescanned(2)),
    });
    render(() => <ProjectsScreen />);
    await screen.findByText("other");
    fireEvent.keyDown(document, { key: "ArrowDown" });
    fireEvent.keyDown(document, { key: "ArrowDown" });
    fireEvent.keyDown(document, { key: "r" });
    await rescanAsked();
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rescan"));
    expect(JSON.parse(call?.[1].body)).toEqual({ project: 2 });
  });

  it("shows the scan screen of the project when a scan is started", async () => {
    serve({
      "projects.list": () => answer([ROW]),
      "projects.rescan": () => answer(rescanned(1)),
    });
    render(() => <ProjectsScreen />);
    await screen.findByText("shop");
    fireEvent.keyDown(document, { key: "r" });
    await rescanAsked();
    expect(currentPath()).toBe("/scan");
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rescan"));
    expect(activeProject()).toBe(JSON.parse(call?.[1].body).project);
  });

  it("ignores the report of a project that was known before when a new one is added", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    await screen.findByText("shop");
    const before = currentPath();
    const adding = addProject("/work/new", "main");
    noteScanReport(ROW.id);
    expect(currentPath()).toBe(before);
    await adding;
  });

  it("shows the scan screen of a new project with its first report", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    await screen.findByText("shop");
    const adding = addProject("/work/new", "main");
    noteScanReport(7);
    expect(currentPath()).toBe("/scan");
    expect(activeProject()).toBe(7);
    await adding;
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

  it("asks before it removes a project and leaves it alone when the user cancels", async () => {
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove project shop" }));
    expect(screen.getByText(/Scans, results and notes of the project are deleted/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("button", { name: "Remove project shop" })).toBeInTheDocument();
    expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.remove"))).toBe(
      false,
    );
  });

  it("removes a project after the confirmation, reads the list again and closes it", async () => {
    let removed = false;
    serve({
      "projects.list": () => answer(removed ? [{ ...ROW, id: 2, name: "other" }] : [ROW]),
      "projects.remove": () => {
        removed = true;
        return answer({ project: 1 });
      },
    });
    openProject(1);
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove project shop" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove for good" }));
    await waitFor(() => expect(screen.queryByText("shop")).not.toBeInTheDocument());
    expect(screen.getByText("other")).toBeInTheDocument();
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.remove"));
    expect(JSON.parse(call?.[1].body)).toEqual({ project: 1 });
    expect(activeProject()).toBeNull();
  });

  it("keeps the active project when another one is removed", async () => {
    serve({
      "projects.list": () => answer([ROW, { ...ROW, id: 2, name: "other" }]),
      "projects.remove": () => answer({ project: 2 }),
    });
    openProject(1);
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove project other" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove for good" }));
    await waitFor(() =>
      expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.remove"))).toBe(
        true,
      ),
    );
    expect(activeProject()).toBe(1);
  });

  it("shows why a removal failed and keeps the project", async () => {
    serve({
      "projects.list": () => answer([ROW]),
      "projects.remove": () => answer({ code: "PRJ-302" }, 409),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove project shop" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove for good" }));
    expect(await screen.findByText("A scan for this project is already running.")).toBeVisible();
    expect(screen.getByText("shop")).toBeInTheDocument();
  });

  it("offers no removal while a scan of the project runs", async () => {
    serve({ "projects.list": () => answer([{ ...ROW, running: true }]) });
    render(() => <ProjectsScreen />);
    await screen.findByText("scan running");
    expect(screen.queryByRole("button", { name: "Remove project shop" })).not.toBeInTheDocument();
  });

  it("speaks German when the removal is asked for", async () => {
    setLanguage("de");
    serve({ "projects.list": () => answer([ROW]) });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Projekt shop entfernen" }));
    expect(screen.getByRole("button", { name: "Endgültig entfernen" })).toBeInTheDocument();
  });
});
