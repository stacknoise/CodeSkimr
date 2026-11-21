import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { showUsages } from "../../shared/navigation/navigation";
import { openHit } from "../search/search";
import { COMMAND, KINDS_ALL, ORDER_CERTAIN, ROW_LIMIT, TAB } from "./consts";
import type {
  Assessment,
  Kinds,
  Order,
  Overview,
  RankingAnswer,
  RankingName,
  Row,
  Tab,
} from "./types";

const [tab, setTab] = createSignal<Tab>(TAB.overview);
const [tests, setTests] = createSignal(false);
const [kinds, setKinds] = createSignal<Kinds>(KINDS_ALL);
const [language, setLanguage] = createSignal<string | null>(null);
const [order, setOrder] = createSignal<Order>(ORDER_CERTAIN);
const [assessment, setAssessment] = createSignal<Assessment | null>(null);
const [overview, setOverview] = createSignal<Overview | null>(null);
const [ranking, setRanking] = createSignal<RankingAnswer | null>(null);
const [selected, setSelected] = createSignal<number | null>(null);
const [failed, setFailed] = createSignal(false);
/** Counts the loads, so that a slow answer for a view that is no longer shown is dropped. */
let latest = 0;

/** The view that is shown. */
export const statsTab = tab;
/** Whether test code counts. */
export const statsTests = tests;
/** Includes or leaves out test code; the next load asks for it. */
export const setStatsTests = setTests;
/** The kinds of symbols the ranking holds. */
export const statsKinds = kinds;
export const setStatsKinds = setKinds;
/** The language the ranking is cut to, `null` for all. */
export const statsLanguage = language;
export const setStatsLanguage = setLanguage;
/** Which count the most used symbols are sorted by. */
export const statsOrder = order;
export const setStatsOrder = setOrder;
/** The assessment the unused symbols are cut to, `null` for all. */
export const statsAssessment = assessment;
export const setStatsAssessment = setAssessment;
/** The overview that was loaded last, `null` until the first answer. */
export const statsOverview = overview;
/** The ranking that was loaded last, `null` until the first answer of a tab. */
export const statsRanking = ranking;
/** Id of the selected row, `null` if none is. */
export const selectedRow = selected;
/** Selects a row. */
export const selectRow = setSelected;
/** Whether the last load failed. */
export const statsFailed = failed;

/** Shows a view, with a choice of kinds if the caller wants one (a card of types leads to a
 * ranking of types); the rows of the view before are forgotten, so no ranking shows under the wrong
 * tab while the new one loads. */
export function showTab(next: Tab, wanted?: Kinds): void {
  if (wanted !== undefined) {
    setKinds(wanted);
  }
  if (next !== tab()) {
    setRanking(null);
    setSelected(null);
    setTab(next);
  }
}

/** The row that is selected, if the ranking still holds it. */
export function currentRow(): Row | null {
  return ranking()?.rows.find((row) => row.id === selected()) ?? null;
}

function rankingInput(project: number, name: RankingName) {
  return {
    project,
    ranking: name,
    // The complexity tab has no choice of kinds, so a choice made on another tab must not cut it.
    kinds: name === TAB.complexity ? KINDS_ALL : kinds(),
    language: language(),
    include_tests: tests(),
    order: order(),
    assessment: name === TAB.unused ? assessment() : null,
    limit: ROW_LIMIT,
  };
}

async function fetchOverview(project: number, mine: number): Promise<void> {
  const answer = await runCommand<Overview>(COMMAND.overview, {
    project,
    include_tests: tests(),
  });
  if (mine === latest) {
    setOverview(answer);
  }
}

async function fetchRanking(project: number, view: RankingName, mine: number): Promise<void> {
  const answer = await runCommand<RankingAnswer>(COMMAND.ranking, rankingInput(project, view));
  if (mine === latest) {
    const first = answer.rows[0]?.id ?? null;
    setRanking(answer);
    setSelected((id) => (answer.rows.some((row) => row.id === id) ? id : first));
  }
}

/** The overview feeds the cards, the counter of the tab and the histogram, so every view loads it
 * next to its own ranking. */
async function fetchView(project: number, view: Tab, mine: number): Promise<void> {
  const ranked = view === TAB.overview ? Promise.resolve() : fetchRanking(project, view, mine);
  await Promise.all([fetchOverview(project, mine), ranked]);
}

/** Loads the view that is shown for a project. A failure is kept for the screen. Only the answers
 * of the newest load are kept, so a slow answer for an older filter, tab or project is dropped. */
export async function loadStats(project: number): Promise<void> {
  const mine = ++latest;
  try {
    await fetchView(project, tab(), mine);
    if (mine === latest) {
      setFailed(false);
    }
  } catch (error) {
    if (mine === latest) {
      setFailed(error instanceof ApiError || error instanceof TypeError);
    }
  }
}

/** Shows a symbol of a ranking in the explorer: the type, or the type that holds the member. */
export function openRow(project: number, row: Row): void {
  openHit({
    id: row.id,
    name: row.name,
    qualified_name: row.qualified_name,
    kind: row.kind,
    path: row.path,
    parent: row.parent,
    project,
    project_name: "",
  });
}

/** Shows the usages of a symbol of a ranking. */
export function openRowUsages(row: Row): void {
  showUsages(row.id);
}

/** Moves the selection by `step` rows, staying inside the ranking. */
export function moveSelection(step: number): void {
  const rows = ranking()?.rows ?? [];
  const at = rows.findIndex((row) => row.id === selected());
  const next = rows[Math.min(Math.max(at + step, 0), rows.length - 1)];
  if (next !== undefined) {
    setSelected(next.id);
  }
}

/** The project whose statistics the signals hold. */
let shown: number | null = null;

/** Tells which project the screen shows. Tab, filters, rows and selection of another project are
 * forgotten: its language may not exist here and its rows must not show under this name. */
export function enterProject(project: number): void {
  if (shown !== null && shown !== project) {
    resetStats();
  }
  shown = project;
}

/** Forgets everything; for tests and when the project changes. */
export function resetStats(): void {
  latest += 1;
  shown = null;
  setTab(TAB.overview);
  setTests(false);
  setKinds(KINDS_ALL);
  setLanguage(null);
  setOrder(ORDER_CERTAIN);
  setAssessment(null);
  setOverview(null);
  setRanking(null);
  setSelected(null);
  setFailed(false);
}
