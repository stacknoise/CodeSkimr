import { For, Match, Switch } from "solid-js";
import { ExplorerScreen } from "../features/explorer/ExplorerScreen";
import { GraphScreen } from "../features/graph/GraphScreen";
import { QuickSearch } from "../features/search/QuickSearch";
import { useSearchShortcut } from "../features/search/keys";
import { openSearch } from "../features/search/search";
import { UsagesScreen } from "../features/usages/UsagesScreen";
import { t, currentLanguage } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { cx } from "../shared/ui/cx";
import { EXPLORER_ROUTE, FLUSH_PATHS, GRAPH_ROUTE, USAGES_ROUTE, type Route } from "../app/consts";
import appStyles from "../app/App.module.css";
import { BANNER_SEPARATOR, MS_PER_SECOND, SHORT_COMMIT } from "./consts";
import { RailLink, TopBar } from "../app/Frame";
import { currentPath } from "./router";
import type { ExportData } from "./types";
import styles from "./ExportApp.module.css";

/** The screens of the export: the ones that read what a scan found. */
const ROUTES: Route[] = [EXPLORER_ROUTE, GRAPH_ROUTE];

function Screen() {
  return (
    <Switch fallback={<ExplorerScreen />}>
      <Match when={currentPath() === GRAPH_ROUTE.path}>
        <GraphScreen />
      </Match>
      <Match when={currentPath() === USAGES_ROUTE.path}>
        <UsagesScreen />
      </Match>
    </Switch>
  );
}

/** How many direct usages the export holds, split by whether level 2 verified them. */
function usageCounts(data: ExportData): { verified: number; unchecked: number } {
  let verified = 0;
  let unchecked = 0;
  for (const stored of Object.values(data.usages)) {
    verified += stored.answer.summary.verified;
    unchecked += stored.answer.summary.unchecked;
  }
  return { verified, unchecked };
}

/** What the strip under the top bar says: which scan this is, and how far the usages can be
 * trusted. A page that is read months later has to say it by itself. */
function Banner(props: { data: ExportData }) {
  const { project } = props.data;
  const counts = () => usageCounts(props.data);
  const date = () =>
    new Date(project.created * MS_PER_SECOND).toLocaleDateString(currentLanguage());
  return (
    <div class={styles.banner} role="note">
      <b>
        {t(I18N_KEYS.EXPORT_BANNER, {
          project: project.name,
          snapshot: props.data.snapshot,
          date: date(),
        })}
        {project.commit
          ? `${BANNER_SEPARATOR}${t(I18N_KEYS.EXPORT_BANNER_COMMIT, { commit: project.commit.slice(0, SHORT_COMMIT) })}`
          : null}
      </b>
      <span>{t(I18N_KEYS.EXPORT_USAGES_STATE, counts())}</span>
      <span>{t(I18N_KEYS.EXPORT_LIMITS)}</span>
    </div>
  );
}

/** The frame of the export: the top bar and rail of the app with the screens that need no
 * server. */
export function ExportApp(props: { data: ExportData }) {
  useSearchShortcut(openSearch);
  return (
    <div class={appStyles.root}>
      <TopBar workspace={t(I18N_KEYS.EXPORT_NAV_LABEL)} name={props.data.project.name} />
      <Banner data={props.data} />
      <div class={appStyles.body}>
        <nav class={appStyles.rail} aria-label={t(I18N_KEYS.NAV_LABEL)}>
          <For each={ROUTES}>{(route) => <RailLink route={route} />}</For>
        </nav>
        <main class={cx(appStyles.main, FLUSH_PATHS.includes(currentPath()) && appStyles.flush)}>
          <Screen />
        </main>
      </div>
      <QuickSearch />
    </div>
  );
}
