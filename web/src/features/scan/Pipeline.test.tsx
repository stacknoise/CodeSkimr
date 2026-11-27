import { render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
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
