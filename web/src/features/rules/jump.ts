import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { requestFocus } from "../../shared/navigation/navigation";
import { openInExplorer } from "../usages/usages";
import { NAMESPACE_KINDS } from "./consts";
import type { SymbolRef } from "./types";

/** Shows the symbol of a violation in the explorer: a namespace or module as its row of the tree,
 * a type or a member where the usages and the quick search show it. */
export async function showInExplorer(
  project: number,
  name: string,
  symbol: SymbolRef,
): Promise<void> {
  if (NAMESPACE_KINDS.includes(symbol.kind)) {
    requestFocus({ namespace: name });
    navigate(EXPLORER_ROUTE.path);
    return;
  }
  await openInExplorer(project, {
    id: symbol.id,
    qualified_name: name,
    kind: symbol.kind,
    layer: null,
  });
}
