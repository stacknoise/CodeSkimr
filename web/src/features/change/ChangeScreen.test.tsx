import { fireEvent, render, screen, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { compareWithCurrent, resetChange, showChangeTab } from "./change";
import { ChangeScreen } from "./ChangeScreen";
import { TAB } from "./consts";
import type { ChangeAnswer, SymbolOut, UsageOut } from "./types";

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
    expect(panel.getByText("− decimal Round(decimal value)")).toBeInTheDocument();
    expect(
      panel.getByText("+ decimal Round(decimal value, RoundingMode mode)"),
    ).toBeInTheDocument();
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
