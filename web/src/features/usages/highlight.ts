import type { Edge, Graph } from "./graph";

/** What a click on a card lights up: every line on a path that runs through the card, over all
 * depths shown, and the cards on those paths. A path ends at the focus: a card on one side is not
 * connected to the other side, except through the focus itself, which is no use of it. The focus
 * lights both sides. Everything else is dimmed, as in the maps of screen 3. Without a picked card
 * nothing is dimmed. */
export interface Highlight {
  active: boolean;
  /** Whether `edge` lies on a path through the picked card. */
  lit: (edge: Edge) => boolean;
  /** Whether the card with this symbol id is the picked one or on a path through it. */
  related: (id: number) => boolean;
}

/** What a walk from one card reaches along the lines, in one direction. */
interface Reached {
  ids: Set<number>;
  edges: Set<Edge>;
}

/** The two ends of `edge` in the direction of the walk: where it is left, and where it leads. */
function ends(edge: Edge, forward: boolean): [number, number] {
  return forward ? [edge.from, edge.to] : [edge.to, edge.from];
}

/** Follows the edges from `start`, one way: `from` to `to`, or back. The walk enters `stop` but
 * does not go on from it, unless it starts there. */
function reach(edges: Edge[], start: number, forward: boolean, stop: number | null): Reached {
  const ids = new Set<number>([start]);
  const lit = new Set<Edge>();
  const queue = [start];
  for (let id = queue.shift(); id !== undefined; id = queue.shift()) {
    for (const edge of edges) {
      const [here, next] = ends(edge, forward);
      if (here !== id) {
        continue;
      }
      lit.add(edge);
      if (!ids.has(next)) {
        ids.add(next);
        if (next !== stop) {
          queue.push(next);
        }
      }
    }
  }
  return { ids, edges: lit };
}

/** The highlight of `picked` (a symbol id, or null) in `graph`. */
export function highlightOf(graph: Graph, picked: number | null): Highlight {
  const present = picked !== null && graph.cards.some((card) => card.place.id === picked);
  if (!present) {
    return { active: false, lit: () => false, related: () => false };
  }
  const focus = graph.cards.find((card) => card.focus)?.place.id ?? null;
  const stop = picked === focus ? null : focus;
  const down = reach(graph.edges, picked, true, stop);
  const up = reach(graph.edges, picked, false, stop);
  return {
    active: true,
    lit: (edge) => down.edges.has(edge) || up.edges.has(edge),
    related: (id) => down.ids.has(id) || up.ids.has(id),
  };
}
