import { describe, expect, it } from "vitest";
import { buildGraph } from "./graph";
import { highlightOf } from "./highlight";
import { ANSWER, VIEW } from "../../tests/usages";

describe("highlightOf", () => {
  const graph = buildGraph(ANSWER);

  it("dims nothing while no card is picked", () => {
    expect(highlightOf(graph, null).active).toBe(false);
  });

  it("lights the lines of the picked card and the cards they join, and dims the rest", () => {
    const highlight = highlightOf(graph, VIEW.id);
    const lit = graph.edges.filter(highlight.lit);
    expect(highlight.active).toBe(true);
    expect(lit.length).toBeGreaterThan(0);
    expect(lit.length).toBeLessThan(graph.edges.length);
    expect(highlight.related(VIEW.id)).toBe(true);
    for (const edge of lit) {
      expect(highlight.related(edge.from) && highlight.related(edge.to)).toBe(true);
    }
    expect(graph.cards.some((card) => !highlight.related(card.place.id))).toBe(true);
  });

  it("ignores a symbol that is not in the graph", () => {
    expect(highlightOf(graph, -1).active).toBe(false);
  });
});
