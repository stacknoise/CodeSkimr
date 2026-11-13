import { render, screen, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { recordTime, resetTimings } from "../../shared/api/timings";
import { answer, serve, useServerSession } from "../../tests/commands";
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
    { language: "csharp", files: 3000, loc: 38_000 },
    { language: "xaml", files: 12, loc: 3208 },
  ],
  running: false,
};

const STATUS = {
  startup_ms: 180,
  memory_mb: 88,
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
