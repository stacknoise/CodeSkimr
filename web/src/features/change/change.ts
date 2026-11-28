import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { activeProject, openProject } from "../../shared/project/active";
import { navigate } from "../../app/router";
import { CHANGE_ROUTE } from "../../app/consts";
import { failed, type Failure } from "../explorer/failure";
import type { ScanHistoryAnswer } from "../scan/types";
import { resetFilter } from "./filter";
import { COMMAND, METRIC_ALL, METRICS_LEVEL, MIN_SCANS, TAB } from "./consts";
import { firstToSelect, impactRows } from "./rows";
import { metricAvailable } from "./metrics";
import type { ChangeAnswer, MetricsAnswer, Selection } from "./types";

type Tab = (typeof TAB)[keyof typeof TAB];

const NO_SELECTION: Selection = { base: null, target: null };

const [tab, setTab] = createSignal<Tab>(TAB.impact);
const [metrics, setMetrics] = createSignal<MetricsAnswer | null>(null);
const [metricsFailure, setMetricsFailure] = createSignal<Failure | null>(null);
const [metricsReading, setMetricsReading] = createSignal(false);
const [metricsLevel, setMetricsLevel] = createSignal<string>(METRICS_LEVEL.member);
const [metricsMetric, setMetricsMetric] = createSignal<string>(METRIC_ALL);
/** Counts the reads of the metrics, so that an answer for a pair that is no longer shown is dropped. */
let latestMetrics = 0;
const [scans, setScans] = createSignal<ScanHistoryAnswer | null>(null);
const [answer, setAnswer] = createSignal<ChangeAnswer | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [reading, setReading] = createSignal(false);
const [picked, setPicked] = createSignal<Selection>(NO_SELECTION);
const [rowKey, setRowKey] = createSignal<string | null>(null);
const [onlyOpen, setOnlyOpen] = createSignal(true);
/** Counts the loads, so that an answer for a project or a pair that is no longer shown is dropped. */
let latest = 0;
/** The scan a button of the history asked to compare with the current one; the screen takes it
 * when it opens. */
let requestedBase: number | null = null;

/** The view that is shown. */
export const changeTab = tab;
/** The metric changes of the pair that is compared; `null` until they are read. */
export const changeMetrics = metrics;
/** The failure of the last read of the metrics; `null` if it worked. */
export const changeMetricsFailure = metricsFailure;
/** Whether the metrics are being read. */
export const changeMetricsReading = metricsReading;
/** The level the metrics are read for: `member`, `type` or `module`. */
export const changeMetricsLevel = metricsLevel;
/** The metric the lists are narrowed to, or the value that shows all. */
export const changeMetricsMetric = metricsMetric;
/** Narrows the lists of the metrics to one metric. */
export const showMetricsMetric = setMetricsMetric;

/** Reads the metric changes between the two scans of the answer that is shown (`scans.metrics`),
 * for the level that is chosen. The pair is the one the comparison resolved, so both views show the
 * same two scans. */
export async function loadMetrics(project: number): Promise<void> {
  const compared = answer();
  if (compared === null) {
    return;
  }
  const mine = ++latestMetrics;
  setMetricsReading(true);
  setMetricsFailure(null);
  try {
    const read = await runCommand<MetricsAnswer>(COMMAND.metrics, {
      project,
      base: compared.base.snapshot,
      target: compared.target.snapshot,
      level: metricsLevel(),
    });
    if (mine === latestMetrics) {
      setMetrics(read);
    }
  } catch (error) {
    if (mine === latestMetrics) {
      setMetrics(null);
      setMetricsFailure(failed(error));
    }
  } finally {
    if (mine === latestMetrics) {
      setMetricsReading(false);
    }
  }
}

/** Chooses the level of the metrics and reads them again. */
export function chooseMetricsLevel(project: number, level: string): void {
  setMetricsLevel(level);
  if (!metricAvailable(metricsMetric(), level)) {
    setMetricsMetric(METRIC_ALL);
  }
  void loadMetrics(project);
}

/** Forgets the metrics that were read, because the pair of scans changed or the screen was left. */
function forgetMetrics(): void {
  latestMetrics += 1;
  setMetrics(null);
  setMetricsFailure(null);
  setMetricsReading(false);
}

