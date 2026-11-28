import { render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, serve, useServerSession } from "../../tests/commands";
import { Pipeline } from "./Pipeline";
import { applyEvent } from "./run";
import type { Run, WireEvent } from "./types";

const STARTED = 1_000_000;

function writing(step: string, number: number): WireEvent {
  return {
    type: "scan_progressed",
    project: 1,
    stage: "persist",
    done: 0,
    total: 1,
    step,
    step_number: number,
    steps: 11,
  };
}

describe("Pipeline while the snapshot is written", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(STARTED);
    setLanguage("de");
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows the step with its place and a time that keeps running between events", () => {
    const run: Run[] = applyEvent([], writing("reference_indexes", 5), STARTED);
    render(() => <Pipeline run={run[0]} />);
    expect(screen.getByText("Verweis-Indizes anlegen · 5 / 11 · seit 0:00")).toBeTruthy();
    vi.advanceTimersByTime(252_000);
    expect(screen.getByText("Verweis-Indizes anlegen · 5 / 11 · seit 4:12")).toBeTruthy();
  });

  it("shows the usual detail of the stage when no step is reported", () => {
    const run: Run[] = applyEvent(
      [],
      { type: "scan_progressed", project: 1, stage: "persist", done: 0, total: 1 },
      STARTED,
    );
    render(() => <Pipeline run={run[0]} />);
    expect(screen.getByText("SQLite, Suchindex")).toBeTruthy();
  });
});

describe("Pipeline after the scan", () => {
  useServerSession();

  beforeEach(() => {
    setLanguage("en");
  });

  it("lists the time of each step of saving under the stages once the scan is over", async () => {
    serve({
      "scans.list": () =>
        answer({
          keep: 5,
          average_ms: null,
          scans: [
            {
              snapshot: 5,
              scan: 5,
              created: 1,
              commit: null,
              mode: "full",
              files: 1,
              symbols: 1,
              duration_ms: 900,
              files_changed: 1,
              steps: [
                { step: "references", duration_ms: 400 },
                { step: "reference_indexes", duration_ms: 1500 },
              ],
            },
          ],
        }),
    });
    const events: WireEvent[] = [
      { type: "scan_progressed", project: 1, stage: "persist", done: 1, total: 1 },
      { type: "snapshot_created", project: 1, snapshot: 5 },
    ];
    const run = events.reduce<Run[]>((runs, event) => applyEvent(runs, event, STARTED), []);
    render(() => <Pipeline run={run[0]} />);
    expect(await screen.findByText("Time per step of saving")).toBeTruthy();
    expect(screen.getByText("Build reference indexes")).toBeTruthy();
    expect(screen.getByText("1.9 s")).toBeTruthy();
  });

  it("offers no list for a scan that found nothing changed", () => {
    serve({ "scans.list": () => answer({ keep: 5, average_ms: null, scans: [] }) });
    const events: WireEvent[] = [
      { type: "scan_progressed", project: 1, stage: "persist", done: 1, total: 1 },
      { type: "snapshot_created", project: 1, snapshot: 5, unchanged: true },
    ];
    const run = events.reduce<Run[]>((runs, event) => applyEvent(runs, event, STARTED), []);
    render(() => <Pipeline run={run[0]} />);
    expect(screen.queryByText("Time per step of saving")).toBeNull();
  });
});
