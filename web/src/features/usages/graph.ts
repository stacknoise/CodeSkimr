import {
  COLUMN,
  CONFIDENCE,
  END,
  FIRST_LEVEL,
  GRAPH,
  JOIN_NONE,
  PAIR_SEPARATOR,
  PATH,
  REF,
  TONE,
  type ColumnKind,
  type End,
  type Tone,
} from "./consts";
import { cardNames, shortName } from "./names";
import type { Hit, Place, UsagesAnswer, Walk } from "./types";

/** A column of the graph with its heading. */
export interface Column {
  kind: ColumnKind;
  /** Steps from the focus, 0 for the focus itself. */
  level: number;
  x: number;
  /** Cards that did not fit. */
  hidden: number;
}

/** A symbol of the graph. */
export interface Card {
  place: Place;
  name: string;
  /** What the card writes big and small: the symbol, and the one it belongs to. */
  member: string;
  owner: string;
  x: number;
  y: number;
  height: number;
  /** Places in which the symbol appears in the walk. */
  count: number;
  focus: boolean;
  /** Whether every hit of the card is a test. */
  test: boolean;
}

/** A line between two cards. */
export interface Edge {
  key: string;
  d: string;
  tone: Tone;
  /** The ids of the symbols the line joins, so that a click on a card can find its lines. */
  from: number;
  to: number;
}

/** What the screen draws. */
export interface Graph {
  columns: Column[];
  cards: Card[];
  edges: Edge[];
  width: number;
  height: number;
}

interface Node {
  place: Place;
  level: number;
  count: number;
  /** How often the symbol was hit at its own step; deeper hits do not change it. */
  near: number;
  tests: number;
}

/** The symbols one side of a walk reached, each at the step it was reached first. `side` is the end
 * of a hit that is not the symbol the walk came from. */
function nodesOf(hits: Hit[], side: End, focus: number): Map<number, Node> {
  const nodes = new Map<number, Node>();
  for (const hit of hits) {
    const place = hit[side];
    if (place.id === focus) {
      continue;
    }
    const node = nodes.get(place.id) ?? { place, level: hit.depth, count: 0, near: 0, tests: 0 };
    if (hit.depth < node.level) {
      node.level = hit.depth;
      node.near = 0;
    }
    node.near += hit.depth === node.level ? 1 : 0;
    node.count += 1;
    node.tests += hit.kind === REF.test ? 1 : 0;
    nodes.set(place.id, node);
  }
  return nodes;
}

/** Which cards stay when a column is cut: the most hit at their own step, so that a deeper walk
 * does not swap the cards of a column that was shown before. */
function byWeight(a: Node, b: Node): number {
  return b.near - a.near || a.place.qualified_name.localeCompare(b.place.qualified_name);
}

/** The order of the cards in a column: by the name the card shows, then by its owner. It does not
 * depend on how often a symbol was hit, so a deeper walk does not shuffle the cards that were
 * shown before. */
function byName(a: Node, b: Node): number {
  const left = cardNames(a.place.qualified_name);
  const right = cardNames(b.place.qualified_name);
  return (
    left.member.localeCompare(right.member) ||
    left.owner.localeCompare(right.owner) ||
    a.place.qualified_name.localeCompare(b.place.qualified_name)
  );
}

/** How an edge looks: tests violet, presumed hits dashed, the rest by direction. */
function toneOf(hits: Hit[], way: Tone): Tone {
  if (hits.some((hit) => hit.kind === REF.test)) {
    return TONE.test;
  }
  return hits.every((hit) => hit.confidence === CONFIDENCE.probable) ? TONE.probable : way;
}

function curve(from: Card, to: Card): string {
  const startX = from.x + GRAPH.columnWidth;
  const startY = from.y + from.height / GRAPH.half;
  const endY = to.y + to.height / GRAPH.half;
  const middle = (startX + to.x) / GRAPH.half;
  return [
    PATH.move,
    startX,
    PATH.space,
    startY,
    PATH.space,
    PATH.curve,
    middle,
    PATH.space,
    startY,
    PATH.space,
    middle,
    PATH.space,
    endY,
    PATH.space,
    to.x,
    PATH.space,
    endY,
  ].join(JOIN_NONE);
}

/** Column index of a level: callers to the left of the focus, callees to the right. */
function columnOf(reach: number, side: ColumnKind, level: number): number {
  return side === COLUMN.callers ? reach - level : reach + level;
}

