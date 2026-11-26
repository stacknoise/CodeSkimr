import type { Preview, Site } from "../features/rename/types";

export const USAGE: Site = {
  kind: "usage",
  path: "Api/Orders.cs",
  line: 41,
  column: 9,
  confidence: "certain",
  verified: true,
  from: null,
  layer: null,
};

export const PREVIEW: Preview = {
  snapshot: 3,
  symbol: { id: 7, qualified_name: "Shop.Orders.PlaceOrder", kind: "method", layer: null },
  old_name: "PlaceOrder",
  new_name: "SubmitOrder",
  public_interface: true,
  preliminary: true,
  sites_total: 2,
  verified: 1,
  files: 2,
  layers: [{ layer: "Api", count: 1 }],
  sites: [
    { ...USAGE, kind: "declaration", path: "Orders/IOrders.cs", line: 12, verified: true },
    { ...USAGE, verified: false, confidence: "probable" },
  ],
  conflicts: [],
  text_searched: true,
  text_total: 1,
  text_skipped: 0,
  text_hits: [{ path: "docs/api.md", line: 3, text: "calls PlaceOrder", kind: "documentation" }],
};
