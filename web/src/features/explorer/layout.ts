import { createPaneSize } from "../../shared/layout/paneSize";
import { PANE_BOUNDS, PANE_KEY } from "./consts";

/** The width of the tree on the left. */
export const treeWidth = createPaneSize(PANE_KEY.tree, PANE_BOUNDS.tree);
/** The width of the source on the right. */
export const sourceWidth = createPaneSize(PANE_KEY.source, PANE_BOUNDS.source);
