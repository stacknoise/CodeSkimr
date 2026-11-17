import type { Edge, Graph } from "./graph";

/** What a click on a card lights up: every line on a path that runs through the card, over all
 * depths shown, and the cards on those paths. Everything else is dimmed, as in the maps of
 * screen 3. Without a picked card nothing is dimmed. */
export interface Highlight {
  active: boolean;
  /** Whether `edge` lies on a path through the picked card. */
  lit: (edge: Edge) => boolean;
  /** Whether the card with this symbol id is the picked one or on a path through it. */
  related: (id: number) => boolean;
}

/** The ids reached from `start` by following the edges one way: `from` to `to`, or back. */
function reach(edges: Edge[], start: number, forward: boolean): Set<number> {
  const seen = new Set<number>([start]);
  const queue = [start];
  for (let id = queue.shift(); id !== undefined; id = queue.shift()) {
    for (const edge of edges) {
      const [here, next] = forward ? [edge.from, edge.to] : [edge.to, edge.from];
      if (here === id && !seen.has(next)) {
        seen.add(next);
        queue.push(next);
      }
    }
  }
  return seen;
}

/** The highlight of `picked` (a symbol id, or null) in `graph`. */
export function highlightOf(graph: Graph, picked: number | null): Highlight {
  const present = picked !== null && graph.cards.some((card) => card.place.id === picked);
  if (!present) {
    return { active: false, lit: () => false, related: () => false };
  }
  const down = reach(graph.edges, picked, true);
  const up = reach(graph.edges, picked, false);
  return {
    active: true,
    lit: (edge) => down.has(edge.from) || up.has(edge.to),
    related: (id) => down.has(id) || up.has(id),
  };
}
