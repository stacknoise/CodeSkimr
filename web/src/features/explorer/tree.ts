import {
  BASE_INDENT_PX,
  FIELD_KINDS,
  GROUP,
  INDENT_PX,
  KEY_SEPARATOR,
  METHOD_KINDS,
  ROW,
  TYPE_DEPTH,
  TYPE_KINDS,
  type RowKind,
} from "./consts";
import type { LayerNode, MemberRow, NamespaceNode, Outline, SymbolInfo, TypeNode } from "./types";

/** One visible row of the explorer tree. */
export interface TreeRow {
  /** Identity of the row, stable while rows open and close. */
  key: string;
  kind: RowKind;
  /** Nesting, 0 for a layer. */
  depth: number;
  label: string;
  /** Count or signature at the right edge. */
  meta: string;
  /** Kind of a type or member row, for its badge. */
  symbolKind: string | null;
  /** Position of a layer among the layers, for its colour dot; `null` for other rows. */
  layerIndex: number | null;
  /** Whether the row has children, and whether they are shown. */
  expandable: boolean;
  open: boolean;
  /** The type that a type or member row belongs to, and the member itself. */
  type: number | null;
  member: number | null;
}

/** The texts the tree needs that depend on the language. */
export interface TreeLabels {
  noLayer: string;
  noNamespace: string;
  fields: string;
  methods: string;
  nested: string;
}

/** Key of the row of a layer, a namespace or a type. */
export const layerKey = (layer: string): string => [ROW.layer, layer].join(KEY_SEPARATOR);
export const namespaceKey = (layer: string, name: string): string =>
  [ROW.namespace, layer, name].join(KEY_SEPARATOR);
export const typeKey = (id: number): string => [ROW.type, String(id)].join(KEY_SEPARATOR);
export const groupKey = (id: number, group: string): string =>
  [ROW.group, String(id), group].join(KEY_SEPARATOR);
export const memberKey = (id: number): string => [ROW.member, String(id)].join(KEY_SEPARATOR);

export function indent(depth: number): number {
  return BASE_INDENT_PX + depth * INDENT_PX;
}

function base(key: string, kind: RowKind, depth: number, label: string): TreeRow {
  return {
    key,
    kind,
    depth,
    label,
    meta: "",
    symbolKind: null,
    layerIndex: null,
    expandable: false,
    open: false,
    type: null,
    member: null,
  };
}

function memberRow(owner: number, depth: number, member: MemberRow): TreeRow {
  return {
    ...base(memberKey(member.id), ROW.member, depth, member.name),
    symbolKind: member.kind,
    meta: member.signature ?? "",
    type: owner,
    member: member.id,
  };
}

/** Everything the rows below a layer need. */
interface Context {
  expanded: ReadonlySet<string>;
  details: ReadonlyMap<number, SymbolInfo>;
  labels: TreeLabels;
  filtering: boolean;
  matches: (name: string) => boolean;
}

/** A type inside a type is a row of its own, so it can be opened and shown; its member count
 * is not known before it is read. */
function nestedNode(member: MemberRow): TypeNode {
  return { id: member.id, name: member.name, kind: member.kind, members: 0 };
}

/** The group of the tree that holds a member of this kind, `null` if it has none. */
export function groupOfKind(kind: string): string | null {
  if (FIELD_KINDS.includes(kind)) {
    return GROUP.fields;
  }
  if (METHOD_KINDS.includes(kind)) {
    return GROUP.methods;
  }
  return TYPE_KINDS.includes(kind) ? GROUP.nested : null;
}

function groupRow(info: SymbolInfo, depth: number, id: string, label: string, count: number) {
  const key = groupKey(info.id, id);
  return {
    ...base(key, ROW.group, depth, label),
    meta: String(count),
    expandable: true,
    type: info.id,
  };
}

function groupRows(info: SymbolInfo, depth: number, ctx: Context): TreeRow[] {
  const groups = [
    { id: GROUP.fields, label: ctx.labels.fields, kinds: FIELD_KINDS },
    { id: GROUP.methods, label: ctx.labels.methods, kinds: METHOD_KINDS },
    { id: GROUP.nested, label: ctx.labels.nested, kinds: TYPE_KINDS },
  ];
  return groups.flatMap(({ id, label, kinds }) => {
    const members = info.members.filter((member) => kinds.includes(member.kind));
    if (members.length === 0) {
      return [];
    }
    const row = groupRow(info, depth, id, label, members.length);
    const open = ctx.expanded.has(row.key);
    const children =
      id === GROUP.nested
        ? members.flatMap((member) => typeRows(nestedNode(member), depth + 1, ctx, true))
        : members.map((member) => memberRow(info.id, depth + 1, member));
    return [{ ...row, open }, ...(open ? children : [])];
  });
}

