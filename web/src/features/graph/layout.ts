import { createPaneSize } from "../../shared/layout/paneSize";
import { PANEL_PANE_BOUNDS, PANEL_PANE_KEY } from "./consts";

/** The width of the side panel on the right. */
export const panelWidth = createPaneSize(PANEL_PANE_KEY, PANEL_PANE_BOUNDS);
