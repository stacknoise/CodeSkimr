import { describe, expect, it } from "vitest";
import {
  applyEvent,
  files,
  filesPerSecond,
  focusProject,
  latestRun,
  percent,
  secondsLeft,
  stageSeconds,
  stageState,
} from "./run";
import type { Run, WireEvent } from "./types";

const progress = (
  stage: "fetch" | "discover" | "analyze" | "resolve",
  done: number,
  total: number,
  project = 1,
  elapsed_ms?: number,
): WireEvent => ({
  type: "scan_progressed",
  project,
  stage,
  done,
  total,
  ...(elapsed_ms === undefined ? {} : { elapsed_ms }),
});
const created: WireEvent = { type: "snapshot_created", project: 1, snapshot: 5 };

function fold(events: [WireEvent, number][]): Run[] {
  return events.reduce<Run[]>((runs, [event, now]) => applyEvent(runs, event, now), []);
}

describe("applyEvent", () => {
  it("starts a run on the first report and ends the earlier stages when a later one starts", () => {
    const [run] = fold([
      [progress("fetch", 1, 1), 1000],
      [progress("discover", 0, 0), 1380],
      [progress("analyze", 10, 100), 1400],
    ]);
    expect(run?.state).toBe("running");
    expect(stageState(run!, "fetch")).toBe("done");
    expect(stageState(run!, "analyze")).toBe("running");
    expect(stageState(run!, "resolve")).toBe("waiting");
    expect(stageSeconds(run!, "fetch", 9999)).toBeCloseTo(0.38);
  });

  it("takes the time of a finished stage from the server, not from the arrival of the events", () => {
    const [run] = fold([
      [progress("fetch", 1, 1, 1, 0), 1000],
      [progress("fetch", 1, 1, 1, 900), 1010],
      [progress("discover", 0, 0), 1020],
    ]);
    expect(stageSeconds(run!, "fetch", 9999)).toBeCloseTo(0.9);
  });

  it("computes percent, rate and remaining time from the file stage", () => {
    const [run] = fold([
      [progress("analyze", 0, 100), 0],
      [progress("analyze", 25, 100), 1000],
    ]);
    expect(files(run!)).toEqual({ done: 25, total: 100 });
    expect(percent(run!)).toBe(25);
    expect(filesPerSecond(run!, 1000)).toBe(25);
    expect(secondsLeft(run!, 1000)).toBe(3);
  });

  it("gives no estimate for the first few files and no percent before the file stage", () => {
    const [run] = fold([[progress("fetch", 0, 0), 0]]);
    expect(percent(run!)).toBe(0);
    expect(files(run!)).toBeNull();
    const [few] = fold([[progress("analyze", 2, 100), 1000]]);
    expect(secondsLeft(few!, 2000)).toBeNull();
  });

  it("ends a run when the snapshot is written", () => {
    const [run] = fold([
      [progress("analyze", 5, 5), 0],
      [progress("resolve", 0, 0), 500],
      [created, 900],
    ]);
    expect(run?.state).toBe("done");
    expect(percent(run!)).toBe(100);
    expect(stageState(run!, "persist")).toBe("done");
  });

  it("marks the open stage as stopped when a scan fails and keeps the code", () => {
    const [run] = fold([
      [progress("analyze", 5, 50), 0],
      [{ type: "scan_stopped", project: 1, reason: "failed", code: "PRS-101" }, 400],
    ]);
    expect(run?.state).toBe("failed");
    expect(run?.code).toBe("PRS-101");
    expect(stageState(run!, "analyze")).toBe("stopped");
    expect(stageSeconds(run!, "analyze", 99_999)).toBeCloseTo(0.4);
  });

  it("starts a new run after one ended, and keeps projects apart", () => {
    const runs = fold([
      [progress("analyze", 1, 2), 0],
      [created, 10],
      [progress("analyze", 1, 2, 2), 20],
      [progress("analyze", 1, 3), 30],
    ]);
    expect(runs.map((run) => [run.project, run.state])).toEqual([
      [1, "done"],
      [2, "running"],
      [1, "running"],
    ]);
    expect(latestRun(runs, 1)?.state).toBe("running");
    expect(latestRun(runs, null)).toBeUndefined();
    expect(focusProject(runs)).toBe(1);
  });

  it("ignores an end without a run and file changes", () => {
    const runs = fold([
      [created, 0],
      [{ type: "files_changed", project: 1, files: 3, rescan: false }, 1],
    ]);
    expect(runs).toEqual([]);
  });
});
