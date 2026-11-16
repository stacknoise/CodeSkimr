import { createPaneSize } from "../../shared/layout/paneSize";
import { HITS_PANE_BOUNDS, HITS_PANE_KEY, PANEL_PANE_BOUNDS, PANEL_PANE_KEY } from "./consts";

/** The height of the hit list under the graph. */
export const hitsHeight = createPaneSize(HITS_PANE_KEY, HITS_PANE_BOUNDS);
/** The width of the side panel on the right. */
export const panelWidth = createPaneSize(PANEL_PANE_KEY, PANEL_PANE_BOUNDS);