interface Side {
  kind: ColumnKind;
  nodes: Map<number, Node>;
  hits: Hit[];
  /** The walk of this side was asked for. */
  requested: boolean;
}

function sideOf(kind: ColumnKind, walk: Walk | null, focus: number): Side {
  const end = kind === COLUMN.callers ? END.from : END.to;
  const hits = walk?.hits ?? [];
  return { kind, nodes: nodesOf(hits, end, focus), hits, requested: walk !== null };
}

/** The symbols of one column that the symbol `node` is reached from: the other ends of its hits. */
function parentsOf(node: Node, side: Side): number[] {
  const own = side.kind === COLUMN.callers ? END.from : END.to;
  const other = side.kind === COLUMN.callers ? END.to : END.from;
  return side.hits.filter((hit) => hit[own].id === node.place.id).map((hit) => hit[other].id);
}

/** Whether a parent of the symbol has a card: a card without one would hang in the air, with no
 * line to the column nearer to the focus. */
function hasParent(node: Node, side: Side, cards: Map<string, Card>, focus: Card): boolean {
  return parentsOf(node, side).some(
    (id) => id === focus.place.id || cards.has(`${side.kind}${id}`),
  );
}

/** The middle lines the cards of a column want: the average middle of their parents in the column
 * nearer to the focus, so that children hang around their parent and not around the focus. */
function wishes(nodes: Node[], side: Side, cards: Map<string, Card>, focus: Card): number[] {
  return nodes.map((node) => {
    const middles = parentsOf(node, side)
      .map((id) => (id === focus.place.id ? focus : cards.get(`${side.kind}${id}`)))
      .filter((card): card is Card => card !== undefined)
      .map(middleOf);
    return middles.length === 0
      ? middleOf(focus)
      : middles.reduce((sum, middle) => sum + middle, 0) / middles.length;
  });
}

/** Middle line of a card. */
function middleOf(card: Card): number {
  return card.y + card.height / GRAPH.half;
}

/** A node with the middle line its card wants. */
interface Wish {
  node: Node;
  middle: number;
}

/** A run of neighbouring cards that move together: the sum of what they want and how many. */
interface Run {
  sum: number;
  size: number;
}

/** The nearest placement of the wished middle lines (in ascending order) in which no two cards
 * touch. A card is only moved as far as the cards around it force it to, so children stay at
 * their parent. Cards at the distance of one step from each other lie on a line; shifting by the
 * index turns the rule "at least one step apart" into "not decreasing", which a merge of
 * neighbouring runs by their mean solves with the least movement in total. */
function spread(wished: Wish[]): Wish[] {
  const step = GRAPH.cardHeight + GRAPH.rowGap;
  const runs: Run[] = [];
  wished.forEach((wish, index) => {
    runs.push({ sum: wish.middle - index * step, size: 1 });
    for (let last = runs.at(-1), before = runs.at(-2); last && before; before = runs.at(-2)) {
      if (before.sum / before.size <= last.sum / last.size) {
        break;
      }
      runs.splice(-2, 2, { sum: before.sum + last.sum, size: before.size + last.size });
      last = runs.at(-1);
    }
  });
  const base = runs.flatMap((run) => Array.from({ length: run.size }, () => run.sum / run.size));
  return wished.map((wish, index) => ({
    node: wish.node,
    middle: (base[index] ?? 0) + index * step,
  }));
}

function stack(
  side: Side,
  reach: number,
  cards: Map<string, Card>,
  columns: Column[],
  focus: Card,
): void {
  const levels = [...new Set([...side.nodes.values()].map((node) => node.level))].sort(
    (a, b) => a - b,
  );
  for (const level of levels) {
    const onLevel = [...side.nodes.values()].filter((node) => node.level === level);
    const all = onLevel.filter((node) => hasParent(node, side, cards, focus)).sort(byWeight);
    const index = columnOf(reach, side.kind, level);
    const x = GRAPH.left + index * (GRAPH.columnWidth + GRAPH.columnGap);
    const kept = all.slice(0, GRAPH.maxCards).sort(byName);
    columns.push({ kind: side.kind, level, x, hidden: onLevel.length - kept.length });
    const middles = wishes(kept, side, cards, focus);
    const wished = kept.map((node, i) => ({ node, middle: middles[i] ?? middleOf(focus) }));
    for (const { node, middle } of spread(wished.sort((a, b) => a.middle - b.middle))) {
      cards.set(`${side.kind}${node.place.id}`, {
        place: node.place,
        name: shortName(node.place.qualified_name),
        ...cardNames(node.place.qualified_name),
        x,
        y: middle - GRAPH.cardHeight / GRAPH.half,
        height: GRAPH.cardHeight,
        count: node.count,
        focus: false,
        test: node.tests === node.count,
      });
    }
  }
}

