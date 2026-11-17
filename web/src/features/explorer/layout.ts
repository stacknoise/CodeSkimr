import { createPaneSize } from "../../shared/layout/paneSize";
import { PANE_BOUNDS, PANE_KEY } from "./consts";

/** The width of the tree on the left. */
export const treeWidth = createPaneSize(PANE_KEY.tree, PANE_BOUNDS.tree);
/** The width of the source on the right. */
export const sourceWidth = createPaneSize(PANE_KEY.source, PANE_BOUNDS.source);
/** The width of the side panel next to a selected member (the source has the middle column then). */
export const panelWidth = createPaneSize(PANE_KEY.panel, PANE_BOUNDS.panel);
