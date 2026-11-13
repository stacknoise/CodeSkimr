import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { ExplorerScreen } from "./ExplorerScreen";
import type { Outline, SymbolInfo } from "./types";

const OUTLINE: Outline = {
  snapshot: 5,
  thresholds: {
    complexity_good_max: 5,
    complexity_medium_max: 10,
    maintainability_good_min: 65,
    maintainability_medium_min: 35,
    instability_stable_max: 0.3,
    instability_medium_max: 0.7,
  },
  layers: [
    {
      name: "Domain",
      namespaces: [
        { name: "Shop.Domain", types: [{ id: 7, name: "Order", kind: "class", members: 2 }] },
      ],
    },
  ],
};

const ORDER: SymbolInfo = {
  snapshot: 5,
  id: 7,
  name: "Order",
  qualified_name: "Shop.Domain.Order",
  kind: "class",
  visibility: "public",
  is_static: false,
  path: "src/Order.cs",
  first_line: 3,
  last_line: 40,
  signature: null,
  doc: "An order of a customer.",
  breadcrumb: [{ id: 6, name: "Shop.Domain", kind: "namespace" }],
  metrics: [
    { kind: "loc", value: 188, rating: null },
    { kind: "mi", value: 25, rating: "poor" },
  ],
  members: [
    {
      id: 8,
      name: "Total",
      kind: "method",
      visibility: "public",
      signature: "decimal Total()",
      complexity: 12,
      complexity_rating: "poor",
      usages: 4,
      first_line: 10,
      last_line: 20,
    },
    {
      id: 9,
      name: "Id",
      kind: "property",
      visibility: "public",
      signature: "int",
      complexity: null,
      complexity_rating: null,
      usages: 1,
      first_line: 5,
      last_line: 5,
    },
  ],
};

useServerSession();

/** Renders the screen for a project whose tree and type the server answers. */
function renderOrder() {
  openProject(1);
  serve({ "explorer.outline": () => answer(OUTLINE), "explorer.symbol": () => answer(ORDER) });
  render(() => <ExplorerScreen />);
}

async function openOrder() {
  fireEvent.click(await screen.findByText("Domain"));
  fireEvent.click(await screen.findByText("Shop.Domain"));
  fireEvent.click(await screen.findByText("Order"));
}

describe("ExplorerScreen", () => {
  it("asks to open a project first", async () => {
    render(() => <ExplorerScreen />);
    expect(await screen.findByText("No project open")).toBeTruthy();
  });

  it("shows the structure, opens a type and shows its metrics and members", async () => {
    renderOrder();
    await openOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    expect(screen.getByText("An order of a customer.")).toBeTruthy();
    expect(screen.getByText("188")).toBeTruthy();
    expect(screen.getByText("decimal Total()")).toBeTruthy();
    expect(screen.getByText("Members 2")).toBeTruthy();
    expect(await screen.findByText("Fields etc.")).toBeTruthy();
  });

  it("filters the member table by chip", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.click(screen.getByRole("button", { name: "Properties" }));
    await waitFor(() => expect(screen.queryByText("decimal Total()")).toBeNull());
    expect(screen.getAllByText("int").length).toBeGreaterThan(0);
  });

  it("says that a project has not been scanned yet", async () => {
    openProject(2);
    serve({ "explorer.outline": () => answer({ snapshot: null, layers: [] }) });
    render(() => <ExplorerScreen />);
    expect(await screen.findByText(/not been scanned/)).toBeTruthy();
  });

  it("shows the message of a failed read", async () => {
    openProject(3);
    serve({ "explorer.outline": () => answer({ code: "PRJ-201" }, 404) });
    render(() => <ExplorerScreen />);
    await waitFor(() => expect(document.querySelector("[class*=message]")).not.toBeNull());
  });
});
