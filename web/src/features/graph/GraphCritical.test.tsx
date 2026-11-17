import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { openProject } from "../../shared/project/active";
import { page, SUMMARY } from "../../tests/annotations";
import { answer, serve, useServerSession } from "../../tests/commands";
import { setHighlightCritical, setOnlyCriticalAndNeighbours } from "../annotations/flags";
import { forgetAnnotations } from "../annotations/store";
import { LEVEL } from "./consts";
import { resetGraph, setGraphFocus, setGraphLevel } from "./graph";
import { GraphScreen } from "./GraphScreen";

const SIZE = { width: 160, height: 44, order: 0, layer: null, types: null, symbol: 1, rank: 0 };
const box = (name: string, label: string, x: number) => ({
  name,
  label,
  context: "Shop › Domain",
  x,
  y: 20,
  ...SIZE,
});
const link = (from: string, to: string) => ({
  from,
  to,
  count: 1,
  violates_layering: false,
  in_cycle: false,
});

/** The map of the type `Order`: it uses `Tax`, `Price` uses it, and `Far` is two steps away. */
const TYPES = {
  snapshot: 5,
  focus: { symbol: 1, name: "Shop.Domain.Order", users_more: 0, used_more: 0 },
  width: 700,
  height: 120,
  boxes: [
    box("Shop.Domain.Order", "Order", 0),
    box("Shop.Domain.Tax", "Tax", 180),
    box("Shop.Domain.Price", "Price", 360),
    box("Shop.Domain.Far", "Far", 540),
  ],
  bands: [],
  links: [
    link("Shop.Domain.Order", "Shop.Domain.Tax"),
    link("Shop.Domain.Price", "Shop.Domain.Order"),
    link("Shop.Domain.Tax", "Shop.Domain.Far"),
  ],
};

useServerSession();

/** Opens the screen and moves its map to the type level, as the button "Type focus" does. */
async function renderTypes() {
  render(() => <GraphScreen />);
  await screen.findByText("all connections weak");
  setGraphFocus(1);
  setGraphLevel(LEVEL.types);
}

beforeEach(() => {
  resetGraph();
  openProject(1);
  setHighlightCritical(true);
  setOnlyCriticalAndNeighbours(false);
  serve({
    "projects.list": () => answer([]),
    "graph.layers": () => answer(TYPES),
    "annotations.summary": () => answer(page(SUMMARY)),
  });
});
afterEach(() => {
  forgetAnnotations();
  setGraphLevel(LEVEL.layers);
});

const boxOf = (label: string) => screen.getByText(label, { selector: "span" }).closest("button")!;

describe("the dependency map with critical symbols", () => {
  it("draws a critical type with the stripe and the label, never by colour alone", async () => {
    await renderTypes();
    const order = await waitFor(() => {
      const found = boxOf("Order");
      expect(within(found).getByText("Critical")).toBeTruthy();
      return found;
    });
    expect(order.className).toMatch(/critical/);
    expect(within(boxOf("Tax")).queryByText("Critical")).toBeNull();
  });

  it("stops marking when the switch Highlight critical is off", async () => {
    await renderTypes();
    await waitFor(() => expect(within(boxOf("Order")).getByText("Critical")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Highlight critical"));
    await waitFor(() => expect(within(boxOf("Order")).queryByText("Critical")).toBeNull());
    expect(boxOf("Order").className).not.toMatch(/critical/);
  });

  it("shows only the critical symbols and their neighbours when asked", async () => {
    await renderTypes();
    await waitFor(() => expect(within(boxOf("Order")).getByText("Critical")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("only critical and their neighbours"));
    await waitFor(() => expect(screen.queryByText("Far", { selector: "span" })).toBeNull());
    expect(boxOf("Tax")).toBeTruthy();
    expect(boxOf("Price")).toBeTruthy();
  });

  it("has the switches on the type level only, not on the layer level", async () => {
    render(() => <GraphScreen />);
    await screen.findByText("all connections weak");
    expect(screen.queryByLabelText("Highlight critical")).toBeNull();
  });

  it("keeps the map and has no switches on a server without annotations", async () => {
    serve({ "projects.list": () => answer([]), "graph.layers": () => answer(TYPES) });
    await renderTypes();
    await screen.findByText("Order", { selector: "span" });
    await waitFor(() => expect(screen.queryByLabelText("Highlight critical")).toBeNull());
  });
});
