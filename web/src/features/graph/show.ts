import { GRAPH_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { runCommand } from "../../shared/api/commands";
import { COMMAND } from "../explorer/consts";
import type { SymbolInfo } from "../explorer/types";
import { DEPTH_ALL, LEVEL } from "./consts";
import { setGraphDepth, setGraphLevel, wantLayer } from "./graph";

/** Shows the layer of a symbol selected on the dependency map (`G` in the explorer, `Ctrl+Enter` in
 * the quick search). A symbol that no layer owns, or one that cannot be read, shows nothing. */
export async function showInGraph(project: number, symbol: number): Promise<void> {
  try {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol });
    if (info.layer !== null) {
      setGraphLevel(LEVEL.layers);
      wantLayer(info.layer);
      navigate(GRAPH_ROUTE.path);
    }
  } catch {
    // The callers fire and forget; a symbol that cannot be read just opens nothing.
  }
}

/** Shows a module on the tab "Modules" of the dependency map, with every module as the cycle rule
 * sees it (the rules screen: "Show in the graph"). */
export function showModuleInGraph(module: string): void {
  setGraphLevel(LEVEL.modules);
  setGraphDepth(DEPTH_ALL);
  wantLayer(module);
  navigate(GRAPH_ROUTE.path);
}
