import { I18N_KEYS } from "../shared/i18n/keys";
import type { MessageKey } from "../shared/i18n/i18n";
import type { IconName } from "../shared/ui/Icon";

/** Id of the element the UI is drawn into. */
export const ROOT_ID = "root";

/** The screens of the rail, top to bottom; the settings sit at the bottom (mock 13). */
export interface Route {
  path: string;
  label: MessageKey;
  icon: IconName;
}

export const PROJECTS_ROUTE: Route = { path: "/", label: I18N_KEYS.NAV_PROJECTS, icon: "projects" };
export const EXPLORER_ROUTE: Route = {
  path: "/explorer",
  label: I18N_KEYS.NAV_EXPLORER,
  icon: "explorer",
};
/** Screen 6; it is reached from the explorer and the graph, so it has no button in the rail. */
export const USAGES_ROUTE = { path: "/usages", label: I18N_KEYS.USAGES_TITLE } as const;
export const SETTINGS_ROUTE: Route = {
  path: "/settings",
  label: I18N_KEYS.NAV_SETTINGS,
  icon: "settings",
};

/** Screens in the upper part of the rail. */
export const MAIN_ROUTES: Route[] = [PROJECTS_ROUTE, EXPLORER_ROUTE];
/** Screens in the lower part of the rail. */
export const FOOTER_ROUTES: Route[] = [SETTINGS_ROUTE];

/** Logo letter of the top bar. */
export const LOGO_LETTER = "C";

/** Event of the browser when the user goes back or forward. */
export const POPSTATE_EVENT = "popstate";

/** Value of `aria-current` for the screen that is shown. */
export const CURRENT_PAGE = "page";

/** Marks a working connection in the status bar. */
export const STATUS_DOT = "●";
