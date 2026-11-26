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
import { chooseProject, chooseTab, TAB_LEVEL2, TAB_SCAN } from "../../tests/pageTabs";
import { projectRow } from "../../tests/projectRow";
import type { LanguageInfo, ProjectRow } from "./types";

const REPORT: ScanReport = {
  last_scan: { at: 1_700_000_000, level: "2", origin: "browser" },
  level2: {
    at: 1_700_000_000,
    origin: "browser",
    languages: [
      { language: "csharp", outcome: "checked", files: 328, stale: 3, failed_workspaces: 2 },
      { language: "typescript", outcome: "skipped", reason: "tools_unavailable" },
      { language: "java", outcome: "failed", code: "RES-501" },
      { language: "rust", outcome: "skipped", reason: "no_project_file" },
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
  await chooseProject();
  await chooseTab(TAB_LEVEL2);
  return await screen.findByRole("region", { name: "Scan" });
}

/** The tab Scan of the project, with the scan buttons. */
async function scanTab(row: ProjectRow = ROW, rescan = () => answer(rescanned(row.id))) {
  await selected(row, rescan);
  await chooseTab(TAB_SCAN);
  return await screen.findByRole("tabpanel");
}

function rescanBody(): unknown {
  const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.rescan"));
  return JSON.parse(call?.[1].body);
}

describe("the report of the last scan", () => {
  it("says at which stage the last scan ran and who started it, and what stage 2 did", async () => {
    const panel = await selected();
    expect(
      within(panel).getByText("C# · 328 files · 3 outdated · 2 workspaces not run through"),
    ).toBeInTheDocument();
    expect(
      within(panel).getByText(/the indexer or a tool it needs is missing/),
    ).toBeInTheDocument();
    expect(
      within(panel).getByText(/The SCIP index is larger than the configured limit/),
    ).toBeInTheDocument();
    expect(within(panel).getByText("RES-501")).toBeInTheDocument();
    expect(within(panel).getByText(/Install or update the tool/)).toBeInTheDocument();
    expect(within(panel).getByRole("button", { name: "Go to Tools" })).toBeInTheDocument();
    await chooseTab(/Overview/);
    expect(screen.getByText(/Stage 2 \(browser\)/)).toBeInTheDocument();
    expect(screen.getByText(/Exact check \(stage 2\) failed for Java \(RES-501\)/)).toBeVisible();
  });

  it("names the missing project file and what to do", async () => {
    const panel = await selected();
    expect(within(panel).getByText(/no Cargo.toml found in the project folder/)).toBeVisible();
    expect(within(panel).getByText(/add that folder as a project of its own/)).toBeVisible();
  });

  it("groups failed, skipped and checked languages in this order", async () => {
    const panel = await selected();
    const groups = [...panel.querySelectorAll("[data-group]")].map((el) =>
      el.getAttribute("data-group"),
    );
    expect(groups).toEqual(["failed", "skipped", "checked"]);
  });

  it("says the same in German", async () => {
    setLanguage("de");
    const panel = await selected();
    expect(
      within(panel).getByText("C# · 328 Dateien · 3 veraltet · 2 Workspaces nicht durchgelaufen"),
    ).toBeVisible();
    expect(within(panel).getByText(/der Indexer oder ein Werkzeug dazu fehlt/)).toBeVisible();
    expect(within(panel).getByText(/Werkzeug installieren oder aktualisieren/)).toBeVisible();
  });

  it("says that no stage 2 run is stored when only stage 1 ran", async () => {
    const level1 = {
      ...REPORT,
      last_scan: { ...REPORT.last_scan, level: "1" as const },
      level2: null,
    };
    const panel = await selected({ ...ROW, report: level1 });
    expect(within(panel).getByText(/No stage 2 run is stored/)).toBeInTheDocument();
    await chooseTab(/Overview/);
    expect(screen.getByText(/Stage 1 \(browser\)/)).toBeInTheDocument();
  });
});

function info(language: string): LanguageInfo {
  return { language, files: 3, loc: 30, symbols: 5, parse_micros: null, tier: "full" };
}

describe("the languages of the report", () => {
  it("lists a language without an indexer in its own group, not in the report", async () => {
    const row = projectRow({
      report: REPORT,
      tools: { checked_at: 1_700_000_000, languages: [], tools: [] },
      languages: [info("csharp"), info("c"), info("ruby")],
    });
    const panel = await selected(row);
    const group = panel.querySelector("[data-group=no-level2]") as HTMLElement;
    expect(within(group).getByText("No stage 2 available")).toBeInTheDocument();
    expect(within(group).getByText("C")).toBeInTheDocument();
    expect(within(group).getByText("Ruby")).toBeInTheDocument();
    expect(within(group).queryByText("C#")).not.toBeInTheDocument();
  });

  it("does not call a language without stage 2 when its indexer only came after the run", async () => {
    const tools = {
      checked_at: 1_700_000_000,
      languages: [{ language: "java", level2_ready: false }],
      tools: [],
    };
    const row = projectRow({
      report: REPORT,
      tools,
      languages: [info("csharp"), info("java"), info("c")],
    });
    const panel = await selected({
      ...row,
      report: {
        ...REPORT,
        level2: {
          ...REPORT.level2!,
          languages: REPORT.level2!.languages.filter((line) => line.language !== "java"),
        },
      },
    });
    const group = panel.querySelector("[data-group=no-level2]") as HTMLElement;
    expect(within(group).getByText("C")).toBeInTheDocument();
    expect(within(group).queryByText("Java")).not.toBeInTheDocument();
  });

  it("says nothing about languages without stage 2 until the check of the tools is loaded", async () => {
    const row = projectRow({ report: REPORT, tools: null, languages: [info("csharp"), info("c")] });
    const panel = await selected(row);
    expect(panel.querySelector("[data-group=no-level2]")).toBeNull();
  });

  it("says once that stage 2 is off instead of one row with a button for each language", async () => {
    const off = {
      ...REPORT,
      level2: {
        ...REPORT.level2!,
        languages: [
          { language: "python", outcome: "skipped" as const, reason: "not_confirmed" },
          { language: "rust", outcome: "skipped" as const, reason: "not_confirmed" },
        ],
      },
    };
    const panel = await selected({ ...ROW, report: off });
    const hint = panel.querySelector("[data-group=switched-off]") as HTMLElement;
    expect(hint).toHaveTextContent("Stage 2 is off or not confirmed");
    expect(hint).toHaveTextContent("2 languages: Python, Rust");
    expect(panel.querySelector("[data-group=skipped]")).toBeNull();
    expect(within(panel).queryByRole("button", { name: "Check again" })).not.toBeInTheDocument();
  });
});

describe("the button of a row of the report", () => {
  it("checks only the language of its row again", async () => {
    const panel = await selected();
    const failed = panel.querySelector("[data-group=failed]") as HTMLElement;
    fireEvent.click(within(failed).getByRole("button", { name: "Check again" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4, language: "java" });
  });
});

describe("the scan buttons", () => {
  it("scans with stage 2 by default and says that build tools run", async () => {
    const panel = await scanTab();
    expect(within(panel).getByText(/runs the build tools of the project/)).toBeVisible();
    fireEvent.click(within(panel).getByRole("button", { name: "Scan again" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4 });
  });

  it("scans stage 1 only on request", async () => {
    const panel = await scanTab();
    fireEvent.click(within(panel).getByRole("button", { name: "Stage 1 only" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4, level2: false });
  });

  it("scans every file again on request", async () => {
    const panel = await scanTab();
    fireEvent.click(within(panel).getByRole("button", { name: "Full scan" }));
    await rescanAsked();
    expect(rescanBody()).toEqual({ project: 4, full: true });
  });

  it.each([
    ["not_confirmed", /not confirmed for this project/],
    ["busy", /another stage 2 run is going on/],
    ["paused", /there is a pause/],
    ["not_permitted", /may not run indexers here/],
  ])("explains why stage 2 did not start (%s)", async (reason, text) => {
    const panel = await scanTab(ROW, () =>
      answer({ ...rescanned(4), level2: { state: "not_started", reason } }),
    );
    fireEvent.click(within(panel).getByRole("button", { name: "Scan again" }));
    expect(await within(panel).findByText(text)).toBeVisible();
    expect(within(panel).getByText(/Stage 1 is stored/)).toBeVisible();
  });
});

describe("stage 2 running", () => {
  it("shows the language, disables the scan buttons and cancels", async () => {
    const buttons = await scanTab({ ...ROW, running: true });
    expect(within(buttons).getByRole("button", { name: "Scan again" })).toBeDisabled();
    await chooseTab(TAB_LEVEL2);
    const panel = await screen.findByRole("region", { name: "Scan" });
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
    expect(within(panel).queryByText(/importing results/)).toBeNull();
    await chooseTab(TAB_SCAN);
    expect(await screen.findByText("Stage 2 ended: 1 checked, 1 skipped, 0 failed.")).toBeVisible();
  });

  it("drops a banner when the list no longer calls the project running", async () => {
    const panel = await selected();
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    expect(await within(panel).findByText("Stage 2 running (C#)")).toBeVisible();
    await refreshProjects();
    await waitFor(() => expect(within(panel).queryByText(/Stage 2 running/)).toBeNull());
  });
});

describe("focus and announcements", () => {
  const switchOf = () => screen.getByRole("switch", { name: /(Level 2|Stufe 2) (for|für) shop/ });

  it("keeps the scan buttons in place, so the focus stays on the one that was pressed", async () => {
    const panel = await scanTab();
    const button = within(panel).getByRole("button", { name: "Scan again" });
    button.focus();
    fireEvent.click(button);
    expect(button).toBeInTheDocument();
    expect(button).toHaveFocus();
  });

  it("leaves the focus alone where a person is typing elsewhere", async () => {
    const panel = await scanTab();
    fireEvent.click(screen.getByRole("button", { name: "New" }));
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    field.focus();
    fireEvent.click(within(panel).getByRole("button", { name: "Scan again" }));
    expect(field).toHaveFocus();
  });

  it("puts the focus on the switch when the run ends under the cancel button", async () => {
    const panel = await selected({ ...ROW, running: true });
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    const cancel = await within(panel).findByRole("button", { name: "Cancel stage 2" });
    cancel.focus();
    noteLevel2Finished(4, { checked: 1, skipped: 0, failed: 0 });
    await waitFor(() => expect(switchOf()).toHaveFocus());
  });

  it("does not take the focus from another place when the run ends", async () => {
    const panel = await selected({ ...ROW, running: true });
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    await within(panel).findByRole("button", { name: "Cancel stage 2" });
    fireEvent.click(screen.getByRole("button", { name: "New" }));
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    field.focus();
    noteLevel2Finished(4, { checked: 1, skipped: 0, failed: 0 });
    await waitFor(() =>
      expect(within(panel).queryByRole("button", { name: "Cancel stage 2" })).toBeNull(),
    );
    expect(field).toHaveFocus();
  });

  it("announces the start and the phase of stage 2 in a region that was there before", async () => {
    const panel = await selected({ ...ROW, running: true });
    const before = [...panel.querySelectorAll("[aria-live=polite]")];
    noteLevel2Progress(4, { language: "csharp", stage: "level2_running" });
    const title = await within(panel).findByText("Stage 2 running (C#)");
    expect(before).toContain(title.closest("[aria-live=polite]"));
    noteLevel2Progress(4, { language: "csharp", stage: "level2_importing" });
    const importing = await within(panel).findByText("Stage 2 importing results (C#)");
    expect(before).toContain(importing.closest("[aria-live=polite]"));
  });
});
