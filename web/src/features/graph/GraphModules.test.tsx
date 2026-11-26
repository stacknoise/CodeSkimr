import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { LEVEL } from "./consts";
import { resetGraph, setGraphLevel } from "./graph";
import { GraphScreen } from "./GraphScreen";
import { stabilityText } from "./ModulesPanel";

const ROW: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: null,
  types: 3,
  loc: 10,
  languages: [],
  running: false,
};

const BOX = { width: 160, height: 44, order: 0, types: 5, symbol: null, context: "" };
const MODULES = {
  snapshot: 5,
  width: 400,
  height: 200,
  boxes: [
    { name: "Shop.Orders", label: "Orders", x: 0, y: 0, rank: 0, layer: "app", ...BOX },
    { name: "Shop.Checkout", label: "Checkout", x: 200, y: 0, rank: 0, layer: "app", ...BOX },
  ],
  bands: [{ name: "app", rank: 0, y: 0, height: 60 }],
  links: [
    {
      from: "Shop.Orders",
      to: "Shop.Checkout",
      count: 11,
      violates_layering: false,
      in_cycle: true,
    },
    {
      from: "Shop.Checkout",
      to: "Shop.Orders",
      count: 2,
      violates_layering: false,
      in_cycle: true,
    },
  ],
  modules: [
    {
      name: "Shop.Orders",
      layer: "app",
      incoming: 16,
      outgoing: 54,
      stability: 771,
      types: 5,
      in_cycle: true,
      violations: 0,
    },
    {
      name: "Shop.Checkout",
      layer: "app",
      incoming: 17,
      outgoing: 20,
      stability: 540,
      types: 5,
      in_cycle: true,
      violations: 1,
    },
  ],
  modules_more: 0,
  cycles: [
    {
      modules: ["Shop.Checkout", "Shop.Orders"],
      edges: [
        { from: "Shop.Orders", to: "Shop.Checkout", count: 11 },
        { from: "Shop.Checkout", to: "Shop.Orders", count: 2 },
      ],
      resolution: [{ from: "Shop.Checkout", to: "Shop.Orders", count: 2 }],
    },
  ],
  violations: [
    {
      edge: { from: "Shop.Api", to: "Shop.Data", count: 1 },
      from_layer: "api",
      to_layer: "data",
    },
  ],
  risks: [
    {
      module: "Shop.Checkout",
      incoming: 17,
      stability: 540,
      depends_on: "Shop.Orders",
      depends_on_stability: 771,
    },
  ],
};

useServerSession();

beforeEach(() => {
  resetGraph();
  setGraphLevel(LEVEL.modules);
  openProject(1);
  setLanguage("en");
});

describe("the level Modules", () => {
  it("shows the table, the cycles with their resolution and the risks", async () => {
    serve({ "projects.list": () => answer([ROW]), "graph.modules": () => answer(MODULES) });
    render(() => <GraphScreen />);
    const panel = await screen.findByRole("complementary", { name: "Modules" });
    expect(within(panel).getByText("0.77")).toBeInTheDocument();
    expect(within(panel).getAllByText("in a cycle")).toHaveLength(2);
    expect(within(panel).getByText("1 violation")).toBeInTheDocument();
    expect(
      within(panel).getByText(/Ending the cycle takes away the weakest edge/),
    ).toHaveTextContent("Shop.Checkout → Shop.Orders (2 places)");
    expect(within(panel).getByText(/Shop.Api \(api\) → Shop.Data \(data\)/)).toBeInTheDocument();
    expect(within(panel).getByRole("note", { name: "" })).toHaveTextContent(
      "Shop.Checkout (stability 0.54) is used in 17 places",
    );
  });

  it("says when there is no cycle and marks a click on a row as the selection", async () => {
    serve({
      "projects.list": () => answer([ROW]),
      "graph.modules": () => answer({ ...MODULES, cycles: [], violations: [], risks: [] }),
    });
    render(() => <GraphScreen />);
    expect(await screen.findByText("No cycles between modules.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Shop.Orders" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Shop.Orders" }).closest("tr")).toHaveClass(/on/),
    );
  });

  it("writes the stability with the digits of the language", () => {
    expect(stabilityText(630, "en")).toBe("0.63");
    expect(stabilityText(630, "de")).toBe("0,63");
    expect(stabilityText(1000, "en")).toBe("1.00");
  });
});
