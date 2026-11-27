import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { useClipboard } from "../../tests/clipboard";
import { setLanguage } from "../../shared/i18n/i18n";
import { compareWithCurrent, resetChange, showChangeTab } from "./change";
import { ChangeScreen } from "./ChangeScreen";
import { TAB } from "./consts";
import type { ChangeAnswer, MetricsAnswer, SymbolOut, UsageOut } from "./types";

useServerSession();

function symbol(id: number, name: string, signature: string, visibility = "public"): SymbolOut {
  return {
    id,
    kind: "method",
    qualified_name: name,
    signature,
    visibility,
    path: `src/${name}.cs`,
  };
}

function usage(path: string, line: number, status: string, confidence = "certain"): UsageOut {
  return {
    from: { id: line, kind: "method", qualified_name: `Shop.Checkout.Flow${line}` },
    path,
    line,
    column: 5,
    kind: "call",
    confidence,
    status,
  };
}

const SCANS = {
  schema: 1,
  keep: 5,
  average_ms: 2000,
  scans: [
    {
      snapshot: 9,
      scan: 9,
      created: 1_800_000_000,
      commit: "a41c9e2ffff",
      mode: "full",
      files: 3,
      symbols: 9,
      duration_ms: 2400,
      files_changed: 2,
    },
    {
      snapshot: 8,
      scan: 8,
      created: 1_799_990_000,
      commit: null,
      mode: "incremental",
      files: 3,
      symbols: 9,
      duration_ms: 600,
      files_changed: 1,
    },
    {
      snapshot: 7,
      scan: 7,
      created: 1_799_900_000,
      commit: null,
      mode: "full",
      files: 2,
      symbols: 7,
      duration_ms: null,
      files_changed: null,
    },
  ],
};

const ROUND = symbol(1, "Price.Round", "decimal Round(decimal value)");
const ROUND_NEW = symbol(1, "Price.Round", "decimal Round(decimal value, RoundingMode mode)");
const LEGACY = symbol(2, "LegacyPricing.Calculate", "decimal Calculate()");

const ANSWER: ChangeAnswer = {
  schema: 1,
  project: 1,
  base: SCANS.scans[1] as ChangeAnswer["base"],
  target: SCANS.scans[0] as ChangeAnswer["target"],
  summary: {
    changed_symbols: 2,
    public_symbols: 2,
    usages_certain: 2,
    usages_probable: 1,
    usages_not_touched: 2,
  },
  added: {
    total: 1,
    is_cut: false,
    items: [symbol(5, "DiscountPolicy", "class DiscountPolicy", "internal")],
  },
  removed: { total: 1, is_cut: false, items: [LEGACY] },
  changed: {
    total: 1,
    is_cut: false,
    items: [{ base: ROUND, target: ROUND_NEW, reasons: ["signature"], narrowed: false }],
  },
  moved: {
    total: 1,
    is_cut: false,
    items: [
      {
        base: { ...symbol(6, "Money.Format", "string Format()"), path: "utils/money.ts" },
        target: { ...symbol(6, "Money.Format", "string Format()"), path: "shared/money.ts" },
        reasons: [],
        narrowed: false,
      },
    ],
  },
  critical: [{ annotation: 3, kind: "method", qualified_name: "Price.Round", outcome: "changed" }],
  layers: { new_violations: [{ from: "Api", to: "Data", count: 2 }], new_cycles: [] },
  impact: {
    total: 2,
    is_cut: false,
    items: [
      {
        symbol: ROUND,
        cause: "changed",
        reasons: ["signature"],
        narrowed: false,
        not_touched: 2,
        usages: {
          total: 3,
          is_cut: false,
          items: [
            usage("Checkout/Flow.cs", 118, "not_touched"),
            usage("Checkout/Flow.cs", 164, "not_touched", "probable"),
            usage("Orders/Service.cs", 88, "adapted"),
          ],
        },
      },
      {
        symbol: LEGACY,
        cause: "removed",
        reasons: [],
        narrowed: false,
        not_touched: 0,
        usages: { total: 1, is_cut: false, items: [usage("Reports/Month.cs", 57, "adapted")] },
      },
    ],
  },
  is_cut: false,
  direct_usages_only: true,
};

function serveChange(extra: Record<string, () => Response> = {}) {
  serve({
    "scans.list": () => answer(SCANS),
    "scans.impact": () => answer(ANSWER),
    ...extra,
  });
}

