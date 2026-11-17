import { createEffect, For, onCleanup, Show } from "solid-js";
import { Dynamic } from "solid-js/web";
import { AlertsDrawer } from "../features/alerts/Drawer";
import { hideDrawer, refreshCount } from "../features/alerts/store";
import { BoardScreen } from "../features/annotations/BoardScreen";
import { ConnectionScreen } from "../features/session/ConnectionScreen";
import { GraphScreen } from "../features/graph/GraphScreen";
import { LayersScreen } from "../features/layers/LayersScreen";
import { ExplorerScreen } from "../features/explorer/ExplorerScreen";
import { listenToScans } from "../features/scan/events";
import { ScanScreen } from "../features/scan/ScanScreen";
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
  EXPLORER_ROUTE,
  FLUSH_PATHS,
  FOOTER_ROUTES,
  GRAPH_ROUTE,
  LAYERS_ROUTE,
  MAIN_ROUTES,
  SCAN_ROUTE,
  SETTINGS_ROUTE,
  USAGES_ROUTE,
} from "./consts";
import styles from "./App.module.css";
import { RailLink, TopBar } from "./Frame";
import { ProjectCrumb } from "./ProjectCrumb";
import { currentPath } from "./router";

/** The screens of the app by their path; the projects screen answers every other path. */
const SCREENS: ScreenTable = [
  [SETTINGS_ROUTE.path, SettingsScreen],
  [LAYERS_ROUTE.path, LayersScreen],
  [GRAPH_ROUTE.path, GraphScreen],
  [SCAN_ROUTE.path, ScanScreen],
  [EXPLORER_ROUTE.path, ExplorerScreen],
  [ANNOTATIONS_ROUTE.path, BoardScreen],
  [USAGES_ROUTE.path, UsagesScreen],
];

/** The screen of the current path. */
function Screen() {
  return <Dynamic component={screenAt(SCREENS, currentPath(), ProjectsScreen)} />;
}

/** The frame of every screen: top bar, rail, main area and status bar (blueprint 15.2). */
export function App() {
  useSearchShortcut(openSearch);
  createEffect(() => {
    if (sessionState().kind === KIND.connected) {
      onCleanup(listenToScans());
    }
  });
  createEffect(() => {
    // The number on the bell follows the project; the drawer belongs to one project.
    const project = activeProject();
    hideDrawer();
    if (sessionState().kind === KIND.connected) {
      void refreshCount(project);
    }
  });
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
            <Screen />
          </Show>
        </main>
      </div>
      <QuickSearch />
      <AlertsDrawer />
      <StatusBar />
    </div>
  );
}
