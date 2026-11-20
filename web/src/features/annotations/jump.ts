import { runCommand } from "../../shared/api/commands";
import { SEARCH_COMMAND } from "../search/consts";
import { openHit } from "../search/search";
import type { SearchResult } from "../search/types";
import { NAME_SEPARATOR } from "./consts";
import type { Target } from "./types";

/** Shows a symbol of an annotation in the explorer: the search finds it by the last part of its
 * qualified name, and the hit with the whole name is opened. Returns `false` if the newest scan
 * does not hold it (a namespace, or a symbol that is gone). */
export async function showTarget(project: number, target: Target): Promise<boolean> {
  const short = target.name.split(NAME_SEPARATOR).pop() ?? target.name;
  try {
    const found = await runCommand<SearchResult>(SEARCH_COMMAND, { project, query: short });
    const hit =
      found.hits.find((one) => one.qualified_name === target.name && one.kind === target.kind) ??
      found.hits.find((one) => one.qualified_name === target.name);
    if (hit === undefined) {
      return false;
    }
    openHit(hit);
    return true;
  } catch {
    return false;
  }
}
