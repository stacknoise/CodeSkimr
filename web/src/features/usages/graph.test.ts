import { describe, expect, it } from "vitest";
import { GRAPH, TONE } from "./consts";
import { buildGraph } from "./graph";
import { ANSWER, FOCUS, hit, place, PRICE, VIEW } from "../../tests/usages";
import { nextHit } from "./keys";
import { shortName } from "./names";

describe("buildGraph", () => {
  const graph = buildGraph(ANSWER);
  const card = (name: string) => graph.cards.find((c) => c.place.qualified_name === name)!;

  it("puts callers left of the focus by step and the callees to the right", () => {
    const page = card("Shop.Ui.Page.Open");
    const view = card(VIEW.qualified_name);
    const focus = card(FOCUS.qualified_name);
    const price = card(PRICE.qualified_name);
    expect(page.x).toBeLessThan(view.x);
    expect(view.x).toBeLessThan(focus.x);
    expect(focus.x).toBeLessThan(price.x);
    expect(focus.focus).toBe(true);
    expect(focus.height).toBe(GRAPH.focusHeight);
  });

  it("counts the places of a card and marks a card that only holds tests", () => {
    expect(card(VIEW.qualified_name).count).toBe(1);
    expect(card("Shop.Tests.CartTests.Totals").test).toBe(true);
    expect(card(VIEW.qualified_name).test).toBe(false);
  });

  it("draws test hits violet, presumed hits dashed and the rest by direction", () => {
    const tones = graph.edges.map((edge) => edge.tone).sort();
    expect(tones).toEqual(
      [TONE.incoming, TONE.incoming, TONE.outgoing, TONE.probable, TONE.test].sort(),
    );
  });

  it("names a symbol by its last two parts", () => {
    expect(shortName("Shop.Cart.Total")).toBe("Cart.Total");
    expect(shortName("shop::cart::total")).toBe("cart.total");
    expect(shortName("Total")).toBe("Total");
    expect(card(FOCUS.qualified_name).name).toBe("Cart.Total");
  });

  it("shows only the focus when nothing uses the symbol", () => {
    const empty = buildGraph({ ...ANSWER, incoming: { total: 0, hits: [] }, outgoing: null });
    expect(empty.cards).toHaveLength(1);
    expect(empty.edges).toHaveLength(0);
  });

  it("cuts a column after the most cards and counts the rest", () => {
    const callers = Array.from({ length: GRAPH.maxCards + 3 }, (_, index) =>
      hit(place(100 + index, `Shop.Many.Caller${index}.Run`), FOCUS),
    );
    const crowded = buildGraph({
      ...ANSWER,
      incoming: { total: callers.length, hits: callers },
      outgoing: null,
    });
    expect(crowded.cards).toHaveLength(GRAPH.maxCards + 1);
    expect(crowded.columns.find((column) => column.hidden > 0)?.hidden).toBe(3);
  });

  it("does not draw the reference that closes a cycle backwards", () => {
    const loop = buildGraph({
      ...ANSWER,
      incoming: { total: 2, hits: [hit(VIEW, FOCUS), hit(FOCUS, VIEW, { depth: 2 })] },
      outgoing: null,
    });
    expect(loop.edges).toHaveLength(1);
  });
});

describe("nextHit", () => {
  it("starts at the first hit going forward and at the last one going back", () => {
    expect(nextHit(-1, 1, 3)).toBe(0);
    expect(nextHit(-1, -1, 3)).toBe(2);
  });

  it("wraps around and stays off without hits", () => {
    expect(nextHit(2, 1, 3)).toBe(0);
    expect(nextHit(0, -1, 3)).toBe(2);
    expect(nextHit(-1, 1, 0)).toBe(-1);
  });
});
