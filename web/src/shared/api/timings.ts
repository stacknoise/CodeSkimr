import { createSignal } from "solid-js";

/** What this session measured: the last round trip in milliseconds by budget key. */
const [timings, setTimings] = createSignal<Readonly<Record<string, number>>>({});
/** The round trip of the command that ran last, `null` before the first one. */
const [lastQuery, setLastQuery] = createSignal<number | null>(null);

/** The last measured round trip by budget key; a key without an entry was not measured. */
export const measuredTimes = timings;

/** The round trip of the last timed command, for the status bar. */
export const lastQueryMs = lastQuery;

/** Remembers the round trip of a command that measures a budget. */
export function recordTime(budget: string, ms: number): void {
  setTimings((before) => ({ ...before, [budget]: ms }));
  setLastQuery(ms);
}

/** Forgets all measurements (tests). */
export function resetTimings(): void {
  setTimings({});
  setLastQuery(null);
}
