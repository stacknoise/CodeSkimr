import { PANEL_WIDTH_VAR } from "./consts";
import type { PaneSize } from "./paneSize";

/** The inline style of a side panel whose width the user can change: the style sheet reads the
 * width from a custom property, so its narrow layout can still override it. */
export function panelStyle(pane: PaneSize): Record<string, string> {
  return { [PANEL_WIDTH_VAR]: `${pane.size()}px` };
}
