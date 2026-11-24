import { createEffect, For, Match, on, Show, Switch } from "solid-js";
import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { layerOpenAtKey } from "../../shared/keys/layers";
import { openProject } from "../../shared/project/active";
import { cx } from "../../shared/ui/cx";
import { Icon } from "../../shared/ui/Icon";
import { Pill } from "../../shared/ui/Pill";
import { cancelScan } from "../scan/cancel";
import { ScanTab, useCancelKey, useScanOf } from "../scan/ScanTab";
import {
  AT_SIGN,
  PANEL_ID,
  TAB_ID,
  TAB_IN,
  TAB_OUT,
  TAB_STEP,
  COMMIT_LENGTH,
  PART_SEPARATOR,
  PROJECT_TAB,
  PROJECT_TABS,
  RESCAN_ICON,
  SOURCE_KIND,
  type ProjectTab,
} from "./consts";
import { Level2Panel } from "./Level2Panel";
import { Level2Switch } from "./Level2Switch";
import { Overview } from "./Overview";
import { projectTab, showTab } from "./page";
import { projectsScanning, renamingProject, rescanProject } from "./projects";
import { RemoveAction, removeQuestionOpen } from "./RemoveAction";
import { interruptedNow, removingNow } from "./removing";
import { RenameButton, RenameField } from "./RenameForm";
import { ScanPanel } from "./ScanPanel";
import { stateOf } from "./state";
import { ToolsPanel } from "./ToolsPanel";
import type { ProjectRow } from "./types";
import styles from "./ProjectPage.module.css";

/** Path or address, and for a Git project the branch and the short commit. */
function sourceLine(row: ProjectRow): string {
  const kind = t(
    row.kind === SOURCE_KIND.git ? I18N_KEYS.PROJECTS_SOURCE_GIT : I18N_KEYS.PROJECTS_SOURCE_FOLDER,
  );
  const place = [row.branch, row.commit?.slice(0, COMMIT_LENGTH)]
    .filter((part): part is string => !!part)
    .join(AT_SIGN);
  return [kind, row.location, place].filter((part) => part !== "").join(PART_SEPARATOR);
}

/** The icons for rename and remove; a project that is being removed has none, one whose removal
 * was interrupted can only be removed again. */
function Icons(props: { row: ProjectRow; onRemove: (id: number) => Promise<boolean> }) {
  const remove = () => (
    <RemoveAction
      id={props.row.id}
      name={props.row.name}
      onRemove={() => props.onRemove(props.row.id)}
    />
  );
  return (
    <Switch>
      <Match when={interruptedNow(props.row)}>{remove()}</Match>
      <Match when={!removingNow(props.row) && !props.row.running}>
        <RenameButton id={props.row.id} name={props.row.name} />
        {remove()}
      </Match>
    </Switch>
  );
}

/** "Scan again" or, while a scan runs, "Cancel" with the key Esc; and "Open in explorer". */
function Actions(props: { row: ProjectRow; running: boolean }) {
  const busy = () => props.running || props.row.running || projectsScanning().has(props.row.id);
  return (
    <>
      <Show when={!removingNow(props.row)}>
        <Show
          when={props.running}
          fallback={
            <button
              type="button"
              class={styles.button}
              disabled={busy()}
              aria-label={t(I18N_KEYS.PROJECTS_RESCAN, { name: props.row.name })}
              title={t(I18N_KEYS.PROJECTS_RESCAN_TIP)}
              onClick={() => void rescanProject(props.row.id)}
            >
              <Icon name={RESCAN_ICON} /> {t(I18N_KEYS.PROJECTS_LEVEL2_RESCAN)}
            </button>
          }
        >
          <button type="button" class={styles.button} onClick={() => void cancelScan(props.row.id)}>
            {t(I18N_KEYS.SCAN_ACTION_CANCEL)}{" "}
            <kbd class={styles.kbd}>{t(I18N_KEYS.SCAN_CANCEL_KEY)}</kbd>
          </button>
        </Show>
        <button
          type="button"
          class={cx(styles.button, styles.primary)}
          onClick={() => {
            openProject(props.row.id);
            navigate(EXPLORER_ROUTE.path);
          }}
        >
          {t(I18N_KEYS.PROJECTS_OPEN_EXPLORER)}
        </button>
      </Show>
    </>
  );
}

