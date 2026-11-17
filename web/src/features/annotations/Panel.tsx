import { createMemo, For, Match, Show, Switch, type JSX } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { moveAlongTabs } from "../../shared/keys/tabs";
import { cx } from "../../shared/ui/cx";
import { CriticalTab } from "./CriticalTab";
import {
  CLOSE_MARK,
  KIND,
  PANEL_ID,
  ROLE,
  SHIELD,
  TAB,
  TAB_ID_PREFIX,
  TAB_STOP,
  TABS,
  type TabId,
} from "./consts";
import { countersOf } from "./join";
import { NoteIcon, Shield, TaskIcon } from "./Marks";
import { NotesTab } from "./NotesTab";
import { activeTab, clearNotice, panelNotice, selectTab } from "./panel";
import { annotationIndex } from "./store";
import { TasksTab } from "./TasksTab";
import type { Counters, Target } from "./types";
import { useSymbolItems } from "./useSymbolItems";
import styles from "./Panel.module.css";

/** The name of a tab that has one (the tab Critical is a shield). */
const TAB_NAME: Partial<Record<TabId, MessageKey>> = {
  [TAB.info]: I18N_KEYS.ANNOTATIONS_TAB_INFO,
  [TAB.notes]: I18N_KEYS.ANNOTATIONS_TAB_NOTES,
  [TAB.tasks]: I18N_KEYS.ANNOTATIONS_TAB_TASKS,
};

/** The counter of a tab: the notes, or the tasks done of all; nothing while there are none. */
function Counter(props: { id: TabId; counters: Counters | undefined }) {
  return (
    <Switch>
      <Match when={props.id === TAB.notes && (props.counters?.notes ?? 0) > 0}>
        <b class={styles.count}>{props.counters?.notes}</b>
      </Match>
      <Match when={props.id === TAB.tasks && (props.counters?.tasksTotal ?? 0) > 0}>
        <b class={cx(styles.count, styles.warn)}>
          {props.counters?.tasksDone}/{props.counters?.tasksTotal}
        </b>
      </Match>
    </Switch>
  );
}

/** What a tab shows in front of its name. */
function TabIcon(props: { id: TabId; marked: boolean }) {
  return (
    <Switch>
      <Match when={props.id === TAB.critical}>
        <span class={cx(styles.shield, props.marked && styles.marked)}>
          <Shield size={SHIELD.tab} />
        </span>
      </Match>
      <Match when={props.id === TAB.notes}>
        <NoteIcon size={SHIELD.tab} />
      </Match>
      <Match when={props.id === TAB.tasks}>
        <TaskIcon size={SHIELD.tab} />
      </Match>
    </Switch>
  );
}

function TabButton(props: { id: TabId; counters: Counters | undefined }) {
  const name = () => TAB_NAME[props.id];
  const label = () => t(name() ?? I18N_KEYS.ANNOTATIONS_TAB_CRITICAL);
  const on = () => activeTab() === props.id;
  return (
    <button
      type="button"
      role="tab"
      id={`${TAB_ID_PREFIX}${props.id}`}
      class={cx(styles.tab, on() && styles.on)}
      aria-selected={on()}
      aria-controls={PANEL_ID}
      aria-label={name() === undefined ? label() : undefined}
      title={label()}
      tabIndex={on() ? TAB_STOP.reachable : TAB_STOP.skipped}
      onClick={() => selectTab(props.id)}
    >
      <TabIcon id={props.id} marked={props.counters?.critical ?? false} />
      <Show when={name()}>
        <span class={styles.name}>{label()}</span>
      </Show>
      <Counter id={props.id} counters={props.counters} />
    </button>
  );
}

/** The arrows move between the tabs (the pattern of a tab list). */
function onTabKey(event: KeyboardEvent): void {
  moveAlongTabs(event, TABS, activeTab(), selectTab, TAB_ID_PREFIX);
}

/** The tab list of the panel (mock 14). It sits in a container whose width decides whether an
 * inactive tab shows its name or only its symbol and counter (blueprint 15.4). */
function TabBar(props: { counters: Counters | undefined }) {
  return (
    <div class={styles.wrap}>
      <div
        class={styles.tabs}
        role="tablist"
        aria-label={t(I18N_KEYS.ANNOTATIONS_TABS_LABEL)}
        onKeyDown={onTabKey}
      >
        <For each={TABS}>{(id) => <TabButton id={id} counters={props.counters} />}</For>
      </div>
    </div>
  );
}

/** The line under the tabs: what a key or a button did, or why it did not work. */
function Notice() {
  return (
    <Show when={panelNotice()}>
      {(notice) => (
        <p class={styles.notice} role={notice().failed ? ROLE.alert : ROLE.status}>
          <span>{notice().text}</span>
          <button
            type="button"
            class={styles.close}
            aria-label={t(I18N_KEYS.ANNOTATIONS_CANCEL)}
            onClick={clearNotice}
          >
            {CLOSE_MARK}
          </button>
        </p>
      )}
    </Show>
  );
}

/** The tab Critical, Notes or Tasks for the shown symbol. */
function TabContent(props: { project: number; target: Target; marked: boolean }) {
  const items = useSymbolItems(
    () => props.project,
    () => props.target,
  );
  return (
    <Show when={items.note()} fallback={<Tab {...props} items={items} />}>
      {(note) => <p class={styles.note}>{note()}</p>}
    </Show>
  );
}

function Tab(props: {
  project: number;
  target: Target;
  marked: boolean;
  items: ReturnType<typeof useSymbolItems>;
}) {
  return (
    <Switch>
      <Match when={activeTab() === TAB.critical}>
        <CriticalTab
          project={props.project}
          target={props.target}
          marked={props.marked}
          mark={props.items.ofKind(KIND.critical)[0]}
        />
      </Match>
      <Match when={activeTab() === TAB.notes}>
        <NotesTab
          project={props.project}
          target={props.target}
          notes={props.items.ofKind(KIND.note)}
        />
      </Match>
      <Match when={activeTab() === TAB.tasks}>
        <TasksTab
          project={props.project}
          target={props.target}
          tasks={props.items.ofKind(KIND.task)}
        />
      </Match>
    </Switch>
  );
}

/** The side panel of the explorer (mock 14, `Annotations.dc.html`): the tabs Info, Critical, Notes
 * and Tasks for one symbol. `info` is the content of the first tab. */
export function AnnotationPanel(props: { project: number; target: Target; info: JSX.Element }) {
  const counters = createMemo(() => countersOf(annotationIndex(), props.target));
  return (
    <div class={styles.panel}>
      <TabBar counters={counters()} />
      <Notice />
      <div
        id={PANEL_ID}
        role="tabpanel"
        class={styles.body}
        aria-labelledby={`${TAB_ID_PREFIX}${activeTab()}`}
      >
        <Show when={activeTab() !== TAB.info} fallback={props.info}>
          <TabContent
            project={props.project}
            target={props.target}
            marked={counters()?.critical ?? false}
          />
        </Show>
      </div>
    </div>
  );
}
