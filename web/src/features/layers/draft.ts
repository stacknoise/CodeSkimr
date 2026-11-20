import { SEPARATOR_AT_START } from "./consts";
import type { LayerEntry } from "./types";

/** Whether `name` equals `prefix` or lies below it, up to a separator. */
export function covers(prefix: string, name: string): boolean {
  if (!name.startsWith(prefix)) {
    return false;
  }
  const rest = name.slice(prefix.length);
  return rest === "" || SEPARATOR_AT_START.test(rest);
}

/** The name of the first layer that owns `name`, `null` if none does. */
export function ownerOf(layers: readonly LayerEntry[], name: string): string | null {
  const owner = layers.find((layer) => layer.namespaces.some((prefix) => covers(prefix, name)));
  return owner?.name ?? null;
}

/** The namespaces of a layer after `added` joined: those that the new ones cover are dropped, and
 * a new one that an old one covers is not added twice. */
function joined(namespaces: readonly string[], added: readonly string[]): string[] {
  const kept = namespaces.filter((old) => !added.some((prefix) => covers(prefix, old)));
  const fresh = added.filter((prefix) => !kept.some((old) => covers(old, prefix)));
  return [...kept, ...fresh];
}

/** Gives the layer at `index` the `prefixes`. Another layer that has exactly one of them gives
 * it up, so a prefix belongs to one layer. */
export function addToLayer(
  layers: readonly LayerEntry[],
  index: number,
  prefixes: readonly string[],
): LayerEntry[] {
  return layers.map((layer, at) =>
    at === index
      ? { ...layer, namespaces: joined(layer.namespaces, prefixes) }
      : { ...layer, namespaces: layer.namespaces.filter((own) => !prefixes.includes(own)) },
  );
}

/** A new layer at the end, made of `prefixes`. */
export function addLayer(
  layers: readonly LayerEntry[],
  name: string,
  prefixes: readonly string[],
): LayerEntry[] {
  return addToLayer([...layers, { name: name.trim(), namespaces: [] }], layers.length, prefixes);
}

/** The layers without the namespace `prefix` of the layer at `index`. */
export function removeNamespace(
  layers: readonly LayerEntry[],
  index: number,
  prefix: string,
): LayerEntry[] {
  return layers.map((layer, at) =>
    at === index
      ? { ...layer, namespaces: layer.namespaces.filter((own) => own !== prefix) }
      : layer,
  );
}

/** The layers with the layer at `index` renamed. */
export function renameLayer(
  layers: readonly LayerEntry[],
  index: number,
  name: string,
): LayerEntry[] {
  return layers.map((layer, at) => (at === index ? { ...layer, name } : layer));
}

/** The layers without the layer at `index`. */
export function removeLayer(layers: readonly LayerEntry[], index: number): LayerEntry[] {
  return layers.filter((_, at) => at !== index);
}

/** The layers with the layer at `index` moved by `step` places; at the ends nothing moves. */
export function moveLayer(
  layers: readonly LayerEntry[],
  index: number,
  step: number,
): LayerEntry[] {
  const target = index + step;
  const moved = [...layers];
  const layer = moved[index];
  if (layer === undefined || target < 0 || target >= moved.length) {
    return moved;
  }
  moved.splice(index, 1);
  moved.splice(target, 0, layer);
  return moved;
}

/** Whether the layers can be saved: every name is filled and used once, every layer has a
 * namespace. The server checks the same. */
export function isValid(layers: readonly LayerEntry[]): boolean {
  const names = layers.map((layer) => layer.name.trim());
  return (
    names.every((name) => name !== "") &&
    new Set(names).size === names.length &&
    layers.every((layer) => layer.namespaces.length > 0)
  );
}

/** Whether two lists are the same layers in the same order. */
export function sameLayers(a: readonly LayerEntry[], b: readonly LayerEntry[]): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