/** The head of the page: name (or the rename field), the one state, the source and the actions. */
function Head(props: {
  row: ProjectRow;
  running: boolean;
  onRemove: (id: number) => Promise<boolean>;
}) {
  const state = () => stateOf(props.row, projectsScanning().has(props.row.id));
  return (
    <header class={styles.head}>
      <div class={styles.title}>
        <div class={styles.nameRow}>
          <Show when={renamingProject()?.id !== props.row.id}>
            <h2 class={styles.name}>{props.row.name}</h2>
          </Show>
          <RenameField id={props.row.id} name={props.row.name} />
          <Pill tone={state().tone}>{state().text}</Pill>
        </div>
        <p class={styles.source}>{sourceLine(props.row)}</p>
      </div>
      <div class={styles.actions}>
        <Actions row={props.row} running={props.running} />
        <Icons row={props.row} onRemove={props.onRemove} />
      </div>
    </header>
  );
}

/** The tabs: `tablist` with the arrow keys moving between them. */
function Tabs() {
  const move = (event: KeyboardEvent, index: number) => {
    const step = TAB_STEP[event.key] ?? 0;
    const next = PROJECT_TABS[(index + step + PROJECT_TABS.length) % PROJECT_TABS.length];
    if (step !== 0 && next !== undefined) {
      event.preventDefault();
      event.stopPropagation();
      showTab(next.id);
      queueMicrotask(() => document.getElementById(`${TAB_ID}${next.id}`)?.focus());
    }
  };
  return (
    <div class={styles.tabs} role="tablist" aria-label={t(I18N_KEYS.PROJECTS_TABS_LABEL)}>
      <For each={PROJECT_TABS}>
        {(tab, index) => (
          <button
            type="button"
            role="tab"
            id={`${TAB_ID}${tab.id}`}
            aria-controls={`${PANEL_ID}${tab.id}`}
            aria-selected={projectTab() === tab.id}
            tabindex={projectTab() === tab.id ? TAB_IN : TAB_OUT}
            class={cx(styles.tab, projectTab() === tab.id && styles.on)}
            onClick={() => showTab(tab.id)}
            onKeyDown={(event) => move(event, index())}
          >
            {t(tab.label)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The content of the shown tab. */
function Panel(props: { row: ProjectRow; tab: ProjectTab }) {
  return (
    <div role="tabpanel" id={`${PANEL_ID}${props.tab}`} aria-labelledby={`${TAB_ID}${props.tab}`}>
      <Switch>
        <Match when={props.tab === PROJECT_TAB.overview}>
          <Overview row={props.row} />
        </Match>
        <Match when={props.tab === PROJECT_TAB.scan}>
          <ScanTab row={props.row} />
        </Match>
        <Match when={props.tab === PROJECT_TAB.level2}>
          <div class={styles.stack}>
            <Level2Switch row={props.row} />
            <Level2Panel row={props.row} />
            <ScanPanel row={props.row} />
          </div>
        </Match>
        <Match when={props.tab === PROJECT_TAB.tools}>
          <ToolsPanel row={props.row} />
        </Match>
      </Switch>
    </div>
  );
}

/** The page of the selected project (mock 23): head, tabs and the content of the tab. The key
 * Esc cancels a scan only when no input or question is open. */
export function ProjectPage(props: {
  row: ProjectRow;
  onRemove: (id: number) => Promise<boolean>;
}) {
  const { running } = useScanOf(() => props.row);
  useCancelKey(
    () => props.row.id,
    running,
    () => renamingProject() === null && !removeQuestionOpen() && !layerOpenAtKey(),
  );
  // A scan that starts brings the page to its tab; the tab stays free afterwards.
  createEffect(
    on(running, (now, was) => {
      if (now && !was) {
        showTab(PROJECT_TAB.scan);
      }
    }),
  );
  return (
    <section class={styles.page} aria-label={props.row.name}>
      <Head row={props.row} running={running()} onRemove={props.onRemove} />
      <div class={styles.card}>
        <Tabs />
        <div class={styles.body}>
          <Panel row={props.row} tab={projectTab()} />
        </div>
      </div>
    </section>
  );
}
