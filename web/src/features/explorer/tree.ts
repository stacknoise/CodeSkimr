import {
  BASE_INDENT_PX,
  FIELD_KINDS,
  GROUP,
  INDENT_PX,
  KEY_SEPARATOR,
  METHOD_KINDS,
  ROW,
  type RowKind,
} from "./consts";
import type { LayerNode, MemberRow, NamespaceNode, Outline, SymbolInfo, TypeNode } from "./types";

/** One visible row of the explorer tree. */
export interface TreeRow {
  /** Identity of the row, stable while rows open and close. */
  key: string;
  kind: RowKind;
  depth: number;
  label: string;
  /** Count or signature at the right edge. */
  meta: string;
  /** Kind of a type or member row, for its badge. */
  symbolKind: string | null;
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
}

/** Key of the row of a layer, a namespace or a type. */
export const layerKey = (layer: string): string => [ROW.layer, layer].join(KEY_SEPARATOR);
export const namespaceKey = (layer: string, name: string): string =>
  [ROW.namespace, layer, name].join(KEY_SEPARATOR);
export const typeKey = (id: number): string => [ROW.type, String(id)].join(KEY_SEPARATOR);
const groupKey = (id: number, group: string): string =>
  [ROW.group, String(id), group].join(KEY_SEPARATOR);

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
    expandable: false,
    open: false,
    type: null,
    member: null,
  };
}

function badge(kind: string): Pick<TreeRow, "symbolKind"> {
  return { symbolKind: kind };
}

function memberRow(owner: number, depth: number, member: MemberRow): TreeRow {
  return {
    ...base([ROW.member, String(member.id)].join(KEY_SEPARATOR), ROW.member, depth, member.name),
    ...badge(member.kind),
    meta: member.signature ?? "",
    type: owner,
    member: member.id,
  };
}

function groupRows(
  info: SymbolInfo,
  depth: number,
  expanded: ReadonlySet<string>,
  labels: TreeLabels,
): TreeRow[] {
  const groups = [
    { id: GROUP.fields, label: labels.fields, kinds: FIELD_KINDS },
    { id: GROUP.methods, label: labels.methods, kinds: METHOD_KINDS },
  ];
  return groups.flatMap(({ id, label, kinds }) => {
    const members = info.members.filter((member) => kinds.includes(member.kind));
    if (members.length === 0) {
      return [];
    }
    const key = groupKey(info.id, id);
    const open = expanded.has(key);
    const row: TreeRow = {
      ...base(key, ROW.group, depth, label),
      meta: String(members.length),
      expandable: true,
      open,
      type: info.id,
    };
    return open ? [row, ...members.map((member) => memberRow(info.id, depth + 1, member))] : [row];
  });
}

function typeRows(
  node: TypeNode,
  depth: number,
  expanded: ReadonlySet<string>,
  details: ReadonlyMap<number, SymbolInfo>,
  labels: TreeLabels,
): TreeRow[] {
  const key = typeKey(node.id);
  const open = expanded.has(key);
  const row: TreeRow = {
    ...base(key, ROW.type, depth, node.name),
    ...badge(node.kind),
    meta: String(node.members),
    expandable: node.members > 0,
    open,
    type: node.id,
  };
  const info = details.get(node.id);
  return open && info ? [row, ...groupRows(info, depth + 1, expanded, labels)] : [row];
}

function namespaceRows(
  layer: string,
  space: NamespaceNode,
  ctx: Context,
  depth: number,
): TreeRow[] {
  const types = space.types.filter((node) => ctx.matches(node.name));
  if (types.length === 0) {
    return [];
  }
  const key = namespaceKey(layer, space.name);
  const open = ctx.filtering || ctx.expanded.has(key);
  const row: TreeRow = {
    ...base(key, ROW.namespace, depth, space.name || ctx.labels.noNamespace),
    meta: String(types.length),
    expandable: true,
    open,
  };
  const children = types.flatMap((node) =>
    typeRows(node, depth + 1, ctx.expanded, ctx.details, ctx.labels),
  );
  return open ? [row, ...children] : [row];
}

interface Context {
  expanded: ReadonlySet<string>;
  details: ReadonlyMap<number, SymbolInfo>;
  labels: TreeLabels;
  filtering: boolean;
  matches: (name: string) => boolean;
}

function layerRows(layer: LayerNode, ctx: Context): TreeRow[] {
  const children = layer.namespaces.flatMap((space) => namespaceRows(layer.name, space, ctx, 1));
  if (children.length === 0) {
    return [];
  }
  const key = layerKey(layer.name);
  const open = ctx.filtering || ctx.expanded.has(key);
  const types = layer.namespaces.reduce((sum, space) => sum + space.types.length, 0);
  const row: TreeRow = {
    ...base(key, ROW.layer, 0, layer.name || ctx.labels.noLayer),
    meta: String(types),
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
  return outline.layers.flatMap((layer) => layerRows(layer, ctx));
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
