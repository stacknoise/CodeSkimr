import type { Hit } from "../features/search/types";
import type { ExportData } from "./types";
import { RANK_FUZZY, RANK_NAME, RANK_PREFIX, RANK_QUALIFIED, SEARCH_LIMIT } from "./consts";

interface Ranked {
  rank: number;
  hit: Hit;
}

/** Whether the letters of `needle` appear in `text` in this order. */
function isSubsequence(needle: string, text: string): boolean {
  let next = 0;
  for (const letter of text) {
    if (letter === needle[next]) {
      next += 1;
    }
  }
  return next === needle.length;
}

function rankOf(hit: Hit, needle: string): number | null {
  const name = hit.name.toLowerCase();
  if (name.startsWith(needle)) {
    return RANK_PREFIX;
  }
  if (name.includes(needle)) {
    return RANK_NAME;
  }
  if (hit.qualified_name.toLowerCase().includes(needle)) {
    return RANK_QUALIFIED;
  }
  return isSubsequence(needle, name) ? RANK_FUZZY : null;
}

function compare(a: Ranked, b: Ranked): number {
  return (
    a.rank - b.rank ||
    a.hit.name.length - b.hit.name.length ||
    a.hit.qualified_name.localeCompare(b.hit.qualified_name)
  );
}

/** The symbols whose name or qualified name contains `text`, or whose name holds its letters in
 * order, best first. The export holds the names, not the index of the scan, so this is a plain
 * match, not the ranking of the server. */
export function searchSymbols(data: ExportData, text: string): Hit[] {
  const needle = text.trim().toLowerCase();
  if (needle === "") {
    return [];
  }
  const found: Ranked[] = [];
  for (const symbol of Object.values(data.symbols)) {
    const hit: Hit = {
      id: symbol.id,
      name: symbol.name,
      qualified_name: symbol.qualified_name,
      kind: symbol.kind,
      path: symbol.path,
      parent: symbol.breadcrumb[symbol.breadcrumb.length - 1]?.id ?? null,
      project: 0,
      project_name: "",
    };
    const rank = rankOf(hit, needle);
    if (rank !== null) {
      found.push({ rank, hit });
    }
  }
  return found
    .sort(compare)
    .slice(0, SEARCH_LIMIT)
    .map((entry) => entry.hit);
}
