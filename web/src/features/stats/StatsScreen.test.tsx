import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { currentPath } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { PROJECT_ROW } from "../../tests/project-row";
import { resetStats, selectedRow, statsTab } from "./stats";
import { StatsScreen } from "./StatsScreen";
import type { Overview, RankingAnswer, Row } from "./types";

function row(id: number, name: string, patch: Partial<Row> = {}): Row {
  return {
    id,
    name,
    qualified_name: `shop::${name}`,
    kind: "function",
    path: `src/shop/${name}.rs`,
    language: "rust",
    parent: null,
    visibility: "private",
    test: false,
    certain: 3,
    total: 40,
    complexity: 4,
    complexity_rating: "good",
    loc: 12,
    efferent: 2,
    afferent: 1,
    assessment: null,
    reason: null,
    ...patch,
  };
}

/** A symbol without usages that is very likely unused. */
function unusedRow(id: number, name: string): Row {
  return row(id, name, { certain: 0, total: 0, assessment: "likely", reason: "private" });
}

const UNUSED = { likely: 2, public: 3, approximate: 1, tests_hidden: 7, left_out: 0 };
const OVERVIEW: Overview = {
  snapshot: 5,
  files: 12,
  symbols: 300,
  ranked: 120,
  references: { certain: 100, probable: 900 },
  unused: UNUSED,
  complexity: {
    limit: 10,
    over_limit: 1,
    histogram: [
      { complexity: 1, count: 90 },
      { complexity: 17, count: 1 },
    ],
  },
  top: {
    most_used: [row(1, "Cart")],
    unused: [row(2, "orphan", { certain: 0, total: 0, assessment: "likely", reason: "private" })],
    complexity: [row(3, "checkout", { complexity: 17, complexity_rating: "bad" })],
    coupling: [row(4, "wire", { efferent: 22 })],
  },
};
const RANKING: RankingAnswer = {
  snapshot: 5,
  rows: [row(1, "Cart"), row(5, "Order", { certain: 1, total: 9 })],
  total: 2,
  unused: null,
  left_out: [],
};

useServerSession();

beforeEach(() => {
  resetStats();
  openProject(1);
  setLanguage("en");
});

function served(
  ranking: RankingAnswer = RANKING,
  overview: Overview = OVERVIEW,
  more: Record<string, () => Response> = {},
) {
  serve({
    "projects.list": () => answer([PROJECT_ROW]),
    "stats.overview": () => answer(overview),
    "stats.ranking": () => answer(ranking),
    ...more,
  });
}

/** The inputs of all calls of one command so far. */
function inputs(command: string): Record<string, unknown>[] {
  return fetchMock.mock.calls
    .filter(([path]) => path === `/api/v1/commands/${command}`)
    .map(([, init]) => JSON.parse(((init as RequestInit).body as string | undefined) ?? "{}"));
}

async function showRanking(name: string) {
  render(() => <StatsScreen />);
  fireEvent.click(await screen.findByRole("tab", { name: new RegExp(name) }));
  return screen.findByRole("table", { name: "Ranking" });
}

describe("StatsScreen: overview", () => {
  it("shows the numbers of the project with both counts of usages", async () => {
    served();
    render(() => <StatsScreen />);
    expect(await screen.findByText("100 certain, 900 probable")).toBeInTheDocument();
    expect(screen.getByText("120 types and functions")).toBeInTheDocument();
    expect(screen.getByText("3 certain, up to 40")).toBeInTheDocument();
    expect(screen.getByText("very likely unused")).toBeInTheDocument();
  });

  it("counts the unused symbols on the tab", async () => {
    served();
    render(() => <StatsScreen />);
    const tab = await screen.findByRole("tab", { name: /No usage found/ });
    await waitFor(() => expect(within(tab).getByText("6")).toBeInTheDocument());
  });

  it("opens a row of a card in the explorer", async () => {
    served();
    render(() => <StatsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Open checkout in the Explorer" }));
    expect(currentPath()).toBe("/explorer");
  });

  it("leads from a card to its ranking", async () => {
    served();
    render(() => <StatsScreen />);
    const [first] = await screen.findAllByRole("button", { name: "Show all" });
    fireEvent.click(first as HTMLElement);
    expect(statsTab()).toBe("most_used");
    expect(await screen.findByRole("table", { name: "Ranking" })).toBeInTheDocument();
  });

  it("says so when the project has no scan", async () => {
    served(RANKING, { ...OVERVIEW, snapshot: null });
    render(() => <StatsScreen />);
    expect(await screen.findByText(/has not been scanned yet/)).toBeInTheDocument();
  });

  it("asks for nothing while no project is open", async () => {
    openProject(null as unknown as number);
    served();
    render(() => <StatsScreen />);
    await waitFor(() => expect(inputs("projects.list")).toHaveLength(1));
    expect(inputs("stats.overview")).toHaveLength(0);
  });

  it("tells a failed load", async () => {
    serve({ "projects.list": () => answer([PROJECT_ROW]) });
    render(() => <StatsScreen />);
    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be loaded/);
  });
});

