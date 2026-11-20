import {
  ALIAS_KINDS,
  BASE_INDENT_PX,
  FIELD_KINDS,
  GROUP,
  INDENT_PX,
  KEY_SEPARATOR,
  NAME_PART,
  METHOD_KINDS,
  ROW,
  TYPE_KINDS,
  type RowKind,
} from "./consts";
import { NAMESPACE_KIND } from "../annotations/consts";
import type { Target } from "../annotations/types";
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
  /** Language of a type row (id of the language pack), `null` for the other rows. */
  language: string | null;
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
  /** The symbol an annotation on this row is about; `null` for a layer and a group. */
  target: Target | null;
  /** The name whose critical symbols below are counted on a collapsed namespace or type. */
  scope: string | null;
  /** The qualified names of the members a group holds, to count the critical ones. */
  names: readonly string[];
}

/** The texts the tree needs that depend on the language. */
export interface TreeLabels {
  noLayer: string;
  noNamespace: string;
  fields: string;
  methods: string;
  nested: string;
  aliases: string;
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
    language: null,
    symbolKind: null,
    layerIndex: null,
    expandable: false,
    open: false,
    type: null,
    member: null,
    target: null,
    scope: null,
    names: [],
  };
}

function memberRow(owner: number, depth: number, member: MemberRow): TreeRow {
  return {
    ...base(memberKey(member.id), ROW.member, depth, member.name),
    symbolKind: member.kind,
    meta: member.signature ?? "",
    type: owner,
    member: member.id,
    target: { kind: member.kind, name: member.qualified_name },
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
function nestedNode(member: MemberRow, language: string): TypeNode {
  return {
    id: member.id,
    name: member.name,
    qualified_name: member.qualified_name,
    kind: member.kind,
    members: 0,
    language,
  };
}

/** The group of the tree that holds a member of this kind, `null` if it has none. */
export function groupOfKind(kind: string): string | null {
  if (FIELD_KINDS.includes(kind)) {
    return GROUP.fields;
  }
  if (METHOD_KINDS.includes(kind)) {
    return GROUP.methods;
  }
  if (ALIAS_KINDS.includes(kind)) {
    return GROUP.aliases;
  }
  return TYPE_KINDS.includes(kind) ? GROUP.nested : null;
}

function groupRow(info: SymbolInfo, depth: number, id: string, label: string, held: MemberRow[]) {
  const key = groupKey(info.id, id);
  return {
    ...base(key, ROW.group, depth, label),
    meta: String(held.length),
    expandable: true,
    type: info.id,
    names: held.map((member) => member.qualified_name),
  };
}

/** The groups of the members of a symbol. A module leaves out the group of nested types: its
 * types are rows of the namespace below it already. */
function groupRows(info: SymbolInfo, depth: number, ctx: Context, withTypes = true): TreeRow[] {
  const groups = [
    { id: GROUP.fields, label: ctx.labels.fields, kinds: FIELD_KINDS },
    { id: GROUP.methods, label: ctx.labels.methods, kinds: METHOD_KINDS },
    { id: GROUP.aliases, label: ctx.labels.aliases, kinds: ALIAS_KINDS },
    ...(withTypes ? [{ id: GROUP.nested, label: ctx.labels.nested, kinds: TYPE_KINDS }] : []),
  ];
  return groups.flatMap(({ id, label, kinds }) => {
    const members = info.members.filter((member) => kinds.includes(member.kind));
    if (members.length === 0) {
      return [];
    }
    const row = groupRow(info, depth, id, label, members);
    const open = ctx.expanded.has(row.key);
    const children =
      id === GROUP.nested
        ? members.flatMap((member) =>
            typeRows(nestedNode(member, info.language), depth + 1, ctx, true),
          )
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
    language: node.language,
    meta: nested ? "" : String(node.members),
    expandable: nested || node.members > 0,
    open,
    type: node.id,
    target: { kind: node.kind, name: node.qualified_name },
    scope: node.qualified_name,
  };
  const info = ctx.details.get(node.id);
  return open && info ? [row, ...groupRows(info, depth + 1, ctx)] : [row];
}

/** A namespace segment of a layer: the types declared exactly here and the segments below. */
interface Segment {
  /** The name up to and including this segment, as the scan wrote it (`App.Domain`). */
  path: string;
  /** The last part of the name (`Domain`). */
  label: string;
  children: Map<string, Segment>;
  types: TypeNode[];
  /** The file module with this name (a Python file is its own namespace), if there is one. */
  module: TypeNode | null;
}

function segment(path: string, label: string): Segment {
  return { path, label, children: new Map<string, Segment>(), types: [], module: null };
}

/** The parts of a namespace name with the name up to and including each part (`App.Domain` gives
 * `App` at `App` and `Domain` at `App.Domain`). A name without any part, empty or only separators,
 * has none; its types belong to the unnamed namespace. */
export function nameParts(name: string): { label: string; path: string }[] {
  return [...name.matchAll(NAME_PART)].map((part) => ({
    label: part[0],
    path: name.slice(0, (part.index ?? 0) + part[0].length),
  }));
}

/** Groups the namespaces of a layer by their parts, so `App.Domain` and `App.Ui` are two rows
 * below one row `App`. A namespace without a name is the segment with the empty path. */
export function segmentsOf(spaces: readonly NamespaceNode[]): Segment[] {
  const roots = new Map<string, Segment>();
  for (const space of spaces) {
    const parts = nameParts(space.name);
    if (parts.length === 0) {
      const root = roots.get("") ?? segment("", "");
      roots.set("", root);
      root.types.push(...space.types);
      continue;
    }
    let level = roots;
    let current: Segment | undefined;
    for (const { label, path } of parts) {
      current = level.get(path) ?? segment(path, label);
      level.set(path, current);
      level = current.children;
    }
    current?.types.push(...space.types);
    if (current && space.module) {
      current.module = space.module;
    }
  }
  return [...roots.values()];
}

/** How many types below a segment, itself included, match the filter. */
function matching(node: Segment, ctx: Context): number {
  const own = node.types.filter((type) => ctx.matches(type.name)).length;
  return [...node.children.values()].reduce((sum, child) => sum + matching(child, ctx), own);
}

/** Whether a segment has a row: a type below it matches, or it is a module whose name does
 * (a module without types, like an empty `__init__`, is still a row). */
function shown(node: Segment, ctx: Context): boolean {
  return (
    matching(node, ctx) > 0 ||
    [node, ...descendants(node)].some((part) => part.module && ctx.matches(part.module.name))
  );
}

function descendants(node: Segment): Segment[] {
  return [...node.children.values()].flatMap((child) => [child, ...descendants(child)]);
}

/** What the row of a segment says about the module that it stands for. */
function moduleFields(node: Segment): Partial<TreeRow> {
  return node.module
    ? { symbolKind: node.module.kind, language: node.module.language, type: node.module.id }
    : {};
}

/** The rows of a segment: its row, and if it is open the segments below, then its own types.
 * A segment without a matching type has no row. `depth` is the depth of its row. */
function segmentRows(layer: string, node: Segment, depth: number, ctx: Context): TreeRow[] {
  const count = matching(node, ctx);
  if (!shown(node, ctx)) {
    return [];
  }
  const key = namespaceKey(layer, node.path);
  const open = ctx.filtering || ctx.expanded.has(key);
  const row: TreeRow = {
    ...base(key, ROW.namespace, depth, node.label || ctx.labels.noNamespace),
    ...moduleFields(node),
    meta: count > 0 || !node.module ? String(count) : "",
    expandable: count > 0 || node.children.size > 0 || (node.module?.members ?? 0) > 0,
    open,
    target: node.path === "" ? null : { kind: NAMESPACE_KIND, name: node.path },
    scope: node.path === "" ? null : node.path,
  };
  if (!open) {
    return [row];
  }
  // The members of a module that are no types: its constants and functions.
  const info = node.module && ctx.expanded.has(key) ? ctx.details.get(node.module.id) : undefined;
  const held = info ? groupRows(info, depth + 1, ctx, false) : [];
  const below = [...node.children.values()].flatMap((child) =>
    segmentRows(layer, child, depth + 1, ctx),
  );
  const own = node.types
    .filter((type) => ctx.matches(type.name))
    .flatMap((type) => typeRows(type, depth + 1, ctx));
  return [row, ...held, ...below, ...own];
}

function layerRows(layer: LayerNode, index: number, ctx: Context, flat: boolean): TreeRow[] {
  const children = segmentsOf(layer.namespaces).flatMap((root) =>
    segmentRows(layer.name, root, flat ? 0 : 1, ctx),
  );
  if (flat || children.length === 0) {
    return children;
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
  // Without any layer the "no layer" row would only be a lid on everything: the namespaces
  // stand at the top instead.
  const flat = outline.layers.every((layer) => layer.name === "");
  return outline.layers.flatMap((layer, index) => layerRows(layer, index, ctx, flat));
}

/** The keys that must be open for the row of `type` to be visible: its layer, every segment of
 * its namespace down to the last one, and the type itself. The last key is the row of `type`; for a
 * module it is its namespace segment. Empty if the type is not in the
 * outline. */
export function revealKeys(outline: Outline, type: number): string[] {
  for (const layer of outline.layers) {
    for (const space of layer.namespaces) {
      const parts = nameParts(space.name);
      const paths = parts.length === 0 ? [""] : parts.map((part) => part.path);
      const spaces = paths.map((path) => namespaceKey(layer.name, path));
      if (space.types.some((node) => node.id === type)) {
        return [layerKey(layer.name), ...spaces, typeKey(type)];
      }
      if (space.module?.id === type) {
        // The row of a module is the row of its namespace: it is the last key, not opened.
        return [layerKey(layer.name), ...spaces];
      }
    }
  }
  return [];
}

/** What the tree has to open and where the cursor goes to show the namespace `name`: its layer and
 * every segment down to it (it opens itself too). `null` if no namespace of the outline has it as
 * a part of its name. A name that exists in several layers is shown in the first of them: the map
 * of the namespaces does not say which layer it came from. */
export function revealNamespace(
  outline: Outline,
  name: string,
): { open: string[]; cursor: string } | null {
  for (const layer of outline.layers) {
    for (const space of layer.namespaces) {
      const parts = nameParts(space.name);
      const at = parts.findIndex((part) => part.path === name);
      if (at >= 0) {
        const paths = parts.slice(0, at + 1).map((part) => namespaceKey(layer.name, part.path));
        return { open: [layerKey(layer.name), ...paths], cursor: namespaceKey(layer.name, name) };
      }
    }
  }
  return null;
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
