import { fireEvent, render, screen, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { openProject } from "../../shared/project/active";
import { setLanguage } from "../../shared/i18n/i18n";
import { recordTime, resetTimings } from "../../shared/api/timings";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { refreshProjects } from "../projects/projects";
import { refreshStatus } from "../status/status";
import { listenToScans, resetRuns } from "./events";
import { ProjectsScreen } from "../projects/ProjectsScreen";
import { Performance } from "../status/Performance";
import { chooseProject, chooseTab, TAB_SCAN } from "../../tests/pageTabs";

const ROW: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: {
    snapshot: 5,
    created: 1_800_000_000,
    mode: "full",
    files: 3812,
    symbols: 9,
    references: 4,
  },
  types: 433,
  loc: 41_208,
  languages: [
    { language: "csharp", files: 3000, loc: 38_000, symbols: 10, parse_micros: null, tier: "full" },
    { language: "xaml", files: 12, loc: 3208, symbols: 10, parse_micros: null, tier: "full" },
  ],
  running: false,
};

const RUNNING = {
  type: "scan_progressed",
  project: 1,
  stage: "analyze",
  done: 10,
  total: 100,
};

function cancelCalls(): string[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith("projects.cancel"))
    .map(([, init]) => String((init as RequestInit).body));
}

const STATUS = {
  startup_ms: 180,
  memory_mb: 88,
  workers: 8,
  budgets: {
    cold_start: 300,
    search_keystroke: 10,
    explorer_switch: 16,
    usages_query: 20,
    graph_layout: 50,
    full_scan_100k_loc: 1000,
    incremental_rescan: 300,
  },
};

class FakeSocket {
  static last: FakeSocket | null = null;
  onmessage: ((message: { data: unknown }) => void) | null = null;
  onclose: (() => void) | null = null;
  constructor() {
    FakeSocket.last = this;
  }
  close() {}
  send(event: object) {
    this.onmessage?.({ data: JSON.stringify(event) });
  }
}

useServerSession();

beforeEach(() => {
  vi.stubGlobal("WebSocket", FakeSocket);
  resetRuns();
  openProject(ROW.id);
  serve({
    "projects.list": () => answer([ROW]),
    "status.info": () => answer(STATUS),
  });
});

afterEach(() => {
  resetRuns();
  resetTimings();
});

async function shown() {
  await refreshProjects();
  await refreshStatus();
  const stop = listenToScans();
  render(() => (
    <>
      <ProjectsScreen />
      <Performance />
    </>
  ));
  await chooseProject();
  await chooseTab(TAB_SCAN);
  return stop;
}

/** Opens the list of the performance targets at the status bar. */
function openPerformance(): void {
  fireEvent.click(screen.getByRole("button", { name: /^(Performance|Leistung)/ }));
}

/** No call went out for the Esc that closed a layer; the next Esc cancels the scan. */
async function cancelsOnlyOnSecondEsc(): Promise<void> {
  expect(cancelCalls()).toEqual([]);
  fireEvent.keyDown(document, { key: "Escape" });
  await vi.waitFor(() => expect(cancelCalls()).toHaveLength(1));
}

/** The screen with a server that also answers `projects.cancel`. */
async function shownWithCancel() {
  setLanguage("en");
  serve({
    "projects.list": () => answer([ROW]),
    "status.info": () => answer(STATUS),
    "projects.cancel": () => answer({ cancelled: true }),
  });
  return shown();
}

