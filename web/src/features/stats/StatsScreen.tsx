import { createEffect, createMemo, For, Match, on, onMount, Show, Switch } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { moveAlongTabs } from "../../shared/keys/tabs";
import { activeProject } from "../../shared/project/active";
import { cx } from "../../shared/ui/cx";
import { NoticeBox } from "../../shared/ui/NoticeBox";
import { SlowLoad } from "../../shared/ui/SlowLoad";
import { count } from "../projects/format";
import { SOURCE_KIND } from "../projects/consts";
import { projects, refreshProjects } from "../projects/projects";
import { Histogram, Hotspots } from "./Charts";
import {
  COUNTER_TONE,
  KEY,
  PANEL_ID,
  ROLE,
  STEP,
  TAB,
  TAB_ID_PREFIX,
  TAB_STOP,
  TABS,
} from "./consts";
import { Filters } from "./Filters";
import { OverviewView, Tile } from "./OverviewView";
import { RankingTable } from "./RankingTable";
import { SidePanel } from "./SidePanel";
import {
  currentRow,
  enterProject,
  loadStats,
  moveSelection,
  openRow,
  openRowUsages,
  selectedRow,
  selectRow,
  setStatsTests,
  showTab,
  statsAssessment,
  statsFailed,
  statsKinds,
  statsLanguage,
  statsOrder,
  statsOverview,
  statsRanking,
  statsTab,
  statsTests,
} from "./stats";
import { keepFailure } from "./keep";
import { LeftOutList } from "./LeftOut";
import type { RankingAnswer, RankingName, Row, Tab, UnusedCounts } from "./types";
import { HatchPattern } from "./UsageBar";
import styles from "./StatsScreen.module.css";

const TAB_IDS = TABS.map((tab) => tab.id);

function unusedTotal(counts: UnusedCounts | undefined): number | null {
  return counts === undefined ? null : counts.likely + counts.public + counts.approximate;
}

function Tabs() {
  const unused = () => unusedTotal(statsOverview()?.unused);
  return (
    <div
      class={styles.tabs}
      role={ROLE.tablist}
      aria-label={t(I18N_KEYS.STATS_TABS_LABEL)}
      onKeyDown={(event) => moveAlongTabs(event, TAB_IDS, statsTab(), showTab, TAB_ID_PREFIX)}
    >
      <For each={TABS}>
        {(tab) => (
          <button
            type="button"
            id={`${TAB_ID_PREFIX}${tab.id}`}
            role={ROLE.tab}
            class={cx(styles.tab, statsTab() === tab.id && styles.on)}
            aria-selected={statsTab() === tab.id}
            aria-controls={PANEL_ID}
            tabIndex={statsTab() === tab.id ? TAB_STOP.on : TAB_STOP.off}
            onClick={() => showTab(tab.id)}
          >
            {t(tab.label)}
            <Show when={tab.id === TAB.unused && unused() !== null}>
              <span class={styles.counter}>{count(unused() ?? 0, currentLanguage())}</span>
            </Show>
          </button>
        )}
      </For>
    </div>
  );
}

function Counter(props: { title: MessageKey; detail: MessageKey; value: number; tone: string }) {
  return (
    <Tile
      title={props.title}
      value={props.value}
      detail={t(props.detail)}
      tone={styles[props.tone as keyof typeof styles]}
    />
  );
}

/** What "no usage found" means, and how the rows split up by assessment. */
function UnusedIntro(props: { counts: UnusedCounts }) {
  return (
    <>
      <p class={styles.notice} role={ROLE.status}>
        <strong>{t(I18N_KEYS.STATS_UNUSED_NOTE_TITLE)}</strong> {t(I18N_KEYS.STATS_UNUSED_NOTE)}
      </p>
      <div class={styles.counters}>
        <Counter
          title={I18N_KEYS.STATS_UNUSED_LIKELY}
          detail={I18N_KEYS.STATS_UNUSED_LIKELY_DETAIL}
          value={props.counts.likely}
          tone={COUNTER_TONE.good}
        />
        <Counter
          title={I18N_KEYS.STATS_UNUSED_PUBLIC}
          detail={I18N_KEYS.STATS_UNUSED_PUBLIC_DETAIL}
          value={props.counts.public}
          tone={COUNTER_TONE.warn}
        />
        <Counter
          title={I18N_KEYS.STATS_UNUSED_APPROXIMATE}
          detail={I18N_KEYS.STATS_UNUSED_APPROXIMATE_DETAIL}
          value={props.counts.approximate}
          tone={COUNTER_TONE.bad}
        />
        <Counter
          title={I18N_KEYS.STATS_UNUSED_TESTS}
          detail={I18N_KEYS.STATS_UNUSED_TESTS_DETAIL}
          value={props.counts.tests_hidden}
          tone={COUNTER_TONE.plain}
        />
      </div>
    </>
  );
}