function bodiesOf(command: string): unknown[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith(command))
    .map(([, init]) => JSON.parse(String((init as RequestInit).body)));
}

beforeEach(() => {
  resetChange();
  showChangeTab(TAB.impact);
  openProject(1);
});

describe("ChangeScreen, view Impact", () => {
  it("shows the figures, the symbols with their state, the notes and the checklist of the first symbol", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    expect(await screen.findByText("changed symbols")).toBeInTheDocument();
    expect(screen.getByText("+1 ~1 −1")).toBeInTheDocument();
    const table = within(screen.getByRole("grid", { name: "Symbols the change touches" }));
    expect(table.getByText("LegacyPricing.Calculate")).toBeInTheDocument();
    expect(table.getByText("adapted")).toBeInTheDocument();
    expect(table.getByText("2 open")).toBeInTheDocument();
    expect(table.getAllByText("no checklist").length).toBeGreaterThan(0);
    expect(screen.getByText("1 new layer violation")).toBeInTheDocument();
    expect(screen.getByText("Api uses Data (2 references)")).toBeInTheDocument();
    expect(
      screen.getByText("Price.Round is marked as system critical and was changed."),
    ).toBeInTheDocument();
    const panel = within(screen.getByRole("region", { name: "Usages of the selected symbol" }));
    expect(panel.getByText("Changed signature")).toBeInTheDocument();
    expect(panel.getByText(/Only direct usages are known/)).toBeInTheDocument();
  });

  it("selects a row with the arrow keys and lists only the open places until 'All' is pressed", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    const grid = await screen.findByRole("grid", { name: "Symbols the change touches" });
    grid.focus();
    fireEvent.keyDown(grid, { key: "ArrowUp" });
    expect(
      within(screen.getByRole("region", { name: "Usages of the selected symbol" })).getByText(
        "Removed symbol",
      ),
    ).toBeInTheDocument();
    fireEvent.keyDown(grid, { key: "ArrowDown" });
    const panel = within(screen.getByRole("region", { name: "Usages of the selected symbol" }));
    expect(panel.getByText("Changed signature")).toBeInTheDocument();
    expect(panel.getByText("decimal Round(decimal value)")).toBeInTheDocument();
    expect(panel.getByText("decimal Round(decimal value, RoundingMode mode)")).toBeInTheDocument();
    expect(panel.queryByText("Shop.Checkout.Flow88")).not.toBeInTheDocument();
    expect(panel.getAllByText("not touched")).toHaveLength(2);
    fireEvent.click(panel.getByRole("button", { name: "All · 3" }));
    expect(panel.getByText("Shop.Checkout.Flow88")).toBeInTheDocument();
    expect(panel.getByText("adapted in the change")).toBeInTheDocument();
  });

  it("moves between the views with the arrow keys of the tab list", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    const impact = await screen.findByRole("tab", { name: "Impact" });
    impact.focus();
    fireEvent.keyDown(impact, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Comparison", selected: true })).toBeInTheDocument();
    expect(
      await screen.findByText(
        'Two states of the same project. "Impact" shows the places that are hit.',
      ),
    ).toBeInTheDocument();
  });
});

const SEARCH = "Search by qualified name";
const REGION = { name: "Usages of the selected symbol" };

