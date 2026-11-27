import { describe, expect, it } from "vitest";
import {
  applyEvent,
  checkedOnly,
  files,
  filesPerSecond,
  activeStep,
  latestRun,
  minutesSeconds,
  percent,
  secondsLeft,
  isUnmeasured,
  stagePercent,
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

describe("checkedOnly", () => {
  it("tells how many files a scan checked when the server says that none changed", () => {
    const [run] = fold([
      [progress("discover", 40, 40), 1000],
      [progress("analyze", 0, 0), 1100],
      [{ ...created, unchanged: true }, 1200],
    ]);
    expect(checkedOnly(run!)).toBe(40);
  });

  it("is null while the scan runs and when a snapshot was written without parsing", () => {
    const [running] = fold([[progress("discover", 40, 40), 1000]]);
    expect(checkedOnly(running!)).toBeNull();
    // Only deleted files: nothing to parse, but the snapshot is a new one.
    const [deleted] = fold([
      [progress("discover", 39, 39), 1000],
      [progress("analyze", 0, 0), 1100],
      [created, 1200],
    ]);
    expect(checkedOnly(deleted!)).toBeNull();
  });
});

describe("applyEvent", () => {
  it("keeps the time of loading language packs from the end event", () => {
    const [run] = fold([
      [progress("fetch", 1, 1), 1000],
      [{ type: "snapshot_created", project: 1, snapshot: 5, pack_load_ms: 700 }, 2500],
    ]);
    expect(run?.packLoadMs).toBe(700);
  });

  it("counts no pack load time when the server tells none", () => {
    const [run] = fold([
      [progress("fetch", 1, 1), 1000],
      [created, 2000],
    ]);
    expect(run?.packLoadMs).toBe(0);
  });

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
  });

  it("ignores an end without a run and file changes", () => {
    const runs = fold([
      [created, 0],
      [{ type: "files_changed", project: 1, files: 3, rescan: false }, 1],
    ]);
    expect(runs).toEqual([]);
  });
});

describe("the share of one stage", () => {
  const at = (events: [WireEvent, number][]) => fold(events)[0] as Run;

  it("is the share of its units while they count, 100 when it is over, none before", () => {
    const run = at([
      [progress("fetch", 1, 1), 0],
      [progress("analyze", 25, 100), 10],
    ]);
    expect(stagePercent(run, "fetch")).toBe(100);
    expect(stagePercent(run, "analyze")).toBe(25);
    expect(stagePercent(run, "resolve")).toBeNull();
  });

  it("is unknown for a stage that runs without a count that says anything", () => {
    const run = at([[progress("resolve", 0, 4000), 0]]);
    expect(stagePercent(run, "resolve")).toBeNull();
    expect(isUnmeasured(run, "resolve")).toBe(true);
    expect(isUnmeasured(run, "analyze")).toBe(false);
  });

  it("puts the start of a run back by the time its stage had run when the page came late", () => {
    const run = at([[progress("analyze", 50, 100, 1, 5000), 20_000]]);
    expect(run.startedAt).toBe(15_000);
    expect(run.stages.analyze?.startedAt).toBe(15_000);
    expect(filesPerSecond(run, 20_000)).toBe(10);
  });
});

describe("the percent of a scan", () => {
  it("stays under 100 while later stages still run, and says 100 once it is done", () => {
    const events: [WireEvent, number][] = [
      [progress("analyze", 10, 10), 0],
      [progress("resolve", 0, 40), 5],
    ];
    expect(percent(fold(events)[0] as Run)).toBe(99);
    expect(percent(fold([...events, [created, 9]])[0] as Run)).toBe(100);
  });
});

describe("a report that arrives late", () => {
  it("does not move the count of a stage back", () => {
    const run = fold([
      [progress("analyze", 6, 10), 0],
      [progress("analyze", 5, 10), 1],
    ])[0] as Run;
    expect(run.stages.analyze?.done).toBe(6);
  });

  it("lets a stage with a new total start again", () => {
    const run = fold([
      [progress("resolve", 0, 0), 0],
      [progress("resolve", 3, 9), 1],
    ])[0] as Run;
    expect(run.stages.resolve?.done).toBe(3);
  });
});

describe("the steps of the write", () => {
  const step = (name: string, number: number, project = 1): WireEvent => ({
    type: "scan_progressed",
    project,
    stage: "persist",
    done: 0,
    total: 1,
    step: name,
    step_number: number,
    steps: 11,
  });

  it("names the step the write is in and counts its time from the first report of the step", () => {
    const [run] = fold([
      [step("reference_indexes", 5), 10_000],
      [step("reference_indexes", 5), 70_000],
    ]);
    const active = activeStep(run as Run, 254_000);
    expect(active?.step).toMatchObject({ name: "reference_indexes", number: 5, of: 11 });
    expect(active?.seconds).toBe(244);
  });

  it("starts the time again with the next step", () => {
    const [run] = fold([
      [step("references", 4), 10_000],
      [step("reference_indexes", 5), 50_000],
    ]);
    expect(activeStep(run as Run, 60_000)?.seconds).toBe(10);
  });

  it("has no step before the write reports one, and none once the scan is over", () => {
    const [before] = fold([[progress("fetch", 1, 1), 1000]]);
    expect(activeStep(before as Run, 2000)).toBeNull();
    const [over] = fold([
      [step("commit", 11), 1000],
      [created, 2000],
    ]);
    expect(activeStep(over as Run, 3000)).toBeNull();
  });
});

describe("minutesSeconds", () => {
  it("writes minutes and two-digit seconds", () => {
    expect(minutesSeconds(0)).toBe("0:00");
    expect(minutesSeconds(252.7)).toBe("4:12");
    expect(minutesSeconds(3605)).toBe("60:05");
  });
});
