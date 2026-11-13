import {
  COLUMN,
  CONFIDENCE,
  END,
  GRAPH,
  REF,
  TONE,
  type ColumnKind,
  type End,
  type Tone,
} from "./consts";
import { shortName } from "./names";
import type { Hit, Place, UsagesAnswer } from "./types";

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
    const node = nodes.get(place.id) ?? { place, level: hit.depth, count: 0, tests: 0 };
    node.level = Math.min(node.level, hit.depth);
    node.count += 1;
    node.tests += hit.kind === REF.test ? 1 : 0;
    nodes.set(place.id, node);
  }
  return nodes;
}

function byWeight(a: Node, b: Node): number {
  return b.count - a.count || a.place.qualified_name.localeCompare(b.place.qualified_name);
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
  return `M${startX} ${startY} C${middle} ${startY} ${middle} ${endY} ${to.x} ${endY}`;
}

/** Column index of a level: callers to the left of the focus, callees to the right. */
function columnOf(reach: number, side: ColumnKind, level: number): number {
  return side === COLUMN.callers ? reach - level : reach + level;
}

interface Side {
  kind: ColumnKind;
  nodes: Map<number, Node>;
  hits: Hit[];
}

function sideOf(kind: ColumnKind, hits: Hit[], focus: number): Side {
  const end = kind === COLUMN.callers ? END.from : END.to;
  return { kind, nodes: nodesOf(hits, end, focus), hits };
}

function stack(side: Side, reach: number, cards: Map<string, Card>, columns: Column[]): void {
  const levels = [...new Set([...side.nodes.values()].map((node) => node.level))].sort(
    (a, b) => a - b,
  );
  for (const level of levels) {
    const nodes = [...side.nodes.values()].filter((node) => node.level === level).sort(byWeight);
    const index = columnOf(reach, side.kind, level);
    const x = GRAPH.left + index * (GRAPH.columnWidth + GRAPH.columnGap);
    columns.push({ kind: side.kind, level, x, hidden: Math.max(0, nodes.length - GRAPH.maxCards) });
    nodes.slice(0, GRAPH.maxCards).forEach((node, row) => {
      cards.set(`${side.kind}${node.place.id}`, {
        place: node.place,
        name: shortName(node.place.qualified_name),
        x,
        y: GRAPH.top + row * (GRAPH.cardHeight + GRAPH.rowGap),
        height: GRAPH.cardHeight,
        count: node.count,
        focus: false,
        test: node.tests === node.count,
      });
    });
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
    const key = `${hit.from.id}>${hit.to.id}`;
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
      edges.push({ key: `${side.kind}${key}`, d: curve(from, to), tone: toneOf(pair.hits, way) });
    }
  }
  return edges;
}

/** Lays the walks of an answer out as columns of cards and the lines between them (mock 6). */
export function buildGraph(answer: UsagesAnswer): Graph {
  const focusId = answer.symbol.id;
  const callers = sideOf(COLUMN.callers, answer.incoming?.hits ?? [], focusId);
  const callees = sideOf(COLUMN.callees, answer.outgoing?.hits ?? [], focusId);
  const reach = Math.max(0, ...[...callers.nodes.values()].map((node) => node.level));
  const cards = new Map<string, Card>();
  const columns: Column[] = [];
  stack(callers, reach, cards, columns);
  stack(callees, reach, cards, columns);
  const rows = Math.max(
    1,
    ...columns.map((column) => [...cards.values()].filter((card) => card.x === column.x).length),
  );
  const span = (rows - 1) * (GRAPH.cardHeight + GRAPH.rowGap);
  const focusX = GRAPH.left + reach * (GRAPH.columnWidth + GRAPH.columnGap);
  const focus: Card = {
    place: answer.symbol,
    name: shortName(answer.symbol.qualified_name),
    x: focusX,
    y: GRAPH.top + (span + GRAPH.cardHeight - GRAPH.focusHeight) / GRAPH.half,
    height: GRAPH.focusHeight,
    count: 0,
    focus: true,
    test: false,
  };
  columns.push({ kind: COLUMN.focus, level: 0, x: focusX, hidden: 0 });
  const last = Math.max(0, ...[...callees.nodes.values()].map((node) => node.level));
  return {
    columns,
    cards: [focus, ...cards.values()],
    edges: [...edgesOf(callers, cards, focus), ...edgesOf(callees, cards, focus)],
    width: GRAPH.left + GRAPH.left + (reach + last + 1) * (GRAPH.columnWidth + GRAPH.columnGap),
    height: GRAPH.top + span + GRAPH.cardHeight + GRAPH.bottom,
  };
}
