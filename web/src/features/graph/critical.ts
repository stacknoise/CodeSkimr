import { NAMESPACE_KIND } from "../annotations/consts";
import { countersOf, type Index } from "../annotations/join";
import { BOX_ID_SUFFIX, LEVEL } from "./consts";
import type { GraphData, LayerBox, Level } from "./types";

/** Whether a box of the map stands for a symbol that is marked system critical. A layer is not
 * annotated; a namespace is looked up as a namespace; a type by its qualified name, which is the
 * name of its box without the id that tells two types of one name apart. */
export function isCriticalBox(index: Index, box: LayerBox, level: Level): boolean {
  if (level === LEVEL.namespaces) {
    return countersOf(index, { kind: NAMESPACE_KIND, name: box.name })?.critical ?? false;
  }
  if (level === LEVEL.types && box.symbol !== null) {
    return index.byName.get(box.name.replace(BOX_ID_SUFFIX, ""))?.critical ?? false;
  }
  return false;
}

/** The map reduced to the critical boxes and the boxes next to them, with the links that touch a
 * critical box. The boxes keep their places; nothing is laid out again. With no critical box on
 * the map the map is returned as it is. */
export function narrowToCritical(data: GraphData, index: Index, level: Level): GraphData {
  const critical = new Set(
    data.boxes.filter((box) => isCriticalBox(index, box, level)).map((box) => box.name),
  );
  if (critical.size === 0) {
    return data;
  }
  const links = data.links.filter((link) => critical.has(link.from) || critical.has(link.to));
  const keep = new Set([...critical, ...links.flatMap((link) => [link.from, link.to])]);
  return { ...data, boxes: data.boxes.filter((box) => keep.has(box.name)), links };
}

/** Whether any box of the map is critical. */
export function hasCriticalBox(data: GraphData, index: Index, level: Level): boolean {
  return data.boxes.some((box) => isCriticalBox(index, box, level));
}
