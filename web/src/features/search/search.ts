import { createSignal } from "solid-js";
import { navigate } from "../../app/router";
import { EXPLORER_ROUTE } from "../../app/consts";
import { runCommand } from "../../shared/api/commands";
import { activeProject } from "../../shared/project/active";
import { TYPE_KINDS } from "../explorer/consts";
import { requestFocus } from "../../shared/navigation/navigation";
import { FILTER, MS_DIGITS, SEARCH_COMMAND, type Filter } from "./consts";
import { parseQuery } from "./parse";
import { resetPreview } from "./preview";
import type { Hit, SearchResult } from "./types";

const [open, setOpen] = createSignal(false);
const [raw, setRaw] = createSignal("");
const [chip, setChip] = createSignal<Filter>(FILTER.all);
const [hits, setHits] = createSignal<Hit[]>([]);
const [scanned, setScanned] = createSignal(true);
const [failed, setFailed] = createSignal(false);
const [took, setTook] = createSignal<number | null>(null);
const [entries, setEntries] = createSignal<number | null>(null);
const [selected, setSelected] = createSignal(0);

/** Counts the searches started; the answer of an earlier one is dropped. */
let sequence = 0;

/** Whether the overlay is shown. */
export const searchOpen = open;
/** What the input holds, with its prefix. */
export const searchText = raw;
/** The chip that is on. A prefix in the input wins over it. */
export const searchChip = chip;
/** How many symbols the index holds, known after the first search of the project. */
export const searchEntries = entries;
/** The symbols found. */
export const searchHits = hits;
/** Whether the project has a scan to search. */
export const searchScanned = scanned;
/** Whether the last search failed. */
export const searchFailed = failed;
/** How long the last search took, in milliseconds. */
export const searchMs = took;
/** The index of the chosen entry. */
export const chosen = selected;
export const choose = setSelected;

/** Shows the overlay with an empty input. */
export function openSearch(): void {
  if (open()) {
    return;
  }
  sequence += 1;
  setRaw("");
  setChip(FILTER.all);
  setHits([]);
  setTook(null);
  setEntries(null);
  setFailed(false);
  setSelected(0);
  setOpen(true);
}

/** Hides the overlay. */
export function closeSearch(): void {
  sequence += 1;
  resetPreview();
  setOpen(false);
}

/** Searches for what the input holds. Only a text without the `>` prefix goes to the server. */
export async function runSearch(): Promise<void> {
  const { filter, text } = parseQuery(raw(), chip());
  const project = activeProject();
  sequence += 1;
  const mine = sequence;
  setSelected(0);
  if (project === null || text === "" || filter === FILTER.commands) {
    setHits([]);
    setTook(null);
    return;
  }
  const started = performance.now();
  try {
    const found = await runCommand<SearchResult>(SEARCH_COMMAND, { project, query: text });
    if (mine === sequence) {
      setHits(found.hits);
      setScanned(found.snapshot !== null);
      setEntries(found.entries ?? null);
      setFailed(false);
      setTook(Number((performance.now() - started).toFixed(MS_DIGITS)));
    }
  } catch {
    if (mine === sequence) {
      setHits([]);
      setTook(null);
      setFailed(true);
    }
  }
}

/** Sets the text of the input and searches. */
export function typeSearch(value: string): void {
  setRaw(value);
  void runSearch();
}

/** Turns a chip on and searches again. */
export function pickChip(filter: Filter): void {
  setChip(filter);
  void runSearch();
}

/** Shows a hit in the explorer: the type, or the type that holds the member. External navigation
 * opens the rows down to it (blueprint 15.4). */
export function openHit(hit: Hit): void {
  const isType = TYPE_KINDS.includes(hit.kind);
  const owner = isType ? hit.id : (hit.parent ?? hit.id);
  const member = owner === hit.id ? null : hit.id;
  requestFocus({ type: owner, member, kind: member === null ? null : hit.kind });
  closeSearch();
  navigate(EXPLORER_ROUTE.path);
}
