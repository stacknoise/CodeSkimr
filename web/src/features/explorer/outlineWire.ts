import { NO_LANGUAGE, QUALIFIED_SEPARATOR } from "./consts";
import type { LayerNode, NamespaceNode, Outline, TypeNode } from "./types";

/** A type as `explorer.outline` sends it: for a type in the list of its namespace the answer
 * leaves out the language of the namespace and a qualified name that is the name of the
 * namespace, a dot and the name (#651). */
type WireType = Omit<TypeNode, "qualified_name" | "language"> & {
  qualified_name?: string;
  language?: string;
};

type WireNamespace = Omit<NamespaceNode, "types" | "module" | "holders" | "file"> & {
  /** Left out when there are none. */
  types?: WireType[];
  /** The file module of the namespace: its qualified name is the name of the namespace. */
  module?: WireType | null;
  holders?: TypeNode[];
  file?: boolean;
  /** The language that the types and the module without their own have. */
  language?: string;
};

type WireLayer = Omit<LayerNode, "namespaces"> & { namespaces: WireNamespace[] };

/** The answer of `explorer.outline`. */
export type WireOutline = Omit<Outline, "layers"> & { layers: WireLayer[] };

function qualifiedName(namespace: string, type: WireType): string {
  return namespace === "" ? type.name : `${namespace}${QUALIFIED_SEPARATOR}${type.name}`;
}

function expandType(namespace: WireNamespace, type: WireType): TypeNode {
  return {
    ...type,
    qualified_name: type.qualified_name ?? qualifiedName(namespace.name, type),
    language: type.language ?? namespace.language ?? NO_LANGUAGE,
  };
}

function expandModule(namespace: WireNamespace, module: WireType): TypeNode {
  return {
    ...module,
    qualified_name: module.qualified_name ?? namespace.name,
    language: module.language ?? namespace.language ?? NO_LANGUAGE,
  };
}

function expandNamespace(namespace: WireNamespace): NamespaceNode {
  const expanded: NamespaceNode & { language?: string } = {
    ...namespace,
    types: (namespace.types ?? []).map((type) => expandType(namespace, type)),
    module: namespace.module ? expandModule(namespace, namespace.module) : null,
    holders: namespace.holders ?? [],
    file: namespace.file ?? false,
  };
  delete expanded.language;
  return expanded;
}

/** Puts back what the answer of the outline left out, so the tree works with whole types. */
export function expandOutline(wire: WireOutline): Outline {
  return {
    ...wire,
    layers: wire.layers.map((layer) => ({
      ...layer,
      namespaces: layer.namespaces.map(expandNamespace),
    })),
  };
}
