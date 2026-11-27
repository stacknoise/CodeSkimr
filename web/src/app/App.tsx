import { createEffect, ErrorBoundary, For, onCleanup, Show } from "solid-js";
import { Dynamic } from "solid-js/web";
import { AlertsDrawer } from "../features/alerts/Drawer";
import { hideDrawer, refreshCount } from "../features/alerts/store";
import { CodesScreen } from "../features/codes/CodesScreen";
import { BoardScreen } from "../features/annotations/BoardScreen";
import { ConnectionScreen } from "../features/session/ConnectionScreen";
import { GraphScreen } from "../features/graph/GraphScreen";
import { ExplorerScreen } from "../features/explorer/ExplorerScreen";
import { listenToScans } from "../features/scan/events";
import { ChangeScreen } from "../features/change/ChangeScreen";
import { RulesScreen } from "../features/rules/RulesScreen";
import { StatsScreen } from "../features/stats/StatsScreen";
import { showScan } from "../features/projects/page";
import { ProjectsScreen } from "../features/projects/ProjectsScreen";
import { QuickSearch } from "../features/search/QuickSearch";
import { useSearchShortcut } from "../features/search/keys";
import { openSearch } from "../features/search/search";
import { UsagesScreen } from "../features/usages/UsagesScreen";
import { StatusBar } from "../features/status/StatusBar";
import { SettingsScreen } from "../features/settings/SettingsScreen";
import { t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { activeProject } from "../shared/project/active";
import { KIND } from "../shared/session/consts";
import { sessionState } from "../shared/session/session";
import { cx } from "../shared/ui/cx";
import { screenAt, type ScreenTable } from "../shared/ui/screenAt";
import {
  ANNOTATIONS_ROUTE,
  CODES_ROUTE,
  EXPLORER_ROUTE,
  FLUSH_PATHS,
  FOOTER_ROUTES,
  GRAPH_ROUTE,
  MAIN_ROUTES,
  PROJECT_PATHS,
  PROJECTS_ROUTE,
  LEGACY_SCAN_PATH,
  CHANGE_ROUTE,
  RULES_ROUTE,
  SETTINGS_ROUTE,
  STATS_ROUTE,
  USAGES_ROUTE,
} from "./consts";
import styles from "./App.module.css";
import { RailLink, TopBar } from "./Frame";
import { UnexpectedError } from "./UnexpectedError";
import { ProjectCrumb } from "./ProjectCrumb";
import { currentPath, navigate } from "./router";

/** The screens of the app by their path; the projects screen answers every other path. */
const SCREENS: ScreenTable = [
  [SETTINGS_ROUTE.path, SettingsScreen],
  [CODES_ROUTE.path, CodesScreen],
  [GRAPH_ROUTE.path, GraphScreen],
  [STATS_ROUTE.path, StatsScreen],
  [CHANGE_ROUTE.path, ChangeScreen],
  [RULES_ROUTE.path, RulesScreen],
  [EXPLORER_ROUTE.path, ExplorerScreen],
  [ANNOTATIONS_ROUTE.path, BoardScreen],
  [USAGES_ROUTE.path, UsagesScreen],
];

/** The screen of the current path. */
function Screen() {
  return <Dynamic component={screenAt(SCREENS, currentPath(), ProjectsScreen)} />;
}

/** The old address of the scan screen shows the tab "Scan" of the open project. */
function redirectLegacyScan() {
  createEffect(() => {
    if (currentPath() === LEGACY_SCAN_PATH) {
      const project = activeProject();
      if (project !== null) {
        showScan(project);
      }
      navigate(PROJECTS_ROUTE.path);
    }
  });
}

/** Sends the user to the projects when a screen of a project is shown while none is open. */
function sendBackWithoutProject() {
  createEffect(() => {
    if (PROJECT_PATHS.includes(currentPath()) && activeProject() === null) {
      navigate(PROJECTS_ROUTE.path);
    }
  });
}

/** The number on the bell follows the project; the drawer belongs to one project. */
function followProjectOnBell() {
  createEffect(() => {
    const project = activeProject();
    hideDrawer();
    if (sessionState().kind === KIND.connected) {
      void refreshCount(project);
    }
  });
}

/** The frame of every screen: top bar, rail, main area and status bar (blueprint 15.2). */
export function App() {
  useSearchShortcut(openSearch);
  redirectLegacyScan();
  sendBackWithoutProject();
  createEffect(() => {
    if (sessionState().kind === KIND.connected) {
      onCleanup(listenToScans());
    }
  });
  followProjectOnBell();
  return (
    <div class={styles.root}>
      <TopBar>
        <ProjectCrumb />
      </TopBar>
      <div class={styles.body}>
        <nav class={styles.rail} aria-label={t(I18N_KEYS.NAV_LABEL)}>
          <For each={MAIN_ROUTES}>{(route) => <RailLink route={route} />}</For>
          <div class={styles.spacer} />
          <For each={FOOTER_ROUTES}>{(route) => <RailLink route={route} />}</For>
        </nav>
        <main class={cx(styles.main, FLUSH_PATHS.includes(currentPath()) && styles.flush)}>
          <Show
            when={sessionState().kind === KIND.connected}
            fallback={<ConnectionScreen state={sessionState()} />}
          >
            <ErrorBoundary
              fallback={(error, reset) => <UnexpectedError error={error} reset={reset} />}
            >
              <Screen />
            </ErrorBoundary>
          </Show>
        </main>
      </div>
      <QuickSearch />
      <AlertsDrawer />
      <StatusBar />
    </div>
  );
}
