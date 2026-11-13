import {
  EDGE,
  EDGE_STYLE,
  LAYER_COLORS,
  LAYER_VAR,
  HALF,
  LOOP_HEIGHT,
  PATH_CURVE,
  PATH_MOVE,
  SPACE,
} from "./consts";
import type { LayerBox, LayerLink } from "./types";

/** How an edge is drawn, from the selection and the switches of the toolbar. */
export type EdgeKind = (typeof EDGE)[keyof typeof EDGE];

/** The look of `link`: a break of the layering is always red; the edges of the selection are
 * coloured by direction; all others are weak. */
export function edgeKind(link: LayerLink, selected: string | null): EdgeKind {
  if (link.violates_layering) {
    return EDGE.violation;
  }
  if (link.from === selected) {
    return EDGE.out;
  }
  return link.to === selected ? EDGE.in : EDGE.weak;
}

/** Whether an edge is drawn: with "violations only" every other edge is hidden. */
export function shown(link: LayerLink, violationsOnly: boolean): boolean {
  return link.from !== link.to && (!violationsOnly || link.violates_layering);
}

/** The style of an edge kind. */
export function styleOf(kind: EdgeKind) {
  return EDGE_STYLE[kind];
}

function middleX(box: LayerBox): number {
  return box.x + box.width * HALF;
}

/** `M x y C x1 y1 x2 y2 x y`: a curve from one point to another through two control points. */
function curve(a: Point, b: Point, controlA: Point, controlB: Point): string {
  const parts = [a, controlA, controlB, b].map((point) => point.x + SPACE + point.y);
  return PATH_MOVE + parts[0] + PATH_CURVE + parts.slice(1).join(SPACE);
}

interface Point {
  x: number;
  y: number;
}

/** The path of an edge: from the bottom of the upper box to the top of the lower one as a
 * curve, and for two boxes in one row a loop above them. Coordinates are layout units. */
export function edgePath(from: LayerBox, to: LayerBox): string {
  const ax = middleX(from);
  const bx = middleX(to);
  if (from.y === to.y) {
    const rise = from.y - LOOP_HEIGHT;
    return curve({ x: ax, y: from.y }, { x: bx, y: to.y }, { x: ax, y: rise }, { x: bx, y: rise });
  }
  const down = from.y < to.y;
  const startY = down ? from.y + from.height : from.y;
  const endY = down ? to.y : to.y + to.height;
  const middle = (startY + endY) * HALF;
  return curve(
    { x: ax, y: startY },
    { x: bx, y: endY },
    { x: ax, y: middle },
    { x: bx, y: middle },
  );
}

/** The colour variable of a layer box by its row, so that the rows are told apart. */
export function rankColor(rank: number): string {
  return LAYER_VAR + ((rank % LAYER_COLORS) + 1) + ")";
}
