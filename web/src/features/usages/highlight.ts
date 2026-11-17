import type { Edge, Graph } from "./graph";

/** What a click on a card lights up: the lines that start or end at the card, and the cards they
 * join. Everything else is dimmed, as in the maps of screen 3. Without a picked card nothing is
 * dimmed. */
export interface Highlight {
  active: boolean;
  /** Whether `edge` touches the picked card. */
  lit: (edge: Edge) => boolean;
  /** Whether the card with this symbol id is the picked one or a neighbour of it. */
  related: (id: number) => boolean;
}

/** The highlight of `picked` (a symbol id, or null) in `graph`. */
export function highlightOf(graph: Graph, picked: number | null): Highlight {
  const present = picked !== null && graph.cards.some((card) => card.place.id === picked);
  const lit = (edge: Edge) => edge.from === picked || edge.to === picked;
  const ids = new Set<number>();
  if (present) {
    ids.add(picked);
    for (const edge of graph.edges.filter(lit)) {
      ids.add(edge.from);
      ids.add(edge.to);
    }
  }
  return { active: present, lit, related: (id) => ids.has(id) };
}
