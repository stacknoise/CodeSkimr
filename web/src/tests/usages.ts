import type { Hit, Place, UsagesAnswer } from "../features/usages/types";

export const place = (
  id: number,
  name: string,
  kind = "method",
  layer: string | null = "App",
): Place => ({
  id,
  qualified_name: name,
  kind,
  layer,
});

export const FOCUS = place(10, "Shop.Cart.Total", "method", "Domain");
export const VIEW = place(11, "Shop.Ui.CartView.Render");
export const HANDLER = place(12, "Shop.App.CartHandler.Handle");
export const TEST = place(13, "Shop.Tests.CartTests.Totals", "method", null);
export const PRICE = place(14, "Shop.Domain.Price.Value", "property", "Domain");

export const hit = (from: Place, to: Place, extra: Partial<Hit> = {}): Hit => ({
  depth: 1,
  kind: "call",
  confidence: "certain",
  verified: false,
  path: "src/Cart.cs",
  line: 12,
  from,
  to,
  ...extra,
});

/** The usages of `Shop.Cart.Total`: two callers (one through a test), one caller of a caller that
 * is only presumed, and one symbol it reads. */
export const ANSWER: UsagesAnswer = {
  snapshot: 5,
  symbol: FOCUS,
  signature: "decimal Total()",
  path: "src/Cart.cs",
  depth: 2,
  incoming: {
    total: 4,
    hits: [
      hit(VIEW, FOCUS, { line: 40, path: "src/Ui.cs" }),
      hit(HANDLER, FOCUS, { confidence: "probable", line: 7, path: "src/Handler.cs" }),
      hit(TEST, FOCUS, { kind: "test", line: 22, path: "tests/CartTests.cs" }),
      hit(place(15, "Shop.Ui.Page.Open"), VIEW, { depth: 2, line: 3, path: "src/Page.cs" }),
    ],
  },
  outgoing: {
    total: 1,
    hits: [hit(FOCUS, PRICE, { kind: "read", line: 14 })],
  },
  summary: { places: 3, files: 3, layers: 2, verified: 0, unchecked: 3, probable: 1 },
  impact: {
    total: 5,
    public_interfaces: 2,
    verified: 0,
    unchecked: 5,
    layers: [
      { layer: "Domain", count: 2 },
      { layer: "Ui", count: 3 },
    ],
  },
  level2: "unavailable",
};