/** The pictures of the complexity tab: the histogram of all functions and the hotspots among the
 * rows of the ranking. */
function ComplexityCharts(props: { rows: Row[] }) {
  return (
    <Show when={statsOverview()}>
      {(data) => (
        <div class={styles.charts}>
          <Histogram bars={data().complexity.histogram} limit={data().complexity.limit} />
          <Hotspots
            rows={props.rows}
            limit={data().complexity.limit}
            selected={selectedRow()}
            onSelect={(row) => selectRow(row.id)}
          />
        </div>
      )}
    </Show>
  );
}

/** What went wrong when a symbol was kept or an exception lifted. */
function KeepFailure() {
  return (
    <Show when={keepFailure()}>
      {(code) => <p role="alert">{t(I18N_KEYS.STATS_KEEP_FAILED, { code: code() })}</p>}
    </Show>
  );
}

function TableCard(props: {
  ranking: RankingName;
  answer: RankingAnswer;
  project: number;
  editable: boolean;
}) {
  return (
    <div class={styles.tableCard}>
      <Show
        when={props.answer.rows.length > 0}
        fallback={<p class={styles.empty}>{t(I18N_KEYS.STATS_EMPTY)}</p>}
      >
        <RankingTable
          ranking={props.ranking}
          rows={props.answer.rows}
          selected={selectedRow()}
          keep={{ project: props.project, editable: props.editable }}
          onSelect={(row) => selectRow(row.id)}
          onOpen={(row) => openRow(props.project, row)}
        />
        <p class={styles.shown}>
          {t(I18N_KEYS.STATS_ROWS_SHOWN, {
            shown: props.answer.rows.length,
            total: props.answer.total,
          })}
        </p>
      </Show>
    </div>
  );
}

/** The languages of a project and whether it has a project file to write (a folder). */
function useProjectInfo(project: () => number) {
  const row = () => projects().find((entry) => entry.id === project());
  return {
    languages: () => (row()?.languages ?? []).map((info) => info.language),
    editable: () => row()?.kind === SOURCE_KIND.folder,
  };
}

function RankingView(props: { ranking: RankingName; answer: RankingAnswer; project: number }) {
  const { languages, editable } = useProjectInfo(() => props.project);
  return (
    <>
      <Show when={props.ranking === TAB.unused && props.answer.unused}>
        {(counts) => <UnusedIntro counts={counts()} />}
      </Show>
      <Show when={props.ranking === TAB.complexity}>
        <ComplexityCharts rows={props.answer.rows} />
      </Show>
      <Filters ranking={props.ranking} languages={languages()} />
      <Show when={props.ranking === TAB.unused}>
        <KeepFailure />
      </Show>
      <div class={styles.split}>
        <TableCard
          ranking={props.ranking}
          answer={props.answer}
          project={props.project}
          editable={editable()}
        />
        <SidePanel
          row={currentRow()}
          onOpen={(row) => openRow(props.project, row)}
          onUsages={openRowUsages}
        />
      </div>
      <Show when={props.ranking === TAB.unused && props.answer.left_out.length > 0}>
        <LeftOutList
          items={props.answer.left_out}
          total={props.answer.unused?.left_out ?? props.answer.left_out.length}
          project={props.project}
          editable={editable()}
        />
      </Show>
    </>
  );
}

