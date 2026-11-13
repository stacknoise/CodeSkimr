import { fireEvent, screen } from "@solidjs/testing-library";
import type { Outline, SymbolInfo } from "../features/explorer/types";

/** The tree and the type that the explorer tests let the server answer with. */
export const OUTLINE: Outline = {
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

export const ORDER: SymbolInfo = {
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
  layer: null,
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

/** Opens layer, namespace and the type `Order` by clicking. */
export async function openOrder() {
  fireEvent.click(await screen.findByText("Domain"));
  fireEvent.click(await screen.findByText("Shop.Domain"));
  fireEvent.click(await screen.findByText("Order"));
}