/** The references between one pair of symbols. */
interface Pair {
  from: Place;
  to: Place;
  hits: Hit[];
}

function pairsOf(hits: Hit[]): Map<string, Pair> {
  const pairs = new Map<string, Pair>();
  for (const hit of hits) {
    const key = [hit.from.id, hit.to.id].join(PAIR_SEPARATOR);
    const pair = pairs.get(key) ?? { from: hit.from, to: hit.to, hits: [] };
    pair.hits.push(hit);
    pairs.set(key, pair);
  }
  return pairs;
}

function edgesOf(side: Side, cards: Map<string, Card>, focus: Card): Edge[] {
  const way = side.kind === COLUMN.callers ? TONE.incoming : TONE.outgoing;
  const cardOf = (place: Place) =>
    place.id === focus.place.id ? focus : cards.get(`${side.kind}${place.id}`);
  const edges: Edge[] = [];
  for (const [key, pair] of pairsOf(side.hits)) {
    const from = cardOf(pair.from);
    const to = cardOf(pair.to);
    if (from && to && from.x < to.x) {
      edges.push({
        key: `${side.kind}${key}`,
        d: curve(from, to),
        tone: toneOf(pair.hits, way),
        from: pair.from.id,
        to: pair.to.id,
      });
    }
  }
  return edges;
}

/** Keeps the column "calls / reads" even when nothing is called, as the mock does. */
function keepCalleesColumn(columns: Column[], callees: Side, reach: number): void {
  if (callees.hits.length > 0 || !callees.requested) {
    return;
  }
  const index = columnOf(reach, COLUMN.callees, FIRST_LEVEL);
  const x = GRAPH.left + index * (GRAPH.columnWidth + GRAPH.columnGap);
  columns.push({ kind: COLUMN.callees, level: FIRST_LEVEL, x, hidden: 0 });
}

/** The card of the symbol in focus: taller than the others, its middle on the line 0. */
function focusCard(place: Place, x: number): Card {
  return {
    place,
    name: shortName(place.qualified_name),
    ...cardNames(place.qualified_name),
    x,
    y: -GRAPH.focusHeight / GRAPH.half,
    height: GRAPH.focusHeight,
    count: 0,
    focus: true,
    test: false,
  };
}

/** Moves all cards so that the top one sits at the top margin; returns the bottom of the lowest. */
function alignTop(cards: Card[]): number {
  const top = Math.min(...cards.map((card) => card.y));
  cards.forEach((card) => {
    card.y += GRAPH.top - top;
  });
  return Math.max(...cards.map((card) => card.y + card.height));
}

/** Lays the walks of an answer out as columns of cards and the lines between them (mock 6). Each
 * card hangs around the middle of its parents, the focus in the middle of its column. */
export function buildGraph(answer: UsagesAnswer): Graph {
  const focusId = answer.symbol.id;
  const callers = sideOf(COLUMN.callers, answer.incoming, focusId);
  const callees = sideOf(COLUMN.callees, answer.outgoing, focusId);
  const reach = Math.max(0, ...[...callers.nodes.values()].map((node) => node.level));
  const focusX = GRAPH.left + reach * (GRAPH.columnWidth + GRAPH.columnGap);
  const focus = focusCard(answer.symbol, focusX);
  const cards = new Map<string, Card>();
  const columns: Column[] = [{ kind: COLUMN.focus, level: 0, x: focusX, hidden: 0 }];
  stack(callers, reach, cards, columns, focus);
  stack(callees, reach, cards, columns, focus);
  const all = [focus, ...cards.values()];
  const bottom = alignTop(all);
  const last = Math.max(
    answer.outgoing === null ? 0 : FIRST_LEVEL,
    ...[...callees.nodes.values()].map((node) => node.level),
  );
  keepCalleesColumn(columns, callees, reach);
  return {
    columns,
    cards: all,
    edges: [...edgesOf(callers, cards, focus), ...edgesOf(callees, cards, focus)],
    width: GRAPH.left + GRAPH.left + (reach + last + 1) * (GRAPH.columnWidth + GRAPH.columnGap),
    height: bottom + GRAPH.bottom,
  };
}
