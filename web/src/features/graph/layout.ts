import { createPaneSize } from "../../shared/layout/paneSize";
import { MODULES_PANE_BOUNDS, MODULES_PANE_KEY, PANEL_PANE_BOUNDS, PANEL_PANE_KEY } from "./consts";

/** The width of the side panel on the right. */
export const panelWidth = createPaneSize(PANEL_PANE_KEY, PANEL_PANE_BOUNDS);

/** The width of the panel of the level "Modules". */
export const modulesPanelWidth = createPaneSize(MODULES_PANE_KEY, MODULES_PANE_BOUNDS);
