import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, serve, useServerSession } from "../../tests/commands";
import { PROJECT_ROW as ROW } from "../../tests/project-row";
import { LEVEL } from "./consts";
import { resetGraph, setGraphLevel } from "./graph";
import { GraphScreen } from "./GraphScreen";
import { stabilityText } from "./ModulesPanel";

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

  it("explains in, out and stability as tooltips of the column heads", async () => {
    serve({ "projects.list": () => answer([ROW]), "graph.modules": () => answer(MODULES) });
    render(() => <GraphScreen />);
    const panel = await screen.findByRole("complementary", { name: "Modules" });
    const head = (name: RegExp) => within(panel).getByRole("columnheader", { name });
    expect(head(/^in/)).toHaveAttribute("title", expect.stringContaining("use this module"));
    expect(head(/^out/)).toHaveAttribute("title", expect.stringContaining("use other modules"));
    expect(head(/^Stability/)).toHaveAttribute(
      "title",
      expect.stringContaining("out ÷ (in + out)"),
    );
  });

  it("opens the places of the edge that ends a cycle, with file, line and both symbols", async () => {
    const places = {
      snapshot: 5,
      total: 3,
      places: [
        {
          path: "src/Checkout.cs",
          line: 12,
          from: "Shop.Checkout.Cart.Add",
          to: "Shop.Orders.Order.Id",
        },
        {
          path: "src/Checkout.cs",
          line: 40,
          from: "Shop.Checkout.Cart.Pay",
          to: "Shop.Orders.Order.Total",
        },
      ],
    };
    serve({
      "projects.list": () => answer([ROW]),
      "graph.modules": () => answer(MODULES),
      "graph.module_places": () => answer(places),
    });
    render(() => <GraphScreen />);
    const panel = await screen.findByRole("complementary", { name: "Modules" });
    const button = within(panel).getByRole("button", { name: "Show the 2 places" });
    expect(button).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(button);
    expect(await within(panel).findByText("src/Checkout.cs:12")).toBeInTheDocument();
    expect(
      within(panel).getByText("Shop.Checkout.Cart.Pay → Shop.Orders.Order.Total"),
    ).toBeInTheDocument();
    expect(within(panel).getByText("1 more place is not shown.")).toBeInTheDocument();
    fireEvent.click(within(panel).getByRole("button", { name: "Hide the places" }));
    expect(within(panel).queryByText("src/Checkout.cs:12")).not.toBeInTheDocument();
  });

  it("says that the places could not be loaded", async () => {
    serve({
      "projects.list": () => answer([ROW]),
      "graph.modules": () => answer(MODULES),
      "graph.module_places": () => answer({}, 500),
    });
    render(() => <GraphScreen />);
    const panel = await screen.findByRole("complementary", { name: "Modules" });
    fireEvent.click(within(panel).getByRole("button", { name: "Show the 2 places" }));
    expect(await within(panel).findByRole("alert")).toHaveTextContent(
      "The places could not be loaded.",
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
