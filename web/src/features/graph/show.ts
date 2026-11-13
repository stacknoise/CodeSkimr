import { GRAPH_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { runCommand } from "../../shared/api/commands";
import { COMMAND } from "../explorer/consts";
import type { SymbolInfo } from "../explorer/types";
import { wantLayer } from "./graph";

/** Shows the layer of a symbol selected on the dependency map (`G` in the explorer, `Ctrl+Enter` in
 * the quick search). A symbol that no layer owns, or one that cannot be read, shows nothing. */
export async function showInGraph(project: number, symbol: number): Promise<void> {
  try {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol });
    if (info.layer !== null) {
      wantLayer(info.layer);
      navigate(GRAPH_ROUTE.path);
    }
  } catch {
    // The callers fire and forget; a symbol that cannot be read just opens nothing.
  }
}