describe("ChangeScreen, filter of the symbols", () => {
  it("searches by qualified name, counts the rows and moves the selection off a hidden row", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    const search = await screen.findByRole("searchbox", { name: SEARCH });
    expect(screen.getByText("4 of 4")).toBeInTheDocument();
    expect(within(screen.getByRole("region", REGION)).getByText("Changed signature")).toBeVisible();
    fireEvent.input(search, { target: { value: "legacy" } });
    expect(screen.getByText("1 of 4")).toBeInTheDocument();
    const table = within(screen.getByRole("grid", { name: "Symbols the change touches" }));
    expect(table.queryByText("Price.Round")).not.toBeInTheDocument();
    expect(within(screen.getByRole("region", REGION)).getByText("Removed symbol")).toBeVisible();
  });

  it("keeps only the public symbols, the kinds of change, the kinds of symbol and the open ones", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    await screen.findByRole("searchbox", { name: SEARCH });
    const table = () => within(screen.getByRole("grid", { name: "Symbols the change touches" }));
    fireEvent.click(screen.getByRole("button", { name: "Only public" }));
    expect(screen.getByText("3 of 4")).toBeInTheDocument();
    expect(table().queryByText("DiscountPolicy")).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox", { name: "Kind of change" }), {
      target: { value: "moved" },
    });
    expect(screen.getByText("1 of 4")).toBeInTheDocument();
    expect(table().getByText("Money.Format")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset filter" }));
    expect(screen.getByText("4 of 4")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Only with open places" }));
    expect(screen.getByText("1 of 4")).toBeInTheDocument();
    expect(table().getByText("2 open")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset filter" }));
    fireEvent.change(screen.getByRole("combobox", { name: "Kind of symbol" }), {
      target: { value: "class" },
    });
    expect(screen.getByText("No symbol matches the filter.")).toBeInTheDocument();
    expect(screen.queryByRole("region", REGION)).not.toBeInTheDocument();
  });

  it("says so when no symbol matches, and the reset button brings the list back", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    const search = await screen.findByRole("searchbox", { name: SEARCH });
    fireEvent.input(search, { target: { value: "no such symbol" } });
    expect(screen.getByText("No symbol matches the filter.")).toBeInTheDocument();
    expect(screen.getByText("0 of 4")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset filter" }));
    expect(screen.getByRole("grid", { name: "Symbols the change touches" })).toBeInTheDocument();
  });

  it("is honest about a cut answer: it counts the listed rows and names the total", async () => {
    serveChange({
      "scans.impact": () =>
        answer({
          ...ANSWER,
          is_cut: true,
          changed: { ...ANSWER.changed, total: 2305, is_cut: true },
        }),
    });
    render(() => <ChangeScreen />);
    expect(await screen.findByText("4 of 4 listed (2,308 in all)")).toBeInTheDocument();
  });

  it("filters the list of the view Comparison too, with the filter for open places greyed out", async () => {
    showChangeTab(TAB.compare);
    serveChange();
    render(() => <ChangeScreen />);
    const search = await screen.findByRole("searchbox", { name: SEARCH });
    const open = screen.getByRole("button", { name: "Only with open places" });
    expect(open).toBeDisabled();
    expect(open).toHaveAttribute("title", expect.stringContaining("no checklist"));
    fireEvent.input(search, { target: { value: "money" } });
    const list = within(screen.getByRole("table", { name: "The symbols of the difference" }));
    expect(list.getByText("Money.Format")).toBeInTheDocument();
    expect(list.queryByText("Price.Round")).not.toBeInTheDocument();
    expect(screen.getByText("1 of 4")).toBeInTheDocument();
  });
});

/** The names in the first column of the rows of a table, in the order shown. */
function firstColumn(table: HTMLElement): string[] {
  return within(table)
    .getAllByRole("row")
    .slice(1)
    .map((row) => row.querySelector("td")?.textContent ?? "");
}

describe("ChangeScreen, order of the lists", () => {
  it("sorts the table of the impact by a column head, after the filter, and keeps the selection", async () => {
    serveChange();
    render(() => <ChangeScreen />);
    const grid = await screen.findByRole("grid", { name: "Symbols the change touches" });
    const selected = () => grid.querySelector("tr[aria-selected='true']")?.textContent ?? "";
    expect(selected()).toContain("Price.Round");
    fireEvent.click(within(grid).getByRole("button", { name: "Symbol" }));
    expect(firstColumn(grid).map((text) => text.replace(/^\W*\w\s?/, ""))).toEqual([
      expect.stringContaining("DiscountPolicy"),
      expect.stringContaining("LegacyPricing"),
      expect.stringContaining("Money.Format"),
      expect.stringContaining("Price.Round"),
    ]);
    expect(within(grid).getByRole("columnheader", { name: /Symbol/ })).toHaveAttribute(
      "aria-sort",
      "ascending",
    );
    expect(selected()).toContain("Price.Round");
    fireEvent.click(within(grid).getByRole("button", { name: "Symbol" }));
    expect(firstColumn(grid)[0]).toContain("Price.Round");
    fireEvent.click(within(grid).getByRole("button", { name: "Symbol" }));
    expect(firstColumn(grid)[0]).toContain("LegacyPricing");
    fireEvent.click(within(grid).getByRole("button", { name: "Symbol" }));
    fireEvent.input(screen.getByRole("searchbox", { name: SEARCH }), { target: { value: "o" } });
    expect(screen.getByText("3 of 4")).toBeInTheDocument();
  });

  it("sorts the list of the comparison by a column head and filters first", async () => {
    showChangeTab(TAB.compare);
    serveChange();
    render(() => <ChangeScreen />);
    const list = await screen.findByRole("table", { name: "The symbols of the difference" });
    fireEvent.click(within(list).getByRole("button", { name: "File" }));
    const files = within(list)
      .getAllByRole("row")
      .slice(1)
      .map((row) => row.querySelectorAll("td")[2]?.textContent);
    expect(files).toEqual([...files].sort((a, b) => (a ?? "").localeCompare(b ?? "")));
    fireEvent.input(screen.getByRole("searchbox", { name: SEARCH }), {
      target: { value: "money" },
    });
    expect(within(list).getAllByRole("row")).toHaveLength(2);
  });
});