/** Shows a view; the metrics are read when their view opens and they are not there yet. */
export function showChangeTab(next: Tab): void {
  setTab(next);
  const project = activeProject();
  if (next === TAB.metrics && project !== null && metrics() === null && !metricsReading()) {
    void loadMetrics(project);
  }
}
/** The retained scans of the project, newest first; `null` until they are read. */
export const changeScans = scans;
/** The comparison that was read; `null` while there is none. */
export const changeAnswer = answer;
/** The failure of the last read; `null` if it worked. */
export const changeFailure = failure;
/** Whether a comparison is being read. */
export const changeReading = reading;
/** The two scans the selectors show: those of the answer, or what the user picked. */
export const changePicked = picked;
/** Picks the two scans of the next comparison. */
export const pickScans = setPicked;
/** The key of the row of the table that is selected. */
export const changeRowKey = rowKey;
/** Selects a row of the table. */
export const selectChangeRow = setRowKey;
/** Whether the list of usages shows only the places that were not touched. */
export const changeOnlyOpen = onlyOpen;
/** Switches the list of usages between the open places and all. */
export const showOnlyOpen = setOnlyOpen;

/** Keeps the row that is selected if the new answer has it, and takes the first one otherwise. */
function keepRow(read: ChangeAnswer | null): void {
  const rows = read === null ? [] : impactRows(read);
  const current = rowKey();
  setRowKey(rows.some((row) => row.key === current) ? current : (firstToSelect(rows)?.key ?? null));
}

/** The snapshot of the scan with the id `scan` in `list`, `null` if the list does not have it. */
function snapshotOf(list: ScanHistoryAnswer, scan: number | null): number | null {
  return list.scans.find((entry) => entry.scan === scan)?.snapshot ?? null;
}

/** The input of the comparison: only the scans that were picked, so the registry decides the rest.
 * The scans are named by their snapshot, which scans without a change share. */
function inputOf(project: number, list: ScanHistoryAnswer, wanted: Selection) {
  const base = snapshotOf(list, wanted.base);
  const target = snapshotOf(list, wanted.target);
  return {
    project,
    ...(base === null ? {} : { base }),
    ...(target === null ? {} : { target }),
  };
}

/** The scans the selectors show after a read: those the user picked, if the list still has them,
 * and the ones the registry chose for the rest. Scans without a change share a snapshot, so the
 * answer cannot tell which of them was meant. */
function shown(list: ScanHistoryAnswer, wanted: Selection, read: ChangeAnswer): Selection {
  const kept = (scan: number | null) => (snapshotOf(list, scan) === null ? null : scan);
  return {
    base: kept(wanted.base) ?? read.base.scan,
    target: kept(wanted.target) ?? read.target.scan,
  };
}

/** Reads the scans of the project and, if there are two, their comparison with the usages the
 * difference hits (`scans.impact` also holds everything `scans.compare` has, so one call serves
 * both views). Without `wanted` the registry compares the newest scan with the one before it. */
export async function loadChange(project: number, wanted: Selection = picked()): Promise<void> {
  const mine = ++latest;
  setReading(true);
  setFailure(null);
  try {
    const list = await runCommand<ScanHistoryAnswer>(COMMAND.list, { project });
    if (mine !== latest) {
      return;
    }
    setScans(list);
    const read =
      list.scans.length < MIN_SCANS
        ? null
        : await runCommand<ChangeAnswer>(COMMAND.impact, inputOf(project, list, wanted));
    if (mine === latest) {
      forgetMetrics();
      setAnswer(read);
      setPicked(read === null ? NO_SELECTION : shown(list, wanted, read));
      keepRow(read);
      if (tab() === TAB.metrics) {
        void loadMetrics(project);
      }
    }
  } catch (error) {
    if (mine === latest) {
      setAnswer(null);
      setFailure(failed(error));
    }
  } finally {
    if (mine === latest) {
      setReading(false);
    }
  }
}

/** Asks the screen "Change" to compare a scan of the list (`base` is its `scan` id) with the
 * current one: the view "Comparison" opens with that scan as the base. */
export function compareWithCurrent(project: number, base: number): void {
  requestedBase = base;
  setTab(TAB.compare);
  openProject(project);
  navigate(CHANGE_ROUTE.path);
}

/** Forgets what was read and takes the request of the history, if there is one; called when the
 * screen opens or the project changes. Returns the choice the first read starts with. */
export function enterChange(): Selection {
  latest += 1;
  setScans(null);
  setAnswer(null);
  setFailure(null);
  setReading(false);
  setRowKey(null);
  forgetMetrics();
  setMetricsMetric(METRIC_ALL);
  resetFilter();
  const wanted: Selection = { base: requestedBase, target: null };
  requestedBase = null;
  setPicked(wanted);
  return wanted;
}

/** Forgets what was read, when the screen is left. */
export function resetChange(): void {
  latest += 1;
  forgetMetrics();
  setScans(null);
  setAnswer(null);
  setFailure(null);
  setReading(false);
  setPicked(NO_SELECTION);
}
