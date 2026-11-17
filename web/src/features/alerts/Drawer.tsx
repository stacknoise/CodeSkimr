import { createMemo, createSignal, For, Match, onCleanup, onMount, Show, Switch } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { moveAlongTabs } from "../../shared/keys/tabs";
import { cx } from "../../shared/ui/cx";
import { AlertItem } from "./AlertItem";
import {
  BELL_ID,
  CLOSE_MARK,
  DRAWER_ID,
  KEY,
  LOADING,
  PANEL_ID,
  TAB_ID_PREFIX,
  TAB_KINDS,
  TAB_NAME,
  TAB_STOP,
  TABS,
  TAB,
  type TabId,
} from "./consts";
import {
  actionFailed,
  activeTab,
  alertsLoading,
  alertsOfTab,
  dismissKinds,
  doneAlerts,
  drawerShown,
  hideDrawer,
  openAlertCount,
  openAlerts,
  selectTab,
} from "./store";
import styles from "./Drawer.module.css";

/** The alerts of a tab. */
const tabAlerts = (id: TabId) => alertsOfTab(id, openAlerts(), doneAlerts());

/** One tab of the drawer, with the number of its alerts. */
function TabButton(props: { id: TabId }) {
  const on = () => activeTab() === props.id;
  return (
    <button
      type="button"
      role="tab"
      id={`${TAB_ID_PREFIX}${props.id}`}
      class={cx(styles.tab, on() && styles.on)}
      aria-selected={on()}
      aria-controls={PANEL_ID}
      tabIndex={on() ? TAB_STOP.reachable : TAB_STOP.skipped}
      onClick={() => selectTab(props.id)}
    >
      <span>{t(TAB_NAME[props.id])}</span>
      <b class={styles.count}>{tabAlerts(props.id).length}</b>
    </button>
  );
}

/** The arrows move between the tabs (the pattern of a tab list). */
function onTabKey(event: KeyboardEvent): void {
  moveAlongTabs(event, TABS, activeTab(), selectTab, TAB_ID_PREFIX);
}

/** The button that dismisses every alert of the tab, after a question: a bulk dismissal is not
 * done by a single click. */
function DismissAll(props: { project: number }) {
  const [asking, setAsking] = createSignal(false);
  const count = () => tabAlerts(activeTab()).length;
  const run = () => {
    setAsking(false);
    void dismissKinds(props.project, TAB_KINDS[activeTab()]);
  };
  return (
    <Show when={activeTab() !== TAB.done && count() > 0}>
      <Show
        when={asking()}
        fallback={
          <button type="button" class={styles.btn} onClick={() => setAsking(true)}>
            {t(I18N_KEYS.ALERTS_DISMISS_ALL)}
          </button>
        }
      >
        <span
          class={styles.ask}
          role="group"
          aria-label={t(I18N_KEYS.ALERTS_DISMISS_ALL_ASK, { count: count() })}
        >
          <span>{t(I18N_KEYS.ALERTS_DISMISS_ALL_ASK, { count: count() })}</span>
          <button type="button" class={styles.btn} onClick={run}>
            {t(I18N_KEYS.ALERTS_DISMISS_ALL_YES)}
          </button>
          <button type="button" class={styles.btn} onClick={() => setAsking(false)}>
            {t(I18N_KEYS.ANNOTATIONS_CANCEL)}
          </button>
        </span>
      </Show>
    </Show>
  );
}

/** The alerts of the shown tab, or what is going on instead. */
function List(props: { project: number }) {
  const items = createMemo(() => tabAlerts(activeTab()));
  return (
    <Switch>
      <Match when={alertsLoading() === LOADING.failed}>
        <p class={styles.note} role="alert">
          {t(I18N_KEYS.ALERTS_FAILED)}
        </p>
      </Match>
      <Match when={alertsLoading() === LOADING.loading}>
        <p class={styles.note}>{t(I18N_KEYS.ALERTS_LOADING)}</p>
      </Match>
      <Match when={items().length === 0}>
        <p class={styles.note}>{t(I18N_KEYS.ALERTS_EMPTY)}</p>
      </Match>
      <Match when={true}>
        <ul class={styles.list}>
          <For each={items()}>{(alert) => <AlertItem project={props.project} alert={alert} />}</For>
        </ul>
      </Match>
    </Switch>
  );
}

/** The head of the drawer: the title, the number of open alerts, "dismiss all" and the cross. */
function Head(props: { project: number }) {
  return (
    <header class={styles.head}>
      <b class={styles.title}>{t(I18N_KEYS.ALERTS_TITLE)}</b>
      <span class={styles.chip}>
        {t(I18N_KEYS.ALERTS_OPEN_CHIP, { count: openAlertCount() ?? 0 })}
      </span>
      <span class={styles.fill} />
      <DismissAll project={props.project} />
      <button
        type="button"
        class={styles.x}
        aria-label={t(I18N_KEYS.ALERTS_CLOSE)}
        onClick={hideDrawer}
      >
        {CLOSE_MARK}
      </button>
    </header>
  );
}

/** The tab list of the drawer. */
function Tabs() {
  return (
    <div
      class={styles.tabs}
      role="tablist"
      aria-label={t(I18N_KEYS.ALERTS_TABS_LABEL)}
      onKeyDown={onTabKey}
    >
      <For each={TABS}>{(id) => <TabButton id={id} />}</For>
    </div>
  );
}

/** The panel with the head, the tabs and the list. Focus moves into it when it opens, and back to
 * the bell when it closes; Escape closes it. */
function Panel(props: { project: number }) {
  let root: HTMLElement | undefined;
  onMount(() => root?.focus());
  onCleanup(() => document.getElementById(BELL_ID)?.focus());
  return (
    <aside
      id={DRAWER_ID}
      ref={(element) => (root = element)}
      class={styles.drawer}
      role="dialog"
      aria-label={t(I18N_KEYS.ALERTS_TITLE)}
      tabIndex={TAB_STOP.skipped}
      onKeyDown={(event) => event.key === KEY.close && hideDrawer()}
    >
      <Head project={props.project} />
      <Show when={actionFailed()}>
        <p class={styles.note} role="alert">
          {t(I18N_KEYS.ALERTS_ACTION_FAILED)}
        </p>
      </Show>
      <Tabs />
      <div
        id={PANEL_ID}
        role="tabpanel"
        class={styles.body}
        aria-labelledby={`${TAB_ID_PREFIX}${activeTab()}`}
      >
        <List project={props.project} />
        <p class={styles.baseline}>{t(I18N_KEYS.ALERTS_BASELINE)}</p>
      </div>
      <p class={styles.footer}>{t(I18N_KEYS.ALERTS_FOOTER)}</p>
    </aside>
  );
}

/** The drawer of the alerts (mock 15, `Alerts.dc.html`): it slides in from the right over a dimmed
 * screen. A click on the dimmed part closes it. */
export function AlertsDrawer() {
  return (
    <Show when={drawerShown() ? activeProject() : null}>
      {(project) => (
        <>
          <div class={styles.dim} onClick={hideDrawer} aria-hidden={true} />
          <Panel project={project()} />
        </>
      )}
    </Show>
  );
}