describe("ChangeScreen, button Copy as Markdown", () => {
  const setClipboard = useClipboard();
  const TEXT = { markdown: "<!-- codeskimr-change -->\n## Impact" };

  it("asks for the text of the two scans shown, in the language of the UI, and copies it", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    setClipboard({ writeText });
    setLanguage("de");
    serveChange({ "scans.impact_markdown": () => answer(TEXT) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Als Markdown kopieren" }));
    expect(await screen.findByRole("button", { name: "Kopiert" })).toBeInTheDocument();
    expect(writeText).toHaveBeenCalledWith(TEXT.markdown);
    expect(bodiesOf("scans.impact_markdown")).toEqual([
      { project: 1, base: ANSWER.base.snapshot, target: ANSWER.target.snapshot, language: "de" },
    ]);
  });

  it("copies the metric changes of the level shown in the view Metrics", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    setClipboard({ writeText });
    const metrics = { markdown: "<!-- codeskimr-metrics -->\n## Metrics" };
    serveChange({
      "scans.metrics": () => answer(METRICS),
      "scans.metrics_markdown": () => answer(metrics),
    });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    await screen.findByText("Got worse", { selector: "h2" });
    fireEvent.click(screen.getByRole("button", { name: "Copy as Markdown" }));
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
    expect(writeText).toHaveBeenCalledWith(metrics.markdown);
    expect(bodiesOf("scans.metrics_markdown")).toEqual([
      {
        project: 1,
        base: ANSWER.base.snapshot,
        target: ANSWER.target.snapshot,
        level: "member",
        language: "en",
      },
    ]);
  });

  it("copies only the metric that is chosen, as the lists show it", async () => {
    setClipboard({ writeText: vi.fn().mockResolvedValue(undefined) });
    serveChange({
      "scans.metrics": () => answer(METRICS),
      "scans.metrics_markdown": () => answer({ markdown: "x" }),
    });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    fireEvent.click(await screen.findByRole("button", { name: "Maintainability" }));
    fireEvent.click(screen.getByRole("button", { name: "Copy as Markdown" }));
    await screen.findByRole("button", { name: "Copied" });
    expect(bodiesOf("scans.metrics_markdown").at(-1)).toMatchObject({ metrics: ["mi"] });
  });

  it("says so when the browser has no clipboard", async () => {
    setClipboard(undefined);
    serveChange({ "scans.impact_markdown": () => answer(TEXT) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Copy as Markdown" }));
    expect(await screen.findByRole("button", { name: "Not copied" })).toBeInTheDocument();
  });

  it("says so when the command fails, and is not offered in the view Comparison", async () => {
    setClipboard({ writeText: vi.fn() });
    serveChange({ "scans.impact_markdown": () => answer({ code: "PRJ-201" }, 404) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Copy as Markdown" }));
    expect(await screen.findByRole("button", { name: "Not copied" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", { name: "Comparison" }));
    expect(screen.queryByRole("button", { name: /Not copied|Copy as Markdown/ })).toBeNull();
  });
});

describe("ChangeScreen, moved and cut", () => {
  it("lists a moved symbol that changed its signature with its checklist", async () => {
    const moved = {
      base: { ...symbol(6, "Money.Format", "string Format(int a)"), path: "utils/money.ts" },
      target: { ...symbol(6, "Money.Format", "string Format(long a)"), path: "shared/money.ts" },
      reasons: ["signature"],
      narrowed: false,
    };
    const impact = {
      symbol: moved.base,
      cause: "changed",
      reasons: ["signature"],
      narrowed: false,
      not_touched: 1,
      usages: { total: 1, is_cut: false, items: [usage("Web/Cart.cs", 17, "not_touched")] },
    };
    serveChange({
      "scans.impact": () =>
        answer({
          ...ANSWER,
          changed: { total: 0, is_cut: false, items: [] },
          removed: { total: 0, is_cut: false, items: [] },
          added: { total: 0, is_cut: false, items: [] },
          moved: { total: 1, is_cut: false, items: [moved] },
          impact: { total: 1, is_cut: false, items: [impact] },
        }),
    });
    render(() => <ChangeScreen />);
    const table = within(await screen.findByRole("grid", { name: "Symbols the change touches" }));
    expect(table.getByText("Money.Format")).toBeInTheDocument();
    expect(table.getByText("moved")).toBeInTheDocument();
    expect(table.getByText("1 open")).toBeInTheDocument();
    expect(screen.getByText("Shop.Checkout.Flow17")).toBeInTheDocument();
  });

  it("says that the counts of the reasons cover only the listed symbols when the list was cut", async () => {
    showChangeTab(TAB.compare);
    serveChange({
      "scans.impact": () =>
        answer({ ...ANSWER, is_cut: true, changed: { ...ANSWER.changed, total: 9, is_cut: true } }),
    });
    render(() => <ChangeScreen />);
    expect(
      await screen.findByText("Of the 1 listed: Signature 1 · Visibility 0 · Content 0"),
    ).toBeInTheDocument();
    expect(screen.getByText("9")).toBeInTheDocument();
  });
});

describe("ChangeScreen, view Comparison", () => {
  it("shows the four counts, the list of the symbols and the selectors with the scans of the answer", async () => {
    showChangeTab(TAB.compare);
    serveChange();
    render(() => <ChangeScreen />);
    expect(await screen.findByText("Signature 1 · Visibility 0 · Content 0")).toBeInTheDocument();
    const list = within(screen.getByRole("table", { name: "The symbols of the difference" }));
    expect(list.getByText("utils/money.ts → shared/money.ts")).toBeInTheDocument();
    expect(list.getByText("Price.Round")).toBeInTheDocument();
    expect(list.getAllByText("removed")).toHaveLength(1);
    expect(screen.getByLabelText("Base (before)")).toHaveValue("8");
    expect(screen.getByLabelText("Target (after)")).toHaveValue("9");
  });

  it("offers only older scans as the base and compares the pick with the button", async () => {
    showChangeTab(TAB.compare);
    serveChange();
    render(() => <ChangeScreen />);
    const base = await screen.findByLabelText("Base (before)");
    const run = screen.getByRole("button", { name: "Compare" });
    await vi.waitFor(() => expect(run).toBeEnabled());
    expect(within(base).getAllByRole("option")).toHaveLength(2);
    fireEvent.change(base, { target: { value: "7" } });
    fireEvent.click(run);
    await vi.waitFor(() => expect(bodiesOf("scans.impact")).toHaveLength(2));
    expect(bodiesOf("scans.impact")[1]).toEqual({ project: 1, base: 7, target: 9 });
  });

  it("starts with the scan the history asked for as the base and the newest scan as the target", async () => {
    serveChange();
    compareWithCurrent(1, 7);
    render(() => <ChangeScreen />);
    expect(
      await screen.findByRole("tab", { name: "Comparison", selected: true }),
    ).toBeInTheDocument();
    await vi.waitFor(() => expect(bodiesOf("scans.impact")).toHaveLength(1));
    expect(bodiesOf("scans.impact")[0]).toEqual({ project: 1, base: 7 });
  });
});

describe("ChangeScreen, scans without a change", () => {
  /** Three scans in a row: the first wrote snapshot 9, the two after it found nothing changed. */
  const REPEATED = {
    ...SCANS,
    scans: [30, 29, 28].map((scan) => ({
      ...SCANS.scans[0],
      scan,
      files_changed: scan === 28 ? 2 : 0,
    })),
  };
  const SAME = {
    ...ANSWER,
    base: REPEATED.scans[1] as ChangeAnswer["base"],
    target: REPEATED.scans[0] as ChangeAnswer["target"],
    added: { total: 0, is_cut: false, items: [] },
    removed: { total: 0, is_cut: false, items: [] },
    changed: { total: 0, is_cut: false, items: [] },
    moved: { total: 0, is_cut: false, items: [] },
    impact: { total: 0, is_cut: false, items: [] },
    layers: { new_violations: [], new_cycles: [] },
    critical: [],
  };

  it("compares the two newest scans and says that nothing changed", async () => {
    serveChange({ "scans.list": () => answer(REPEATED), "scans.impact": () => answer(SAME) });
    render(() => <ChangeScreen />);
    expect(
      await screen.findByText(/Nothing changed: both scans have the same state/),
    ).toBeInTheDocument();
    expect(bodiesOf("scans.impact")[0]).toEqual({ project: 1 });
  });

  it("names the scans by their snapshot and keeps the pick of the scan the user made", async () => {
    showChangeTab(TAB.compare);
    serveChange({ "scans.list": () => answer(REPEATED), "scans.impact": () => answer(SAME) });
    render(() => <ChangeScreen />);
    const base = await screen.findByLabelText("Base (before)");
    expect(base).toHaveValue("29");
    const run = screen.getByRole("button", { name: "Compare" });
    await vi.waitFor(() => expect(run).toBeEnabled());
    fireEvent.change(base, { target: { value: "28" } });
    fireEvent.click(run);
    await vi.waitFor(() => expect(bodiesOf("scans.impact")).toHaveLength(2));
    expect(bodiesOf("scans.impact")[1]).toEqual({ project: 1, base: 9, target: 9 });
    expect(screen.getByLabelText("Base (before)")).toHaveValue("28");
  });
});

describe("ChangeScreen, without an answer", () => {
  it("says that two scans are needed when the project has fewer", async () => {
    serve({ "scans.list": () => answer({ ...SCANS, scans: SCANS.scans.slice(0, 1) }) });
    render(() => <ChangeScreen />);
    expect(await screen.findByText(/fewer than two scans/)).toBeInTheDocument();
    expect(bodiesOf("scans.impact")).toHaveLength(0);
  });

  it("names the code of a failed comparison", async () => {
    serveChange({ "scans.impact": () => answer({ code: "CMP-301" }, 409) });
    render(() => <ChangeScreen />);
    expect(await screen.findByRole("alert")).toHaveTextContent("CMP-301");
  });

  it("asks for a project first", () => {
    openProject(0);
    fetchMock.mockReset();
    render(() => <ChangeScreen />);
    expect(screen.getByText(/Pick a project first/)).toBeInTheDocument();
  });
});

const METRICS: MetricsAnswer = {
  schema: 1,
  project: 1,
  base: ANSWER.base,
  target: ANSWER.target,
  level: "member",
  looked_at: 10,
  worse: {
    total: 1,
    is_cut: false,
    items: [
      {
        name: "Shop.Checkout.Total",
        kind: "method",
        path: "src/Checkout.cs",
        verdict: "mixed",
        crosses: true,
        moves: [
          { metric: "cc", from: 6, to: 14, direction: "worse", crossing: "into_poor" },
          { metric: "loc", from: 22, to: 47, direction: "grew", crossing: null },
        ],
      },
    ],
  },
  better: {
    total: 2,
    is_cut: false,
    items: [
      {
        name: "Shop.Checkout.Total",
        kind: "method",
        path: "src/Checkout.cs",
        verdict: "mixed",
        crosses: false,
        moves: [{ metric: "mi", from: 49.5, to: 61, direction: "better", crossing: null }],
      },
      {
        name: "Shop.Legacy.Run",
        kind: "method",
        path: "src/Legacy.cs",
        verdict: "better",
        crosses: true,
        moves: [{ metric: "cc", from: 15, to: 9, direction: "better", crossing: "out_of_poor" }],
      },
    ],
  },
  is_cut: false,
};

describe("ChangeScreen, view Metrics", () => {
  it("reads the metrics of the compared pair and shows both lists with their moves", async () => {
    serveChange({ "scans.metrics": () => answer(METRICS) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    expect(await screen.findByText("Got worse", { selector: "h2" })).toBeInTheDocument();
    expect(bodiesOf("scans.metrics")).toEqual([
      { project: 1, base: 8, target: 9, level: "member" },
    ]);
    const worse = within(screen.getByRole("region", { name: "Got worse" }));
    expect(worse.getAllByText("Shop.Checkout.Total")).toHaveLength(2);
    expect(worse.getByText("6 → 14")).toBeInTheDocument();
    expect(worse.getByText("+8")).toBeInTheDocument();
    expect(worse.getByText("past the limit")).toBeInTheDocument();
    const better = within(screen.getByRole("region", { name: "Got better" }));
    expect(better.getByText("49.5 → 61")).toBeInTheDocument();
    expect(better.getByText("−6")).toBeInTheDocument();
    expect(better.getByText("back under the limit")).toBeInTheDocument();
    expect(screen.getByText("objects, 1 past a limit")).toBeInTheDocument();
    expect(screen.getByText("not listed")).toBeInTheDocument();
    expect(screen.getByText("new 1 · removed 1")).toBeInTheDocument();
  });

  it("narrows the lists to one metric and reads again for another level", async () => {
    serveChange({ "scans.metrics": () => answer(METRICS) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    await screen.findByText("Shop.Legacy.Run");
    fireEvent.click(screen.getByRole("button", { name: "Maintainability" }));
    expect(screen.queryByText("Shop.Legacy.Run")).not.toBeInTheDocument();
    expect(screen.queryByText("+8")).not.toBeInTheDocument();
    expect(screen.getByText("49.5 → 61")).toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox", { name: "Level" }), {
      target: { value: "type" },
    });
    await screen.findByText("Got worse", { selector: "h2" });
    expect(bodiesOf("scans.metrics").at(-1)).toMatchObject({ level: "type" });
  });

  it("sorts a list by a column head and offers a handle for every column", async () => {
    serveChange({ "scans.metrics": () => answer(METRICS) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    const worse = within(await screen.findByRole("region", { name: "Got worse" }));
    const names = () =>
      worse
        .getAllByRole("row")
        .slice(1)
        .map((row) => row.querySelectorAll("td")[2]?.textContent);
    expect(names()).toEqual(["Complexity", "Lines"]);
    fireEvent.click(worse.getByRole("button", { name: "Metric" }));
    expect(names()).toEqual(["Complexity", "Lines"]);
    fireEvent.click(worse.getByRole("button", { name: "Metric" }));
    expect(names()).toEqual(["Lines", "Complexity"]);
    expect(worse.getAllByRole("columnheader")).toHaveLength(5);
  });

  it("greys out the metrics that members do not have, with the reason, and resets the choice", async () => {
    serveChange({ "scans.metrics": () => answer(METRICS) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    const level = await screen.findByRole("combobox", { name: "Level" });
    fireEvent.change(level, { target: { value: "type" } });
    await waitFor(() => expect(screen.getByRole("button", { name: "Coupling out" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "Coupling out" }));
    fireEvent.change(level, { target: { value: "member" } });
    const coupling = await screen.findByRole("button", { name: "Coupling out" });
    expect(coupling).toBeDisabled();
    expect(coupling).toHaveAttribute("title", expect.stringContaining("types and modules"));
    expect(screen.getByRole("button", { name: "Stability" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "All" })).toHaveAttribute("aria-pressed", "true");
  });

  it("says how many of the objects are listed when a list was cut", async () => {
    const cut = { ...METRICS, worse: { ...METRICS.worse, total: 40, is_cut: true } };
    serveChange({ "scans.metrics": () => answer(cut) });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    expect(await screen.findByText("1 of 1 listed (40 in all)")).toBeInTheDocument();
  });

  it("says so when nothing moved by its threshold", async () => {
    const none = { ...METRICS, worse: { total: 0, is_cut: false, items: [] } };
    serveChange({
      "scans.metrics": () => answer({ ...none, better: { total: 0, is_cut: false, items: [] } }),
    });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    expect(await screen.findByText("No object moved by its threshold.")).toBeInTheDocument();
  });

  it("shows the failure of the read with its code", async () => {
    serveChange({
      "scans.metrics": () => answer({ code: "CMP-201", message: "x", retryable: false }, 400),
    });
    render(() => <ChangeScreen />);
    fireEvent.click(await screen.findByRole("tab", { name: "Metrics" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    const copy = screen.getByRole("button", { name: "Copy as Markdown" });
    expect(copy).toBeDisabled();
    expect(copy).toHaveAttribute("title", expect.stringContaining("could not be read"));
  });
});
