import { fireEvent, render, screen, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { recordTime, resetTimings } from "../../shared/api/timings";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { refreshProjects } from "../projects/projects";
import { refreshStatus } from "../status/status";
import { listenToScans, resetRuns } from "./events";
import { ScanScreen } from "./ScanScreen";

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
    { language: "csharp", files: 3000, loc: 38_000, symbols: 10, tier: "full" },
    { language: "xaml", files: 12, loc: 3208, symbols: 10, tier: "full" },
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
  render(() => <ScanScreen />);
  return stop;
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

describe("ScanScreen", () => {
  it("shows the budgets with the measured start time, the languages and recent scans when idle", async () => {
    setLanguage("en");
    const stop = await shown();
    expect(await screen.findByText("No scan running")).toBeInTheDocument();
    expect(screen.getByText("Program start")).toBeInTheDocument();
    expect(screen.getAllByText("Target < 300 ms")).toHaveLength(2);
    expect(screen.getByText("180 ms")).toBeInTheDocument();
    expect(screen.getByText("C#")).toBeInTheDocument();
    expect(screen.getByText("38,000")).toBeInTheDocument();
    expect(screen.getByText("shop · full")).toBeInTheDocument();
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
              { language: "csharp", files: 3000, loc: 38_000, symbols: 9120, tier: "full" },
              { language: "go", files: 12, loc: 3208, symbols: 212, tier: "basic" },
              { language: "xaml", files: 4, loc: 80, symbols: 0, tier: null },
            ],
          },
        ]),
      "status.info": () => answer(STATUS),
    });
    const stop = await shown();
    expect(
      await screen.findByText("shop · main @ e07c4a1 · full (first scan)"),
    ).toBeInTheDocument();
    const table = screen.getByRole("table");
    expect(within(table).getByText("Tier")).toBeInTheDocument();
    expect(within(table).getByText("Symbols")).toBeInTheDocument();
    expect(within(table).getByText("full")).toBeInTheDocument();
    expect(within(table).getByText("basic")).toBeInTheDocument();
    expect(within(table).getByText("9,120")).toBeInTheDocument();
    stop();
  });

  it("shows the times this session measured and a dash for the rest", async () => {
    setLanguage("en");
    recordTime("search_keystroke", 4.2);
    recordTime("graph_layout", 61);
    const stop = await shown();
    await screen.findByText("No scan running");
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
    const row = screen.getByText("Scan 100k lines").parentElement as HTMLElement;
    const value = row.lastElementChild as HTMLElement;
    expect(value.className).not.toMatch(/good|bad/);
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
    expect(screen.getByText("62 %")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "62");
    expect(screen.getByText("2,364 of 3,812 files", { exact: false })).toBeInTheDocument();
    const pipeline = screen.getByText("Pipeline").parentElement as HTMLElement;
    expect(within(pipeline).getByText("Fetch source")).toBeInTheDocument();
    expect(
      within(pipeline).getByText(/Tree-sitter, in parallel · 2,364 \/ 3,812/),
    ).toBeInTheDocument();
    FakeSocket.last?.send({ type: "snapshot_created", project: 1, snapshot: 6 });
    expect(await screen.findByText("Scan finished")).toBeInTheDocument();
    expect(screen.getByText("100 %")).toBeInTheDocument();
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
    expect(screen.getByText("Leistungsziele")).toBeInTheDocument();
    expect(screen.getAllByText("Ziel < 300 ms")).toHaveLength(2);
    stop();
  });

  it("asks for a first project when there is none", async () => {
    setLanguage("en");
    serve({ "projects.list": () => answer([]), "status.info": () => answer(STATUS) });
    const stop = await shown();
    expect(await screen.findByText(/Add a project first/)).toBeInTheDocument();
    stop();
  });
});
