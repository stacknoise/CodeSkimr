import { For, Match, Show, Switch } from "solid-js";
import { ConnectionScreen } from "../features/session/ConnectionScreen";
import { ExplorerScreen } from "../features/explorer/ExplorerScreen";
import { ProjectsScreen } from "../features/projects/ProjectsScreen";
import { QuickSearch } from "../features/search/QuickSearch";
import { useSearchShortcut } from "../features/search/keys";
import { openSearch } from "../features/search/search";
import { SettingsScreen } from "../features/settings/SettingsScreen";
import { t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { KIND } from "../shared/session/consts";
import { sessionState } from "../shared/session/session";
import { cx } from "../shared/ui/cx";
import { Icon } from "../shared/ui/Icon";
import {
  CURRENT_PAGE,
  EXPLORER_ROUTE,
  FOOTER_ROUTES,
  LOGO_LETTER,
  STATUS_DOT,
  MAIN_ROUTES,
  SETTINGS_ROUTE,
  type Route,
} from "./consts";
import styles from "./App.module.css";
import { currentPath, navigate } from "./router";

function RailLink(props: { route: Route }) {
  return (
    <a
      href={props.route.path}
      class={cx(styles.railButton, currentPath() === props.route.path && styles.on)}
      aria-label={t(props.route.label)}
      aria-current={currentPath() === props.route.path ? CURRENT_PAGE : undefined}
      onClick={(event) => {
        event.preventDefault();
        navigate(props.route.path);
      }}
    >
      <Icon name={props.route.icon} />
    </a>
  );
}

function StatusText() {
  return (
    <Switch>
      <Match when={sessionState().kind === KIND.connected}>
        <span class={styles.ok}>
          {STATUS_DOT} {t(I18N_KEYS.STATUS_CONNECTED)}
        </span>
      </Match>
      <Match when={sessionState().kind === KIND.connecting}>
        <span>{t(I18N_KEYS.STATUS_CONNECTING)}</span>
      </Match>
      <Match when={true}>
        <span>{t(I18N_KEYS.STATUS_DISCONNECTED)}</span>
      </Match>
    </Switch>
  );
}

/** The screen of the current path. */
function Screen() {
  return (
    <Switch fallback={<ProjectsScreen />}>
      <Match when={currentPath() === SETTINGS_ROUTE.path}>
        <SettingsScreen />
      </Match>
      <Match when={currentPath() === EXPLORER_ROUTE.path}>
        <ExplorerScreen />
      </Match>
    </Switch>
  );
}

/** The search field of the top bar: it opens the quick search (blueprint 15.2). */
function SearchField() {
  return (
    <button type="button" class={styles.search} onClick={openSearch}>
      <span>{t(I18N_KEYS.APP_SEARCH_PLACEHOLDER)}</span>
      <kbd>{t(I18N_KEYS.APP_SEARCH_SHORTCUT)}</kbd>
    </button>
  );
}

/** The frame of every screen: top bar, rail, main area and status bar (blueprint 15.2). */
export function App() {
  useSearchShortcut(openSearch);
  return (
    <div class={styles.root}>
      <header class={styles.top}>
        <div class={styles.brand}>
          <span class={styles.logo}>{LOGO_LETTER}</span>
          <span class={styles.name}>{t(I18N_KEYS.APP_NAME)}</span>
        </div>
        <SearchField />
        <div class={styles.workspace}>
          <span>{t(I18N_KEYS.APP_WORKSPACE)}</span>
          <b>{t(I18N_KEYS.APP_WORKSPACE_LOCAL)}</b>
        </div>
      </header>
      <div class={styles.body}>
        <nav class={styles.rail} aria-label={t(I18N_KEYS.NAV_LABEL)}>
          <For each={MAIN_ROUTES}>{(route) => <RailLink route={route} />}</For>
          <div class={styles.spacer} />
          <For each={FOOTER_ROUTES}>{(route) => <RailLink route={route} />}</For>
        </nav>
        <main class={cx(styles.main, currentPath() === EXPLORER_ROUTE.path && styles.flush)}>
          <Show
            when={sessionState().kind === KIND.connected}
            fallback={<ConnectionScreen state={sessionState()} />}
          >
            <Screen />
          </Show>
        </main>
      </div>
      <QuickSearch />
      <footer class={styles.status}>
        <StatusText />
      </footer>
    </div>
  );
}
