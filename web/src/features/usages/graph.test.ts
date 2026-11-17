import { describe, expect, it } from "vitest";
import { COLUMN, GRAPH, TONE } from "./consts";
import { buildGraph } from "./graph";
import { ANSWER, FOCUS, HANDLER, hit, place, PRICE, VIEW } from "../../tests/usages";
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

  it("hangs the callers of a caller around that caller, not around the focus", () => {
    const middle = (c: { y: number; height: number }) => c.y + c.height / 2;
    const view = card(VIEW.qualified_name);
    const page = card("Shop.Ui.Page.Open");
    expect(middle(page)).toBeCloseTo(middle(view), 5);
    expect(page.y).toBeGreaterThanOrEqual(GRAPH.top);
  });

  it("spreads several children and several parents without overlap and keeps their order", () => {
    const a = place(21, "Shop.A.One");
    const b = place(22, "Shop.A.Two");
    const c = place(23, "Shop.A.Three");
    const answer = {
      ...ANSWER,
      incoming: {
        total: 5,
        hits: [
          ...ANSWER.incoming!.hits,
          hit(a, VIEW, { depth: 2 }),
          hit(b, VIEW, { depth: 2 }),
          hit(c, HANDLER, { depth: 2 }),
        ],
      },
    };
    const wide = buildGraph(answer);
    const find = (name: string) => wide.cards.find((x) => x.place.qualified_name === name)!;
    const column = wide.cards.filter((x) => x.x === find("Shop.A.One").x).sort((p, q) => p.y - q.y);
    expect(column).toHaveLength(4);
    for (let i = 1; i < column.length; i += 1) {
      expect(column[i]!.y - column[i - 1]!.y).toBeGreaterThanOrEqual(
        GRAPH.cardHeight + GRAPH.rowGap - 1e-6,
      );
    }
    const upper = find(VIEW.qualified_name).y < find(HANDLER.qualified_name).y;
    const order = column.map((x) => x.place.id);
    expect(order.indexOf(21) < order.indexOf(23)).toBe(upper);
    expect(Math.min(...wide.cards.map((x) => x.y))).toBe(GRAPH.top);
  });

  it("puts a card without a parent in the graph next to the height of the focus", () => {
    const orphan = place(30, "Shop.Lost.Caller");
    const lonely = buildGraph({
      ...ANSWER,
      incoming: {
        total: 5,
        hits: [...ANSWER.incoming!.hits, hit(orphan, place(31, "Shop.Lost.Parent"), { depth: 2 })],
      },
    });
    const card = lonely.cards.find((x) => x.place.id === 30)!;
    const focus = lonely.cards.find((x) => x.focus)!;
    expect(Number.isFinite(card.y)).toBe(true);
    // Another card of the same column wants the same line, so the two share it, side by side.
    const apart = Math.abs(card.y + card.height / 2 - (focus.y + focus.height / 2));
    expect(apart).toBeLessThanOrEqual((GRAPH.cardHeight + GRAPH.rowGap) / 2 + 1e-6);
  });

  it("keeps the column of what is called when nothing is called", () => {
    const empty = buildGraph({ ...ANSWER, outgoing: { ...ANSWER.outgoing!, hits: [], total: 0 } });
    const callees = empty.columns.filter((column) => column.kind === COLUMN.callees);
    expect(callees).toHaveLength(1);
    expect(callees[0]!.x).toBeGreaterThan(empty.columns.find((c) => c.kind === COLUMN.focus)!.x);
    const none = buildGraph({ ...ANSWER, outgoing: null });
    expect(none.columns.some((column) => column.kind === COLUMN.callees)).toBe(false);
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

  it("orders the cards of a column by name, whatever their counts, and keeps that order at a greater depth", () => {
    const alpha = place(201, "Shop.Z.Alpha.Run");
    const beta = place(202, "Shop.Y.Beta.Run");
    const hits = [hit(beta, FOCUS), hit(beta, FOCUS), hit(beta, FOCUS), hit(alpha, FOCUS)];
    const order = (list: typeof hits) => {
      const built = buildGraph({
        ...ANSWER,
        incoming: { total: list.length, hits: list },
        outgoing: null,
      });
      return built.cards
        .filter((c) => !c.focus && c.x === built.cards.find((x) => x.place.id === 201)!.x)
        .sort((a, b) => a.y - b.y)
        .map((c) => c.place.id);
    };
    expect(order(hits)).toEqual([201, 202]);
    const deeper = [...hits, hit(alpha, beta, { depth: 2 }), hit(alpha, beta, { depth: 2 })];
    expect(order(deeper.filter((h) => h.depth === 1))).toEqual([201, 202]);
  });

  it("keeps the cards of a shown level where they are when a deeper level is added", () => {
    const parents = Array.from({ length: GRAPH.maxCards + 2 }, (_, i) =>
      place(300 + i, `Shop.Wide.Parent${String(i).padStart(2, "0")}.Run`),
    );
    const direct = parents.map((parent) => hit(parent, FOCUS));
    const deeper = [
      ...parents
        .slice(0, 3)
        .flatMap((parent, i) =>
          Array.from({ length: 5 + i }, (_, k) =>
            hit(place(400 + i * 10 + k, `Shop.Deep.C${i}${k}.Go`), parent, { depth: 2 }),
          ),
        ),
      ...parents.slice(0, 2).flatMap((parent) =>
        [1, 2, 3].map((n) =>
          hit(place(500 + n, `Shop.Deeper.D${n}.Go`), place(400, "Shop.Deep.C00.Go"), {
            depth: 3,
          }),
        ),
      ),
      ...parents
        .slice(-1)
        .flatMap((parent) => [
          hit(parent, parents[0]!, { depth: 2 }),
          hit(parent, parents[0]!, { depth: 2 }),
        ]),
    ];
    const first = (depth: number) => {
      const hits = [...direct, ...deeper].filter((h) => h.depth <= depth);
      const built = buildGraph({
        ...ANSWER,
        incoming: { total: hits.length, hits },
        outgoing: null,
      });
      return built.cards
        .filter((c) => c.x === built.cards.find((x) => x.place.id === 300)!.x)
        .sort((a, b) => a.y - b.y)
        .map((c) => c.place.id);
    };
    expect(first(2)).toEqual(first(1));
    expect(first(3)).toEqual(first(1));
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