describe("the tab Scan", () => {
  it("shows the budgets with the measured start time, the languages and recent scans when idle", async () => {
    setLanguage("en");
    const stop = await shown();
    expect(await screen.findByText("No scan running")).toBeInTheDocument();
    openPerformance();
    expect(screen.getByText("Program start")).toBeInTheDocument();
    expect(screen.getAllByText("Target < 300 ms")).toHaveLength(2);
    expect(screen.getByText("180 ms")).toBeInTheDocument();
    expect(screen.getByText("C#")).toBeInTheDocument();
    expect(screen.getByText("38,000")).toBeInTheDocument();
    stop();
  });

  it("names the files the scan left out with their code, and how many more there were", async () => {
    setLanguage("en");
    serve({
      "projects.list": () =>
        answer([
          {
            ...ROW,
            left_out: [
              { path: "src/legacy.txt", code: "PRS-101", kept: false },
              { path: "src/big.cs", code: "PRS-501", kept: true },
              { path: "src/old.txt", code: "PRS-101", kept: false },
            ],
            left_out_more: 3,
          },
        ]),
      "status.info": () => answer(STATUS),
    });
    const stop = await shown();
    expect(await screen.findByText("Files left out")).toBeInTheDocument();
    expect(screen.getByText("src/legacy.txt")).toBeInTheDocument();
    expect(screen.getAllByText("PRS-101")).toHaveLength(2);
    expect(screen.getAllByText("missing in the result")).toHaveLength(2);
    expect(screen.getByText("PRS-501")).toBeInTheDocument();
    expect(screen.getByText("shown as the previous scan saw it")).toBeInTheDocument();
    expect(screen.getByText("and 3 more files")).toBeInTheDocument();
    expect(
      screen.getAllByText("A source file is not valid UTF-8 or UTF-16 and was skipped."),
    ).toHaveLength(1);
    stop();
  });

  it("says that the names come with the next scan if only the count is known", async () => {
    setLanguage("en");
    serve({
      "projects.list": () => answer([{ ...ROW, left_out: [], left_out_more: 2 }]),
      "status.info": () => answer(STATUS),
    });
    const stop = await shown();
    expect(
      await screen.findByText("2 files were left out. Their names show after the next scan."),
    ).toBeInTheDocument();
    expect(screen.queryByText("and 2 more files")).not.toBeInTheDocument();
    stop();
  });

  it("shows no list of left-out files if the scan left none out", async () => {
    setLanguage("en");
    const stop = await shown();
    expect(await screen.findByText("No scan running")).toBeInTheDocument();
    expect(screen.queryByText("Files left out")).not.toBeInTheDocument();
    stop();
  });

  it("lists tier, files, lines and symbols per language and the place of the scan", async () => {
    setLanguage("en");
    serve({
      "projects.list": () =>
        answer([
          {
            ...ROW,
            branch: "main",
            commit: "e07c4a1b9d",
            languages: [
              {
                language: "csharp",
                files: 3000,
                loc: 38_000,
                symbols: 9120,
                parse_micros: 620_000,
                tier: "full",
              },
              {
                language: "go",
                files: 12,
                loc: 3208,
                symbols: 212,
                parse_micros: null,
                tier: "basic",
              },
              { language: "xaml", files: 4, loc: 80, symbols: 0, parse_micros: null, tier: null },
            ],
          },
        ]),
      "status.info": () => answer(STATUS),
    });
    const stop = await shown();
    expect(await screen.findByText("shop · main @ e07c4a1 · full")).toBeInTheDocument();
    const table = screen.getByRole("table");
    expect(within(table).getByText("Tier")).toBeInTheDocument();
    expect(within(table).getByText("Symbols")).toBeInTheDocument();
    expect(within(table).getByText("full")).toBeInTheDocument();
    expect(within(table).getByText("basic")).toBeInTheDocument();
    expect(within(table).getByText("9,120")).toBeInTheDocument();
    expect(within(table).getByText("Parse time")).toBeInTheDocument();
    expect(within(table).getByText("0.62 s")).toBeInTheDocument();
    stop();
  });

  it("shows the times this session measured and a dash for the rest", async () => {
    setLanguage("en");
    recordTime("search_keystroke", 4.2);
    recordTime("graph_layout", 61);
    const stop = await shown();
    await screen.findByText("No scan running");
    openPerformance();
    expect(screen.getByText("4 ms")).toBeInTheDocument();
    expect(screen.getByText("61 ms")).toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(2);
    stop();
  });

  it("shows the cores in use while a scan runs and a dash otherwise", async () => {
    setLanguage("en");
    const stop = await shown();
    await screen.findByText("No scan running");
    FakeSocket.last?.send(RUNNING);
    const label = await screen.findByText("Cores active");
    expect(label.previousElementSibling?.textContent).toBe("8");
    stop();
  });

  it("shows no more cores than there are files to analyse", async () => {
    setLanguage("en");
    const stop = await shown();
    await screen.findByText("No scan running");
    FakeSocket.last?.send({ ...RUNNING, done: 1, total: 3 });
    const label = await screen.findByText("Cores active");
    expect(label.previousElementSibling?.textContent).toBe("3");
    stop();
  });

  it("leaves a running scan alone when Esc is pressed in a field or was already used", async () => {
    const stop = await shownWithCancel();
    FakeSocket.last?.send(RUNNING);
    await screen.findByRole("button", { name: /Cancel/ });
    const field = document.body.appendChild(document.createElement("input"));
    fireEvent.keyDown(field, { key: "Escape" });
    const used = new KeyboardEvent("keydown", { key: "Escape", cancelable: true });
    used.preventDefault();
    document.dispatchEvent(used);
    expect(cancelCalls()).toEqual([]);
    field.remove();
    stop();
  });

  it("lets Esc close the performance window first and leaves the running scan alone", async () => {
    const stop = await shownWithCancel();
    FakeSocket.last?.send(RUNNING);
    await screen.findByRole("button", { name: /Cancel/ });
    openPerformance();
    expect(screen.getByRole("button", { name: /^Performance/ })).toHaveAttribute(
      "aria-expanded",
      "true",
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.getByRole("button", { name: /^Performance/ })).toHaveAttribute(
      "aria-expanded",
      "false",
    );
    await cancelsOnlyOnSecondEsc();
    stop();
  });

  it("lets Esc close the form New project first and leaves the running scan alone", async () => {
    const stop = await shownWithCancel();
    FakeSocket.last?.send(RUNNING);
    await screen.findByRole("button", { name: /Cancel/ });
    fireEvent.click(screen.getByRole("button", { name: "New" }));
    expect(screen.getByLabelText("New project")).toBeInTheDocument();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByLabelText("New project")).toBeNull();
    await cancelsOnlyOnSecondEsc();
    stop();
  });

  it("cancels the running scan with the button and with Esc", async () => {
    const stop = await shownWithCancel();
    expect(screen.queryByRole("button", { name: /Cancel/ })).toBeNull();
    FakeSocket.last?.send(RUNNING);
    fireEvent.click(await screen.findByRole("button", { name: /Cancel/ }));
    fireEvent.keyDown(document, { key: "Escape" });
    await vi.waitFor(() => expect(cancelCalls()).toHaveLength(2));
    expect(cancelCalls()[0]).toBe('{"project":1}');
    FakeSocket.last?.send({ type: "scan_stopped", project: 1, reason: "cancelled", code: null });
    expect(await screen.findByText("Scan cancelled")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Cancel/ })).toBeNull();
    stop();
  });

  it("shows the full scan time of a small project without a good or bad colour", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({
      type: "scan_progressed",
      project: 1,
      stage: "analyze",
      done: 1,
      total: 2,
    });
    FakeSocket.last?.send({ type: "snapshot_created", project: 1, snapshot: 6 });
    await screen.findByText("Scan finished");
    openPerformance();
    const row = screen.getByText("Scan 100k lines").parentElement as HTMLElement;
    const value = row.lastElementChild as HTMLElement;
    expect(value.className).not.toMatch(/good|bad/);
    stop();
  });

  it("leaves the time of loading language packs out of the measured scan time", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({ ...RUNNING, stage: "fetch", done: 1, total: 1 });
    FakeSocket.last?.send({
      type: "snapshot_created",
      project: 1,
      snapshot: 6,
      pack_load_ms: 60_000,
    });
    await screen.findByText("Scan finished");
    openPerformance();
    const row = screen.getByText("Scan 100k lines").parentElement as HTMLElement;
    // The load took longer than the whole run in the browser, so nothing is left to measure.
    expect(within(row).getByText("0 ms")).toBeInTheDocument();
    stop();
  });

  it("follows a scan over the stream, stage by stage", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({
      type: "scan_progressed",
      project: 1,
      stage: "fetch",
      done: 1,
      total: 1,
    });
    FakeSocket.last?.send({
      type: "scan_progressed",
      project: 1,
      stage: "analyze",
      done: 2364,
      total: 3812,
    });
    expect(await screen.findByText("Scan running")).toBeInTheDocument();
    // Once in the card of the scan and once on the row of the stage.
    expect(screen.getAllByText("62 %")).toHaveLength(2);
    expect(screen.getByRole("progressbar", { name: "Progress of the scan" })).toHaveAttribute(
      "aria-valuenow",
      "62",
    );
    expect(screen.getByText("2,364 of 3,812 files", { exact: false })).toBeInTheDocument();
    const pipeline = screen.getByText("Pipeline").parentElement as HTMLElement;
    expect(within(pipeline).getByText("Fetch source")).toBeInTheDocument();
    expect(
      within(pipeline).getByText(/Tree-sitter, in parallel · 2,364 \/ 3,812/),
    ).toBeInTheDocument();
    FakeSocket.last?.send({ type: "snapshot_created", project: 1, snapshot: 6 });
    expect(await screen.findByText("Scan finished")).toBeInTheDocument();
    expect(screen.getAllByText("100 %")[0]).toBeInTheDocument();
    stop();
  });

  it("calls the scan of a project that was never scanned the first one", async () => {
    serve({
      "projects.list": () => answer([{ ...ROW, scan: null }]),
      "status.info": () => answer(STATUS),
    });
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send(RUNNING);
    await screen.findByText("Scan running");
    expect(document.body).toHaveTextContent("shop · full (first scan)");
    stop();
  });

  it("calls the running scan of a project that was scanned before incremental", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send(RUNNING);
    await screen.findByText("Scan running");
    expect(document.body).toHaveTextContent("shop · incremental");
    expect(screen.queryByText(/first scan/)).toBeNull();
    stop();
  });

  it("shows a share only for a stage that counts, and a moving bar for one that cannot", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({ ...RUNNING, stage: "fetch", done: 1, total: 1 });
    FakeSocket.last?.send({ ...RUNNING, stage: "analyze", done: 100, total: 100 });
    FakeSocket.last?.send({ ...RUNNING, stage: "resolve", done: 0, total: 5000 });
    const pipeline = (await screen.findByText("Pipeline")).parentElement as HTMLElement;
    expect(within(pipeline).getAllByText("100 %")).toHaveLength(2);
    const resolving = within(pipeline).getByRole("progressbar", { name: "Resolve dependencies" });
    expect(resolving).not.toHaveAttribute("aria-valuenow");
    expect(resolving).toHaveAccessibleDescription(/cannot be measured/);
    expect(within(pipeline).getByRole("progressbar", { name: "Parse" })).toHaveAttribute(
      "aria-valuenow",
      "100",
    );
    FakeSocket.last?.send({ ...RUNNING, stage: "resolve", done: 2500, total: 5000 });
    expect(await within(pipeline).findByText("50 %")).toBeInTheDocument();
    expect(resolving).toHaveAttribute("aria-valuenow", "50");
    expect(within(pipeline).queryByText(/cannot be measured/)).toBeNull();
    stop();
  });

  it("picks up a scan that was running before the page was loaded", async () => {
    setLanguage("en");
    serve({
      "projects.list": () => answer([{ ...ROW, running: true }]),
      "status.info": () => answer(STATUS),
    });
    const stop = await shown();
    expect(await screen.findByText("Scan starting")).toBeInTheDocument();
    expect(screen.getByText(/The progress shows up with the next report/)).toBeInTheDocument();
    // What the server replays when the stream opens: the stages that are over and the open one.
    FakeSocket.last?.send({ ...RUNNING, stage: "fetch", done: 1, total: 1, elapsed_ms: 900 });
    FakeSocket.last?.send({ ...RUNNING, stage: "analyze", done: 50, total: 100, elapsed_ms: 5000 });
    expect(await screen.findByText("Scan running")).toBeInTheDocument();
    expect(screen.getAllByText("50 %")).toHaveLength(2);
    stop();
  });

  it("shows the time a stage took as the server measured it", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({ ...RUNNING, stage: "fetch", done: 1, total: 1, elapsed_ms: 1200 });
    FakeSocket.last?.send({ ...RUNNING, stage: "analyze", elapsed_ms: 40 });
    const pipeline = (await screen.findByText("Pipeline")).parentElement as HTMLElement;
    expect(within(pipeline).getByText("1.2 s")).toBeInTheDocument();
    stop();
  });

  it("says so when a scan fails", async () => {
    setLanguage("en");
    const stop = await shown();
    FakeSocket.last?.send({
      type: "scan_progressed",
      project: 1,
      stage: "analyze",
      done: 1,
      total: 9,
    });
    FakeSocket.last?.send({ type: "scan_stopped", project: 1, reason: "failed", code: "PRS-101" });
    expect(await screen.findByText("Scan failed")).toBeInTheDocument();
    stop();
  });

  it("is written in German too", async () => {
    setLanguage("de");
    const stop = await shown();
    expect(await screen.findByText("Kein Scan läuft")).toBeInTheDocument();
    openPerformance();
    expect(screen.getByText("Leistungsziele")).toBeInTheDocument();
    expect(screen.getAllByText("Ziel < 300 ms")).toHaveLength(2);
    stop();
  });

  it("asks for a first project when there is none", async () => {
    setLanguage("en");
    serve({ "projects.list": () => answer([]), "status.info": () => answer(STATUS) });
    await refreshProjects();
    render(() => <ProjectsScreen />);
    expect(await screen.findByText("No projects yet")).toBeInTheDocument();
  });
});
