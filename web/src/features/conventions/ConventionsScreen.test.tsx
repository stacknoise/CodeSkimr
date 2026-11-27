import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

const rows = vi.hoisted(() => vi.fn());
const config = vi.hoisted(() => vi.fn());
const open = vi.hoisted(() => vi.fn());

vi.mock("../projects/projects", () => ({ projects: rows }));
vi.mock("../../shared/settings/ui-config", () => ({ uiConfig: config }));
vi.mock("../../shared/editor/editor", async (original) => ({
  ...(await original<object>()),
  openLink: open,
}));

import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, useServerSession } from "../../tests/commands";
import { ConventionsScreen } from "./ConventionsScreen";
import { resetConventions } from "./conventions";
import type { ConventionsOut, Finding } from "./types";

useServerSession();

const SETTINGS = {
  editor: { url_template: "vscode://file/{path}:{line}", allowed_schemes: ["vscode"] },
  default_theme: "petrol",
};

function finding(over: Partial<Finding>): Finding {
  return {
    code: "CHK-104",
    description: "",
    path: "src/a.rs",
    symbol: "a::run",
    line: 10,
    end_line: 70,
    lines: 52,
    max_lines: 40,
    value: null,
    max_value: null,
    literals: null,
    duplicate_of: null,
    similarity_percent: null,
    ...over,
  };
}

const ANSWER: ConventionsOut = {
  files_checked: 212,
  total: 3,
  findings: [
    finding({}),
    finding({
      code: "CHK-102",
      path: "src/web/format.ts",
      symbol: "formatBytes",
      line: 40,
      lines: null,
      max_lines: null,
      similarity_percent: 91,
      duplicate_of: { path: "src/shared/size.ts", symbol: "humanSize", line: 12, end_line: 30 },
    }),
    finding({
      code: "CHK-105",
      path: "src/rules/judge.rs",
      symbol: "judge_layer",
      line: 203,
      lines: null,
      max_lines: null,
      value: 19,
      max_value: 15,
    }),
  ],
  unchecked: [{ path: "docs/old.vb", code: "PRS-501" }],
  unmatched: [],
  not_checked: ["CHK-101"],
};

function serveCheck(body: unknown = ANSWER, status = 200) {
  fetchMock.mockImplementation(() => Promise.resolve(answer(body, status)));
}

async function check() {
  fireEvent.click(screen.getByRole("button", { name: "Check now" }));
  await screen.findByText("src/a.rs:10");
}

beforeEach(() => {
  resetConventions();
  openProject(7);
  setLanguage("en");
  config.mockReturnValue(SETTINGS);
  rows.mockReturnValue([{ id: 7, name: "app", kind: "folder", location: "/work/app" }]);
});

describe("the screen Conventions", () => {
  it("shows nothing but the button before the first check, then the numbers and the findings", async () => {
    serveCheck();
    render(() => <ConventionsScreen />);
    expect(screen.getByText(/No result yet/)).toBeInTheDocument();
    expect(fetchMock).not.toHaveBeenCalled();
    await check();
    expect(fetchMock.mock.calls[0]?.[0]).toMatch(/conventions\.check$/);
    expect(screen.getByText("212")).toBeInTheDocument();
    expect(screen.getByText("52 lines, at most 40")).toBeInTheDocument();
    expect(screen.getByText("91 % like src/shared/size.ts:12")).toBeInTheDocument();
    expect(screen.getByText("Complexity 19, at most 15")).toBeInTheDocument();
    expect(screen.getByText(/CHK-101/)).toBeInTheDocument();
    expect(screen.getByText("docs/old.vb PRS-501")).toBeInTheDocument();
  });

  it("sorts by a column, turns the order around and takes the sort away with the next presses", async () => {
    serveCheck();
    render(() => <ConventionsScreen />);
    await check();
    const places = () =>
      within(screen.getAllByRole("rowgroup")[1] as HTMLElement)
        .getAllByRole("row")
        .map((row) => within(row).getAllByRole("cell")[1]?.textContent);
    expect(places()).toEqual(["src/a.rs:10", "src/web/format.ts:40", "src/rules/judge.rs:203"]);
    fireEvent.click(screen.getByRole("button", { name: "Place" }));
    expect(places()).toEqual(["src/a.rs:10", "src/rules/judge.rs:203", "src/web/format.ts:40"]);
    fireEvent.click(screen.getByRole("button", { name: /Place/ }));
    expect(places()).toEqual(["src/web/format.ts:40", "src/rules/judge.rs:203", "src/a.rs:10"]);
    // The third press takes the sort away again: the order of the server comes back.
    fireEvent.click(screen.getByRole("button", { name: /Place/ }));
    expect(places()).toEqual(["src/a.rs:10", "src/web/format.ts:40", "src/rules/judge.rs:203"]);
  });

  it("says that nothing was found and still names what was not checked", async () => {
    serveCheck({ ...ANSWER, total: 0, findings: [], unchecked: [] });
    render(() => <ConventionsScreen />);
    fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByText("No findings in 212 files.")).toBeInTheDocument();
    expect(screen.getByText(/CHK-101/)).toBeInTheDocument();
  });

  it("opens the file at the line of a finding in the editor of the settings", async () => {
    serveCheck();
    render(() => <ConventionsScreen />);
    await check();
    fireEvent.click(screen.getAllByRole("button", { name: "Open" })[0] as HTMLElement);
    expect(open).toHaveBeenCalledWith("vscode://file/work/app/src/a.rs:10");
  });

  it("switches the open button off without an editor and says why", async () => {
    config.mockReturnValue(null);
    serveCheck();
    render(() => <ConventionsScreen />);
    await check();
    const button = screen.getAllByRole("button", { name: "Open" })[0] as HTMLElement;
    expect(button).toBeDisabled();
    expect(button.getAttribute("title")).toMatch(/editor of the settings is not available/);
  });

  it("switches the check off for a Git project and says why", () => {
    rows.mockReturnValue([{ id: 7, name: "app", kind: "git", location: "https://x.test/a.git" }]);
    render(() => <ConventionsScreen />);
    const button = screen.getByRole("button", { name: "Check now" });
    expect(button).toBeDisabled();
    expect(button.getAttribute("title")).toMatch(/Git address/);
  });

  it("says how many findings the list does not hold", async () => {
    serveCheck({ ...ANSWER, total: ANSWER.findings.length + 3 });
    render(() => <ConventionsScreen />);
    await check();
    expect(screen.getByText(/^3 more findings are not listed/)).toBeInTheDocument();
  });

  it("shows the failure of the check with its code", async () => {
    serveCheck({ code: "PRJ-101" }, 400);
    render(() => <ConventionsScreen />);
    fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(screen.queryByText(/No result yet/)).not.toBeInTheDocument();
  });

  it("copies the result as Markdown and says so", async () => {
    serveCheck();
    const write = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText: write } });
    render(() => <ConventionsScreen />);
    expect(screen.getByRole("button", { name: "Copy as Markdown" })).toBeDisabled();
    await check();
    fireEvent.click(screen.getByRole("button", { name: "Copy as Markdown" }));
    await waitFor(() => expect(write).toHaveBeenCalled());
    const text = write.mock.calls[0]?.[0] as string;
    expect(text.split("\n")[0]).toBe("Convention check of app: 3 findings in 212 files");
    expect(text).toContain("- `CHK-104` · `src/a.rs:10` · a::run · 52 lines, at most 40");
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("asks for a project first", () => {
    openProject(null as unknown as number);
    render(() => <ConventionsScreen />);
    expect(screen.getByText(/Pick a project first/)).toBeInTheDocument();
  });
});
