import { createEffect, For, Match, on, onCleanup, Show, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { moveAlongTabs } from "../../shared/keys/tabs";
import { activeProject } from "../../shared/project/active";
import { NoticeBox } from "../../shared/ui/NoticeBox";
import { cx } from "../../shared/ui/cx";
import {
  changeAnswer,
  changeFailure,
  changeMetrics,
  changeMetricsFailure,
  changeMetricsMetric,
  changeReading,
  changeScans,
  changeTab,
  enterChange,
  loadChange,
  reloadForFilter,
  resetChange,
  showChangeTab,
} from "./change";
import { CompareTab } from "./CompareTab";
import { CopyMarkdown } from "./CopyMarkdown";
import {
  COMMAND,
  METRIC_ALL,
  FILTER_DELAY_MS,
  MIN_SCANS,
  PANEL_ID,
  ROLE,
  SCAN_ARROW,
  TAB,
  TAB_ID_PREFIX,
  TAB_STOP,
  TABS,
} from "./consts";
import { changeFilter } from "./filter";
import { ImpactTab } from "./ImpactTab";
import { MetricsTab } from "./MetricsTab";
import { chipText } from "./scans";
import type { ChangeAnswer } from "./types";
import styles from "./ChangeScreen.module.css";

const TAB_IDS = TABS.map((tab) => tab.id);

/** The two scans that were compared, as chips: base, an arrow and target. */
function Compared(props: { answer: ChangeAnswer }) {
  return (
    <>
      <span class={styles.sub}>{t(I18N_KEYS.CHANGE_VIEW_COMPARED)}</span>
      <span class={styles.chip}>{chipText(props.answer.base)}</span>
      <span class={styles.arrow} aria-hidden={true}>
        {SCAN_ARROW}
      </span>
      <span class={styles.chip}>{chipText(props.answer.target)}</span>
    </>
  );
}

/** The tab list: the arrow keys select the next or the previous view. */
function Tabs() {
  return (
    <div
      class={styles.tabs}
      role={ROLE.tablist}
      aria-label={t(I18N_KEYS.CHANGE_VIEW_TABS_LABEL)}
      onKeyDown={(event) =>
        moveAlongTabs(event, TAB_IDS, changeTab(), showChangeTab, TAB_ID_PREFIX)
      }
    >
      <For each={TABS}>
        {(tab) => (
          <button
            type="button"
            role={ROLE.tab}
            id={`${TAB_ID_PREFIX}${tab.id}`}
            class={cx(styles.tab, changeTab() === tab.id && styles.on)}
            aria-selected={changeTab() === tab.id}
            aria-controls={PANEL_ID}
            tabindex={changeTab() === tab.id ? TAB_STOP.on : TAB_STOP.off}
            onClick={() => showChangeTab(tab.id)}
          >
            {t(tab.label)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The failure of the read, with its code and the button that copies the details. */
function Failure() {
  return (
    <Show when={changeFailure()}>
      {(failure) => (
        <NoticeBox
          class={styles.alert}
          alert
          text={
            failure().code === null
              ? t(I18N_KEYS.CHANGE_VIEW_LOAD_FAILED)
              : errorText(failure().code as ErrorCode)
          }
          code={failure().code}
        />
      )}
    </Show>
  );
}

/** The panel of the view that is shown. */
function Views(props: { project: number }) {
  return (
    <div
      role={ROLE.tabpanel}
      id={PANEL_ID}
      class={styles.panel}
      aria-labelledby={`${TAB_ID_PREFIX}${changeTab()}`}
    >
      <Switch>
        <Match when={changeTab() === TAB.impact}>
          <Show when={changeAnswer()}>{(answer) => <ImpactTab answer={answer()} />}</Show>
        </Match>
        <Match when={changeTab() === TAB.compare}>
          <CompareTab project={props.project} answer={changeAnswer()} />
        </Match>
        <Match when={changeTab() === TAB.metrics}>
          <MetricsTab project={props.project} answer={changeAnswer()} />
        </Match>
      </Switch>
    </div>
  );
}

/** What the screen shows for a project: a note while the scans are read or if there are too few,
 * otherwise the views. */
function Content(props: { project: number }) {
  const tooFew = () => (changeScans()?.scans.length ?? MIN_SCANS) < MIN_SCANS;
  return (
    <>
      <Failure />
      <Show when={changeReading() && changeScans() === null}>
        <p class={styles.status} role="status">
          {t(I18N_KEYS.CHANGE_VIEW_READING)}
        </p>
      </Show>
      <Show
        when={!tooFew()}
        fallback={<p class={styles.status}>{t(I18N_KEYS.CHANGE_VIEW_TOO_FEW)}</p>}
      >
        <Show when={changeScans() !== null}>
          <Tabs />
          <Views project={props.project} />
        </Show>
      </Show>
    </>
  );
}

/** The button of the view "Metrics": always there, greyed out with the reason while there is no
 * answer. It copies the level and the metric that are chosen, so that the text is what the lists
 * show. */
function MetricsCopy(props: { project: number }) {
  const unavailable = () =>
    changeMetrics() !== null
      ? undefined
      : t(
          changeMetricsFailure() === null
            ? I18N_KEYS.CHANGE_METRICS_COPY_WAITING
            : I18N_KEYS.CHANGE_METRICS_COPY_FAILED,
        );
  return (
    <CopyMarkdown
      command={COMMAND.metricsMarkdown}
      unavailable={unavailable()}
      input={() => ({
        project: props.project,
        base: changeMetrics()?.base.snapshot,
        target: changeMetrics()?.target.snapshot,
        level: changeMetrics()?.level,
        ...(changeMetricsMetric() === METRIC_ALL ? {} : { metrics: [changeMetricsMetric()] }),
      })}
    />
  );
}

/** The button "Copy as Markdown" of the view that is shown: the impact or the metrics. */
function CopyButtons() {
  return (
    <>
      <Show when={changeTab() === TAB.impact ? changeAnswer() : null}>
        {(answer) => (
          <Show when={activeProject()}>
            {(project) => (
              <CopyMarkdown
                command={COMMAND.markdown}
                input={() => ({
                  project: project(),
                  base: answer().base.snapshot,
                  target: answer().target.snapshot,
                })}
              />
            )}
          </Show>
        )}
      </Show>
      <Show when={changeTab() === TAB.metrics}>
        <Show when={activeProject()}>{(project) => <MetricsCopy project={project()} />}</Show>
      </Show>
    </>
  );
}

/** Reads the comparison again a moment after the filter changed, so typing in the search does not
 * ask the server for every letter. */
function filterReads(): void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(
    on(
      changeFilter,
      () => {
        clearTimeout(timer);
        const project = activeProject();
        if (project !== null) {
          timer = setTimeout(() => reloadForFilter(project), FILTER_DELAY_MS);
        }
      },
      { defer: true },
    ),
  );
  onCleanup(() => clearTimeout(timer));
}

/** The screen "Change": what the difference between two scans of the project hits (ADR 0097,
 * mocks 29 and erw-auswirkung). One call, `scans.impact`, serves both views. */
export function ChangeScreen() {
  createEffect(
    on(activeProject, (id) => {
      const wanted = enterChange();
      if (id !== null) {
        void loadChange(id, wanted);
      }
    }),
  );
  onCleanup(resetChange);
  filterReads();
  return (
    <section class={styles.screen}>
      <header class={styles.head}>
        <h1>{t(I18N_KEYS.CHANGE_VIEW_TITLE)}</h1>
        <span class={styles.sub}>{t(I18N_KEYS.CHANGE_VIEW_SUBTITLE)}</span>
        <span class={styles.spacer} />
        <Show when={changeTab() === TAB.impact ? changeAnswer() : null}>
          {(answer) => <Compared answer={answer()} />}
        </Show>
        <CopyButtons />
      </header>
      <Show
        when={activeProject()}
        fallback={<p class={styles.status}>{t(I18N_KEYS.CHANGE_VIEW_NO_PROJECT)}</p>}
      >
        {(project) => <Content project={project()} />}
      </Show>
    </section>
  );
}