describe("StatsScreen: rankings", () => {
  it("asks for the ranking of the tab, sorted by certain usages and without tests", async () => {
    served();
    const table = await showRanking("Most used");
    expect(within(table).getByText("Order")).toBeInTheDocument();
    expect(inputs("stats.ranking").at(-1)).toMatchObject({
      project: 1,
      ranking: "most_used",
      order: "certain",
      include_tests: false,
    });
  });

  it("asks again when a filter changes", async () => {
    served();
    await showRanking("Most used");
    fireEvent.click(screen.getByRole("button", { name: "All (upper bound)" }));
    await waitFor(() => expect(inputs("stats.ranking").at(-1)).toMatchObject({ order: "all" }));
    fireEvent.click(screen.getByRole("button", { name: "Include tests" }));
    await waitFor(() =>
      expect(inputs("stats.ranking").at(-1)).toMatchObject({ include_tests: true }),
    );
    expect(inputs("stats.overview").at(-1)).toMatchObject({ include_tests: true });
  });

  it("does not carry a choice of kinds into the complexity tab, which cannot undo it", async () => {
    served();
    await showRanking("Most used");
    fireEvent.click(screen.getByRole("button", { name: "Types" }));
    await waitFor(() => expect(inputs("stats.ranking").at(-1)).toMatchObject({ kinds: "types" }));
    fireEvent.click(screen.getByRole("tab", { name: "Complexity" }));
    await waitFor(() =>
      expect(inputs("stats.ranking").at(-1)).toMatchObject({ ranking: "complexity", kinds: "all" }),
    );
  });

  it("keeps the rows of the newest filter when an older answer comes late", async () => {
    const late: ((answer: Response) => void)[] = [];
    served();
    await showRanking("Most used");
    fetchMock.mockImplementation((path: string) =>
      path.endsWith("stats.ranking")
        ? new Promise<Response>((done) => late.push(done))
        : Promise.resolve(answer(path.endsWith("stats.overview") ? OVERVIEW : [PROJECT_ROW])),
    );
    fireEvent.click(screen.getByRole("button", { name: "Types" }));
    fireEvent.click(screen.getByRole("button", { name: "Functions" }));
    await waitFor(() => expect(late).toHaveLength(2));
    late[1]?.(answer({ ...RANKING, rows: [row(7, "newest")], total: 1 }));
    expect(await screen.findByRole("row", { name: /newest/ })).toBeInTheDocument();
    late[0]?.(answer({ ...RANKING, rows: [row(8, "outdated")], total: 1 }));
    await new Promise((done) => setTimeout(done, 20));
    expect(screen.queryByRole("row", { name: /outdated/ })).toBeNull();
    expect(screen.getByRole("row", { name: /newest/ })).toBeInTheDocument();
  });

  it("stops the request of an older filter when a newer one is sent", async () => {
    const signals: (AbortSignal | undefined)[] = [];
    served();
    await showRanking("Most used");
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path.endsWith("stats.ranking")) {
        signals.push(init?.signal ?? undefined);
        return new Promise<Response>(() => undefined);
      }
      return Promise.resolve(answer(path.endsWith("stats.overview") ? OVERVIEW : [PROJECT_ROW]));
    });
    fireEvent.click(screen.getByRole("button", { name: "Types" }));
    fireEvent.click(screen.getByRole("button", { name: "Functions" }));
    await waitFor(() => expect(signals).toHaveLength(2));
    expect(signals[0]?.aborted).toBe(true);
    expect(signals[1]?.aborted).toBe(false);
  });

  it("starts again with the overview when another project is opened", async () => {
    served();
    await showRanking("Most used");
    openProject(2);
    await waitFor(() => expect(statsTab()).toBe("overview"));
    await waitFor(() => expect(inputs("stats.overview").at(-1)).toMatchObject({ project: 2 }));
  });

  it("leads from a card of types to a ranking of types", async () => {
    served();
    render(() => <StatsScreen />);
    const [first] = await screen.findAllByRole("button", { name: "Show all" });
    fireEvent.click(first as HTMLElement);
    await waitFor(() =>
      expect(inputs("stats.ranking").at(-1)).toMatchObject({
        ranking: "most_used",
        kinds: "types",
      }),
    );
  });

  it("selects the first row and names its upper bound", async () => {
    served();
    await showRanking("Most used");
    await waitFor(() => expect(selectedRow()).toBe(1));
    expect(screen.getByText(/37 of 40 usages are not confirmed/)).toBeInTheDocument();
    expect(screen.getByRole("row", { name: /Cart/ })).toHaveAttribute("aria-current", "true");
    expect(screen.getByRole("row", { name: /Order/ })).not.toHaveAttribute("aria-current", "true");
  });

  it("leaves the keys of the browser alone: U with a modifier does nothing", async () => {
    served();
    await showRanking("Most used");
    await waitFor(() => expect(selectedRow()).toBe(1));
    const before = currentPath();
    for (const modifier of [{ ctrlKey: true }, { metaKey: true }, { altKey: true }]) {
      fireEvent.keyDown(document.body, { key: "u", ...modifier });
    }
    expect(currentPath()).toBe(before);
    expect(before).not.toBe("/usages");
  });

  it("moves the selection with the arrows and opens the explorer with Enter", async () => {
    served();
    await showRanking("Most used");
    await waitFor(() => expect(selectedRow()).toBe(1));
    fireEvent.keyDown(document.body, { key: "ArrowDown" });
    expect(selectedRow()).toBe(5);
    fireEvent.keyDown(document.body, { key: "ArrowDown" });
    expect(selectedRow()).toBe(5);
    fireEvent.keyDown(document.body, { key: "Enter" });
    expect(currentPath()).toBe("/explorer");
  });

  it("shows the usages of the selected row with U", async () => {
    served();
    await showRanking("Most used");
    await waitFor(() => expect(selectedRow()).toBe(1));
    fireEvent.keyDown(document.body, { key: "u" });
    expect(currentPath()).toBe("/usages");
  });

  it("explains what no usage means and splits the rows by assessment", async () => {
    const orphan = row(2, "orphan", {
      certain: 0,
      total: 0,
      assessment: "public",
      reason: "public",
    });
    served({ ...RANKING, rows: [orphan], total: 1, unused: UNUSED });
    const table = await showRanking("No usage found");
    expect(screen.getByText(/does not mean dead code/)).toBeInTheDocument();
    expect(screen.getByText("Tests hidden")).toBeInTheDocument();
    expect(within(table).getByTitle("public: may be used from outside")).toBeInTheDocument();
    expect(
      within(table).getByText("public: users outside the project are possible"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "very likely unused" }));
    await waitFor(() =>
      expect(inputs("stats.ranking").at(-1)).toMatchObject({ assessment: "likely" }),
    );
  });

  it("lists the symbols the exception list leaves out, with the rule on request", async () => {
    const orphan = unusedRow(2, "orphan");
    const kept = unusedRow(9, "Migrate");
    const rule = { symbols: ["*.Migrate"], reason: "called by the migration tool" };
    served({
      ...RANKING,
      rows: [orphan],
      total: 1,
      unused: { ...UNUSED, left_out: 1 },
      left_out: [{ row: kept, rule }],
    });
    await showRanking("No usage found");
    expect(await screen.findByText("Left out")).toBeInTheDocument();
    expect(screen.getByText("Rule: “called by the migration tool”")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Show rule" }));
    expect(screen.getByText("*.Migrate")).toBeInTheDocument();
    expect(screen.getByText(/^1 symbol does not appear in the number above/)).toBeInTheDocument();
  });

  it("keeps a symbol on purpose with a reason and loads the statistics again", async () => {
    const orphan = unusedRow(2, "orphan");
    const ranked = { ...RANKING, rows: [orphan], total: 1, unused: UNUSED };
    served(ranked, OVERVIEW, { "unused.keep": () => answer({ entries: 1 }) });
    const table = await showRanking("No usage found");
    fireEvent.click(within(table).getByRole("button", { name: "Keep orphan on purpose" }));
    fireEvent.input(screen.getByRole("textbox", { name: "Reason" }), {
      target: { value: "called by reflection" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(inputs("unused.keep")).toHaveLength(1));
    expect(inputs("unused.keep")[0]).toMatchObject({ project: 1, symbol: "shop::orphan" });
    expect(String(inputs("unused.keep")[0]?.reason)).toMatch(
      /^called by reflection \(\d{4}-\d{2}-\d{2}\)$/,
    );
    await waitFor(() => expect(inputs("stats.ranking").length).toBeGreaterThan(1));
  });

  it("disables the keep button with a hint for a project that is not a folder", async () => {
    const orphan = unusedRow(2, "orphan");
    const ranked = { ...RANKING, rows: [orphan], total: 1, unused: UNUSED };
    served(ranked, OVERVIEW, {
      "projects.list": () => answer([{ ...PROJECT_ROW, kind: "git" }]),
    });
    const table = await showRanking("No usage found");
    const button = within(table).getByRole("button", { name: "Keep orphan on purpose" });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute("title", expect.stringContaining("no project file"));
  });

  it("says why an exception could not be written", async () => {
    const orphan = unusedRow(2, "orphan");
    const ranked = { ...RANKING, rows: [orphan], total: 1, unused: UNUSED };
    served(ranked, OVERVIEW, { "unused.keep": () => answer({ code: "CFG-102" }, 422) });
    const table = await showRanking("No usage found");
    fireEvent.click(within(table).getByRole("button", { name: "Keep orphan on purpose" }));
    fireEvent.input(screen.getByRole("textbox", { name: "Reason" }), { target: { value: "x" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be changed/);
  });

  it("lifts an exception with the entry as the list shows it", async () => {
    const kept = unusedRow(9, "Migrate");
    const rule = { symbols: ["*.Migrate"], reason: "called by the migration tool" };
    const ranked = {
      ...RANKING,
      rows: [row(2, "orphan")],
      total: 1,
      unused: { ...UNUSED, left_out: 1 },
      left_out: [{ row: kept, rule }],
    };
    served(ranked, OVERVIEW, { "unused.lift": () => answer({ entries: 0 }) });
    await showRanking("No usage found");
    fireEvent.click(
      await screen.findByRole("button", { name: "Lift the rule that leaves out Migrate" }),
    );
    await waitFor(() => expect(inputs("unused.lift")).toHaveLength(1));
    expect(inputs("unused.lift")[0]).toEqual({ project: 1, paths: [], ...rule });
  });

  it("counts all the symbols the list leaves out, not only the ones it shows", async () => {
    const rule = { symbols: ["*.Migrate"], reason: "called by the migration tool" };
    served({
      ...RANKING,
      rows: [row(2, "orphan")],
      total: 1,
      unused: { ...UNUSED, left_out: 5 },
      left_out: [{ row: row(1, "Cart"), rule }],
    });
    await showRanking("No usage found");
    expect(
      await screen.findByText(/^5 symbols do not appear in the number above/),
    ).toBeInTheDocument();
    expect(screen.getByText("1 of 5")).toBeInTheDocument();
  });

  it("draws the histogram and the hotspots in the complexity tab", async () => {
    served({ ...RANKING, rows: [row(3, "checkout", { complexity: 17 }), row(1, "Cart")] });
    await showRanking("Complexity");
    expect(await screen.findByRole("img", { name: /How complex/ })).toBeInTheDocument();
    expect(screen.getByText("Complexity 11–19: 1 functions")).toBeInTheDocument();
    expect(screen.getByText("above the limit")).toBeInTheDocument();
    expect(screen.getByText("Cart: complexity 4, 3 certain usages, up to 40")).toBeInTheDocument();
  });

  it("says so when a ranking is empty", async () => {
    served({ ...RANKING, rows: [], total: 0 });
    render(() => <StatsScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: /Size/ }));
    expect(await screen.findByText("Nothing to show here.")).toBeInTheDocument();
  });
});
