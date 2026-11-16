import { OPEN_ALL_LIMIT, SEPARATOR } from "./consts";
import type { NamespaceEntry, PrefixNode } from "./types";

/** The prefixes of a qualified name, shortest first, each with its last segment. */
export function segmentsOf(name: string): { prefix: string; label: string }[] {
  const segments: { prefix: string; label: string }[] = [];
  let start = 0;
  for (const match of name.matchAll(SEPARATOR)) {
    segments.push({ prefix: name.slice(0, match.index), label: name.slice(start, match.index) });
    start = match.index + match[0].length;
  }
  segments.push({ prefix: name, label: name.slice(start) });
  return segments.filter((segment) => segment.prefix !== "");
}

/** The tree of the prefixes of all namespaces; a node counts the types below it. */
export function buildTree(namespaces: readonly NamespaceEntry[]): PrefixNode[] {
  const nodes = new Map<string, PrefixNode>();
  const roots: PrefixNode[] = [];
  for (const namespace of namespaces) {
    let siblings = roots;
    segmentsOf(namespace.name).forEach((segment, depth) => {
      let node = nodes.get(segment.prefix);
      if (node === undefined) {
        node = { prefix: segment.prefix, label: segment.label, depth, types: 0, children: [] };
        nodes.set(segment.prefix, node);
        siblings.push(node);
      }
      node.types += namespace.types;
      siblings = node.children;
    });
  }
  return roots;
}

/** The prefixes that are opened when the tree is first shown. */
export function initiallyOpen(roots: readonly PrefixNode[]): Set<string> {
  const open = new Set<string>();
  const stack = [...roots];
  const all: PrefixNode[] = [];
  for (let node = stack.pop(); node !== undefined; node = stack.pop()) {
    all.push(node);
    stack.push(...node.children);
  }
  if (all.length <= OPEN_ALL_LIMIT) {
    all.forEach((node) => open.add(node.prefix));
  }
  return open;
}

/** The rows that are visible: a node's children follow it if it is open. */
export function visibleRows(roots: readonly PrefixNode[], open: ReadonlySet<string>): PrefixNode[] {
  return roots.flatMap((node) => [
    node,
    ...(open.has(node.prefix) ? visibleRows(node.children, open) : []),
  ]);
}