function typeRows(node: TypeNode, depth: number, ctx: Context, nested = false): TreeRow[] {
  const key = typeKey(node.id);
  const open = ctx.expanded.has(key);
  const row: TreeRow = {
    ...base(key, ROW.type, depth, node.name),
    symbolKind: node.kind,
    meta: nested ? "" : String(node.members),
    expandable: nested || node.members > 0,
    open,
    type: node.id,
  };
  const info = ctx.details.get(node.id);
  return open && info ? [row, ...groupRows(info, depth + 1, ctx)] : [row];
}

function namespaceRows(layer: string, space: NamespaceNode, ctx: Context): TreeRow[] {
  const types = space.types.filter((node) => ctx.matches(node.name));
  if (types.length === 0) {
    return [];
  }
  const key = namespaceKey(layer, space.name);
  const open = ctx.filtering || ctx.expanded.has(key);
  const row: TreeRow = {
    ...base(key, ROW.namespace, 1, space.name || ctx.labels.noNamespace),
    meta: String(types.length),
    expandable: true,
    open,
  };
  return open ? [row, ...types.flatMap((node) => typeRows(node, TYPE_DEPTH, ctx))] : [row];
}

function layerRows(layer: LayerNode, index: number, ctx: Context): TreeRow[] {
  const children = layer.namespaces.flatMap((space) => namespaceRows(layer.name, space, ctx));
  if (children.length === 0) {
    return [];
  }
  const key = layerKey(layer.name);
  const open = ctx.filtering || ctx.expanded.has(key);
  const types = layer.namespaces.reduce((sum, space) => sum + space.types.length, 0);
  const row: TreeRow = {
    ...base(key, ROW.layer, 0, layer.name || ctx.labels.noLayer),
    meta: String(types),
    layerIndex: index,
    expandable: true,
    open,
  };
  return open ? [row, ...children] : [row];
}

/** The visible rows of the tree. A filter shows every type whose name contains it, with its
 * layer and namespace opened; without a filter only rows below an opened parent show. Type rows
 * open into groups once their members have been read (`details`). */
export function buildRows(
  outline: Outline,
  expanded: ReadonlySet<string>,
  details: ReadonlyMap<number, SymbolInfo>,
  filter: string,
  labels: TreeLabels,
): TreeRow[] {
  const needle = filter.trim().toLowerCase();
  const ctx: Context = {
    expanded,
    details,
    labels,
    filtering: needle !== "",
    matches: (name) => name.toLowerCase().includes(needle),
  };
  return outline.layers.flatMap((layer, index) => layerRows(layer, index, ctx));
}

/** The keys that must be open for the row of `type` to be visible: its layer, its namespace and
 * the type itself. Empty if the type is not in the outline. */
export function revealKeys(outline: Outline, type: number): string[] {
  for (const layer of outline.layers) {
    for (const space of layer.namespaces) {
      if (space.types.some((node) => node.id === type)) {
        return [layerKey(layer.name), namespaceKey(layer.name, space.name), typeKey(type)];
      }
    }
  }
  return [];
}

/** Where a row stands among its siblings: the count of rows with the same parent and its place
 * among them, counted from 1. */
export interface Place {
  size: number;
  position: number;
}

/** The place of every row among its siblings. An open parent shows all its children, so the
 * numbers are those of the whole tree, not of the rows in the window of the list. */
export function siblingPlaces(rows: readonly TreeRow[]): Place[] {
  const places: Place[] = [];
  const groups: Place[][] = [];
  const all = new Set<Place[]>();
  rows.forEach((row) => {
    groups.length = row.depth + 1;
    const group = groups[row.depth] ?? [];
    groups[row.depth] = group;
    all.add(group);
    const place: Place = { size: 0, position: group.length + 1 };
    group.push(place);
    places.push(place);
  });
  all.forEach((group) =>
    group.forEach((place) => {
      place.size = group.length;
    }),
  );
  return places;
}
