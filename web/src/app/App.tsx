import { createEffect, For, Match, onCleanup, Show, Switch } from "solid-js";
import { ConnectionScreen } from "../features/session/ConnectionScreen";
import { GraphScreen } from "../features/graph/GraphScreen";
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
import { KIND } from "../shared/session/consts";
import { sessionState } from "../shared/session/session";
import { cx } from "../shared/ui/cx";
import {
  EXPLORER_ROUTE,
  FLUSH_PATHS,
  FOOTER_ROUTES,
  GRAPH_ROUTE,
  MAIN_ROUTES,
  SCAN_ROUTE,
  SETTINGS_ROUTE,
  USAGES_ROUTE,
} from "./consts";
import styles from "./App.module.css";
import { RailLink, TopBar } from "./Frame";
import { ProjectCrumb } from "./ProjectCrumb";
import { currentPath } from "./router";

/** The screen of the current path. */
function Screen() {
  return (
    <Switch fallback={<ProjectsScreen />}>
      <Match when={currentPath() === SETTINGS_ROUTE.path}>
        <SettingsScreen />
      </Match>
      <Match when={currentPath() === GRAPH_ROUTE.path}>
        <GraphScreen />
      </Match>
      <Match when={currentPath() === SCAN_ROUTE.path}>
        <ScanScreen />
      </Match>
      <Match when={currentPath() === EXPLORER_ROUTE.path}>
        <ExplorerScreen />
      </Match>
      <Match when={currentPath() === USAGES_ROUTE.path}>
        <UsagesScreen />
      </Match>
    </Switch>
  );
}

/** The frame of every screen: top bar, rail, main area and status bar (blueprint 15.2). */
export function App() {
  useSearchShortcut(openSearch);
  createEffect(() => {
    if (sessionState().kind === KIND.connected) {
      onCleanup(listenToScans());
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
      <StatusBar />
    </div>
  );
}
