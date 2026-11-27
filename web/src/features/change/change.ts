import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { openProject } from "../../shared/project/active";
import { navigate } from "../../app/router";
import { CHANGE_ROUTE } from "../../app/consts";
import { failed, type Failure } from "../explorer/failure";
import type { ScanHistoryAnswer } from "../scan/types";
import { COMMAND, MIN_SCANS, TAB } from "./consts";
import { impactRows, openCount } from "./rows";
import type { ChangeAnswer, Selection } from "./types";

type Tab = (typeof TAB)[keyof typeof TAB];

const NO_SELECTION: Selection = { base: null, target: null };

const [tab, setTab] = createSignal<Tab>(TAB.impact);
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
/** Shows a view. */
export const showChangeTab = setTab;
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
  setRowKey(
    rows.some((row) => row.key === current)
      ? current
      : ((rows.find((row) => openCount(row) > 0) ?? rows[0])?.key ?? null),
  );
}

/** The input of the comparison: only the scans that were picked, so the registry decides the rest. */
function inputOf(project: number, wanted: Selection) {
  return {
    project,
    ...(wanted.base === null ? {} : { base: wanted.base }),
    ...(wanted.target === null ? {} : { target: wanted.target }),
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
        : await runCommand<ChangeAnswer>(COMMAND.impact, inputOf(project, wanted));
    if (mine === latest) {
      setAnswer(read);
      setPicked(
        read === null ? NO_SELECTION : { base: read.base.snapshot, target: read.target.snapshot },
      );
      keepRow(read);
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

/** Asks the screen "Change" to compare a scan of the list with the current one: the view
 * "Comparison" opens with that scan as the base. */
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
  const wanted: Selection = { base: requestedBase, target: null };
  requestedBase = null;
  setPicked(wanted);
  return wanted;
}

/** Forgets what was read, when the screen is left. */
export function resetChange(): void {
  latest += 1;
  setScans(null);
  setAnswer(null);
  setFailure(null);
  setReading(false);
  setPicked(NO_SELECTION);
}
