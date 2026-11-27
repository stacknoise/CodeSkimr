import { I18N_KEYS } from "../shared/i18n/keys";
import type { MessageKey } from "../shared/i18n/i18n";
import type { ErrorCode } from "../shared/api/error-codes.generated";
import type { IconName } from "../shared/ui/Icon";

/** Id of the element the UI is drawn into. */
export const ROOT_ID = "root";

/** Id of the loading page in `index.html`; the app removes it as soon as it starts. */
export const BOOT_ID = "boot";

/** The screens of the rail, top to bottom; the settings sit at the bottom (mock 13). */
export interface Route {
  path: string;
  label: MessageKey;
  icon: IconName;
  /** The screen shows the data of a project, so it stays shut until one is open. */
  needsProject?: boolean;
}

export const PROJECTS_ROUTE: Route = { path: "/", label: I18N_KEYS.NAV_PROJECTS, icon: "projects" };
export const EXPLORER_ROUTE: Route = {
  path: "/explorer",
  label: I18N_KEYS.NAV_EXPLORER,
  icon: "explorer",
  needsProject: true,
};
/** Screen 6; it is reached from the explorer and the graph, so it has no button in the rail. */
export const USAGES_ROUTE = { path: "/usages", label: I18N_KEYS.USAGES_TITLE } as const;
/** The list of the error codes to look up; it is reached from the settings, so it has no button in
 * the rail. */
export const CODES_ROUTE = { path: "/codes", label: I18N_KEYS.CODES_TITLE } as const;
/** The board of the tasks and the notes of the project (mock 16). */
export const ANNOTATIONS_ROUTE: Route = {
  path: "/annotations",
  label: I18N_KEYS.NAV_ANNOTATIONS,
  icon: "annotations",
  needsProject: true,
};
/** The former scan screen; its content is the tab "Scan" of the projects screen now, and the old
 * address leads there (ADR 0096). */
export const LEGACY_SCAN_PATH = "/scan";
export const GRAPH_ROUTE: Route = {
  path: "/graph",
  label: I18N_KEYS.NAV_GRAPH,
  icon: "graph",
  needsProject: true,
};
/** The statistics of the project (ADR 0081). */
export const STATS_ROUTE: Route = {
  path: "/stats",
  label: I18N_KEYS.NAV_STATS,
  icon: "stats",
  needsProject: true,
};
/** What the difference between two scans of the project hits (ADR 0097). */
export const CHANGE_ROUTE: Route = {
  path: "/change",
  label: I18N_KEYS.NAV_CHANGE,
  icon: "diff",
  needsProject: true,
};
/** The architecture rules of the project and their violations (ADR 0104). */
export const RULES_ROUTE: Route = {
  path: "/rules",
  label: I18N_KEYS.NAV_RULES,
  icon: "rules",
  needsProject: true,
};
/** The layers of the project: where they are set from its namespaces (ADR 0065, ADR 0112). */
export const LAYERS_ROUTE: Route = {
  path: "/layers",
  label: I18N_KEYS.NAV_LAYERS,
  icon: "layers",
  needsProject: true,
};
export const SETTINGS_ROUTE: Route = {
  path: "/settings",
  label: I18N_KEYS.NAV_SETTINGS,
  icon: "settings",
};

/** The button of the rail that stays marked on a screen that has no button of its own: the symbol
 * view is the symbol level of the graph. */
export const RAIL_OWNER: Record<string, string> = { [USAGES_ROUTE.path]: GRAPH_ROUTE.path };

/** Paths of the screens that show the data of a project; the usages screen has no button of its
 * own but needs a project as well. */
export const PROJECT_PATHS: readonly string[] = [
  EXPLORER_ROUTE.path,
  GRAPH_ROUTE.path,
  ANNOTATIONS_ROUTE.path,
  STATS_ROUTE.path,
  CHANGE_ROUTE.path,
  LAYERS_ROUTE.path,
  RULES_ROUTE.path,
  USAGES_ROUTE.path,
];

/** Screens in the upper part of the rail. */
export const MAIN_ROUTES: Route[] = [
  PROJECTS_ROUTE,
  EXPLORER_ROUTE,
  GRAPH_ROUTE,
  STATS_ROUTE,
  CHANGE_ROUTE,
  LAYERS_ROUTE,
  RULES_ROUTE,
  ANNOTATIONS_ROUTE,
];
/** Screens in the lower part of the rail. */
export const FOOTER_ROUTES: Route[] = [SETTINGS_ROUTE];

/** Logo letter of the top bar. */
export const LOGO_LETTER = "C";

/** Separates the product name from the project in the top bar. */
export const BREADCRUMB_SLASH = "/";

/** Separates the parts of the chip next to the project name. */
export const CHIP_SEPARATOR = " · ";

/** Event of the browser when the user goes back or forward. */
export const POPSTATE_EVENT = "popstate";

/** Value of `aria-current` for the screen that is shown. */
export const CURRENT_PAGE = "page";

/** Marks a working connection in the status bar. */
export const STATUS_DOT = "●";

/** Screens that use the whole main area, without padding. */
export const FLUSH_PATHS: readonly string[] = [
  EXPLORER_ROUTE.path,
  GRAPH_ROUTE.path,
  USAGES_ROUTE.path,
];

/** The line in the console and in the copied details that names the code. */
export const ERROR_CODE_FIELD = "code";
/** Separates the lines of the copied details of the error panel. */
export const DETAILS_LINE_BREAK = "\n";
/** The code of an exception nobody expected: it is a bug (blueprint 11.6, E-03). */
export const UNEXPECTED_CODE: ErrorCode = "UI-901";
