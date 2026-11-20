import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import type { ScanReport } from "../../shared/project/report";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { rescanAsked, rescanned } from "../../tests/scanned";
import { resetLevel2 } from "./level2";
import { noteLevel2Finished, noteLevel2Progress, resetLevel2Runs } from "./level2run";
import { refreshProjects } from "./projects";
import { ProjectsScreen } from "./ProjectsScreen";
import { projectRow } from "../../tests/projectRow";
import type { ProjectRow } from "./types";

const REPORT: ScanReport = {
  last_scan: { at: 1_700_000_000, level: "2", origin: "browser" },
  level2: {
    at: 1_700_000_000,
    origin: "browser",
    languages: [
      { language: "csharp", outcome: "checked", files: 328, stale: 3 },
      { language: "typescript", outcome: "skipped", reason: "tools_unavailable" },
      { language: "java", outcome: "failed", code: "RES-501" },
    ],
  },
};

const ROW: ProjectRow = projectRow({ report: REPORT });

const STATUS = { state: "on", confirmed: true, launch: "window" };

useServerSession();
afterEach(() => {
  resetLevel2();
  resetLevel2Runs();
});

async function selected(row: ProjectRow = ROW, rescan = () => answer(rescanned(row.id))) {
  serve({
    "projects.list": () => answer([row]),
    "projects.rescan": rescan,
    "projects.cancel": () => answer({ cancelled: true }),
    "level2.status": () => answer(STATUS),
  });
  // The list is a module signal: a test must not start from the rows of the one before.
  await refreshProjects();
  render(() => <ProjectsScreen />);
  fireEvent.click(await screen.findByText("/work/shop"));
  return await screen.findByRole("region", { name: "Scan" });
}

function rescanBody(): unknown {
  const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rescan"));
  return JSON.parse(call?.[1].body);
}

describe("the report of the last scan", () => {
  it("says at which stage the last scan ran, who started it and what stage 2 did", async () => {
    const panel = await selected();
    const row = screen.getByRole("row", { name: /shop/ });
    expect(within(row).getByText("Stage 2 (browser)")).toBeInTheDocument();
    expect(within(row).getByText("1 checked, 1 skipped, 1 failed")).toBeInTheDocument();
    expect(within(row).getByText("3 files changed since")).toBeInTheDocument();
    expect(within(row).getByText("Stage 2 failed (RES-501)")).toBeInTheDocument();
    expect(within(panel).getByText("C#: checked, 328 files, 3 changed since")).toBeInTheDocument();
    expect(within(panel).getByText(/TypeScript: skipped, a tool is missing/)).toBeInTheDocument();
    expect(within(panel).getByText("Java: failed (RES-501)")).toBeInTheDocument();
    expect(within(panel).getByText(/Results of an earlier run may remain/)).toBeInTheDocument();
  });

  it("says the same in German", async () => {
    setLanguage("de");
    const panel = await selected();
    expect(within(panel).getByText(/C#: geprüft, 328 Dateien, 3 seitdem geändert/)).toBeVisible();
    expect(within(panel).getByText(/TypeScript: übersprungen, ein Werkzeug fehlt/)).toBeVisible();
  });

  it("says that no stage 2 run is stored when only stage 1 ran", async () => {
    const level1 = {
      ...REPORT,
      last_scan: { ...REPORT.last_scan, level: "1" as const },
      level2: null,
    };
    const panel = await selected({ ...ROW, report: level1 });
    expect(screen.getByText("Stage 1 (browser)")).toBeInTheDocument();
    expect(within(panel).getByText(/No stage 2 run is stored/)).toBeInTheDocument();
  });
});

describe("the scan buttons", () => {
  it("scans with stage 2 by default and says that build tools run", async () => {
    const panel = await selected();
    expect(within(panel).getByText(/runs the build tools of the project/)).toBeVisible();
    fireEvent.click(within(panel).getByRole("button", { name: "Scan again" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4 });
  });

  it("scans stage 1 only on request", async () => {
    const panel = await selected();
    fireEvent.click(within(panel).getByRole("button", { name: "Stage 1 only" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4, level2: false });
  });

  it.each([
    ["not_confirmed", /not confirmed for this project/],
    ["busy", /another stage 2 run is going on/],
    ["paused", /there is a pause/],
    ["not_permitted", /may not run indexers here/],
  ])("explains why stage 2 did not start (%s)", async (reason, text) => {
    const panel = await selected(ROW, () =>
      answer({ ...rescanned(4), level2: { state: "not_started", reason } }),
    );
    fireEvent.click(within(panel).getByRole("button", { name: "Scan again" }));
    expect(await within(panel).findByText(text)).toBeVisible();
    expect(within(panel).getByText(/Stage 1 is stored/)).toBeVisible();
  });
});

describe("stage 2 running", () => {
  it("shows the language, hides the scan buttons and cancels", async () => {
    const panel = await selected({ ...ROW, running: true });
    expect(within(panel).queryByRole("button", { name: "Scan again" })).toBeNull();
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    expect(await within(panel).findByText("Stage 2 running (C#)")).toBeVisible();
    expect(within(panel).getByText(/Cancelling keeps the result of stage 1/)).toBeVisible();
    fireEvent.click(within(panel).getByRole("button", { name: "Cancel stage 2" }));
    await waitFor(() =>
      expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.cancel"))).toBe(
        true,
      ),
    );
  });

  it("shows the importing stage and the end of the run", async () => {
    const panel = await selected({ ...ROW, running: true });
    noteLevel2Progress(4, { language: "csharp", stage: "level2_importing" });
    expect(await within(panel).findByText("Stage 2 importing results (C#)")).toBeVisible();
    noteLevel2Finished(4, { checked: 1, skipped: 1, failed: 0 });
    expect(
      await within(panel).findByText("Stage 2 ended: 1 checked, 1 skipped, 0 failed."),
    ).toBeVisible();
    expect(within(panel).queryByText(/importing results/)).toBeNull();
  });

  it("drops a banner when the list no longer calls the project running", async () => {
    const panel = await selected();
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    expect(await within(panel).findByText("Stage 2 running (C#)")).toBeVisible();
    await refreshProjects();
    await waitFor(() => expect(within(panel).queryByText(/Stage 2 running/)).toBeNull());
  });
});