/** The keys of a ranking: arrows move the selection, Enter opens the explorer, U the usages. */
function useRankingKeys(project: () => number | null) {
  useKeydown((event) => {
    const [row, id] = [currentRow(), project()];
    const modified = event.ctrlKey || event.altKey || event.metaKey;
    if (event.defaultPrevented || modified || isTyping(event.target)) {
      return;
    }
    if (statsTab() === TAB.overview) {
      return;
    }
    if (event.key === KEY.up || event.key === KEY.down) {
      moveSelection(event.key === KEY.up ? STEP.up : STEP.down);
      event.preventDefault();
    } else if (
      row !== null &&
      id !== null &&
      event.key === KEY.open &&
      !(event.target instanceof HTMLButtonElement)
    ) {
      openRow(id, row);
    } else if (row !== null && event.key.toLowerCase() === KEY.usages) {
      openRowUsages(row);
    }
  });
}

/** Loads the view again whenever the project, its newest scan, the tab or a filter changes. */
function useLoading(project: () => number | null) {
  const wanted = () => [
    project(),
    statsTab(),
    statsTests(),
    statsKinds(),
    statsLanguage(),
    statsOrder(),
    statsAssessment(),
  ];
  createEffect(
    on(wanted, () => {
      const id = project();
      if (id !== null) {
        enterProject(id);
        void loadStats(id);
      }
    }),
  );
  reloadAfterScan(project);
}

/** Loads again when a scan ends: the newest scan of the open project is another one than before. */
function reloadAfterScan(project: () => number | null) {
  const newest = () => {
    const row = projects().find((entry) => entry.id === project());
    return row === undefined ? undefined : (row.scan?.snapshot ?? null);
  };
  createEffect(
    on(
      newest,
      (now, before) => {
        const id = project();
        if (id !== null && before !== undefined && now !== undefined && now !== before) {
          void loadStats(id);
        }
      },
      { defer: true },
    ),
  );
}

function Head(props: { name: string | undefined }) {
  return (
    <div class={styles.head}>
      <h1>{t(I18N_KEYS.STATS_TITLE)}</h1>
      <span class={styles.meta}>{props.name}</span>
      <button
        type="button"
        class={cx(styles.toggle, statsTests() && styles.toggleOn)}
        aria-pressed={statsTests()}
        onClick={() => setStatsTests(!statsTests())}
      >
        {t(I18N_KEYS.STATS_TESTS_TOGGLE)}
      </button>
    </div>
  );
}

function Body(props: { project: number; tab: Tab }) {
  return (
    <Switch>
      <Match when={props.tab === TAB.overview && statsOverview()}>
        {(data) => (
          <Show
            when={data().snapshot !== null}
            fallback={<p class={styles.empty}>{t(I18N_KEYS.STATS_NO_SCAN)}</p>}
          >
            <OverviewView
              data={data()}
              onShowAll={showTab}
              onOpen={(row) => openRow(props.project, row)}
            />
          </Show>
        )}
      </Match>
      <Match when={props.tab !== TAB.overview && statsRanking()}>
        {(answer) => (
          <Show
            when={answer().snapshot !== null}
            fallback={<p class={styles.empty}>{t(I18N_KEYS.STATS_NO_SCAN)}</p>}
          >
            <RankingView
              ranking={props.tab as RankingName}
              answer={answer()}
              project={props.project}
            />
          </Show>
        )}
      </Match>
    </Switch>
  );
}

/** The statistics of the open project (ADR 0081): an overview of cards, and one tab per ranking.
 * Every row leads into the explorer. */
export function StatsScreen() {
  onMount(() => void refreshProjects());
  const project = createMemo(() => activeProject());
  const name = () => projects().find((row) => row.id === project())?.name;
  useLoading(project);
  useRankingKeys(project);
  return (
    <section class={styles.screen}>
      <HatchPattern />
      <Head name={name()} />
      <Tabs />
      <Show when={statsFailed()}>
        <NoticeBox class={styles.failed} text={t(I18N_KEYS.STATS_FAILED)} alert={true} />
      </Show>
      <SlowLoad
        waiting={() => project() !== null && statsOverview() === null && !statsFailed()}
        class={styles.loading}
      />
      <div
        id={PANEL_ID}
        class={styles.panel}
        role={ROLE.tabpanel}
        aria-labelledby={`${TAB_ID_PREFIX}${statsTab()}`}
      >
        <Show when={project()}>{(id) => <Body project={id()} tab={statsTab()} />}</Show>
      </div>
    </section>
  );
}
