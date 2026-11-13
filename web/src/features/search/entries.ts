import { t } from "../../shared/i18n/i18n";
import type { UiCommand } from "./commands";
import { FILTER, GROUP, NONE, TYPE_KINDS, type Filter, type GroupId } from "./consts";
import type { Hit } from "./types";

/** A line of the list that can be chosen: a symbol or a command. */
export interface Entry {
  key: string;
  group: GroupId;
  /** The text of the line. */
  label: string;
  hit: Hit | null;
  command: UiCommand | null;
}

/** A row of the list: the title of a group (no entry) or an entry. */
export interface ListRow {
  key: string;
  group: GroupId;
  entry: Entry | null;
  /** Index of the entry among the entries, `NONE` for a title. */
  index: number;
}

const shows = (filter: Filter, wanted: Filter) => filter === FILTER.all || filter === wanted;

/** The entries for a filter and a text, grouped: types, members, commands. Symbols come from the
 * server in the order of their rank; a command is kept if its text contains the search text, and
 * all of them show for `>` alone. */
export function buildEntries(
  hits: Hit[],
  commands: readonly UiCommand[],
  filter: Filter,
  text: string,
): Entry[] {
  const symbol = (group: GroupId, hit: Hit): Entry => ({
    key: `${group}-${hit.id}`,
    group,
    label: hit.name,
    hit,
    command: null,
  });
  const isType = (hit: Hit) => TYPE_KINDS.includes(hit.kind);
  const types = shows(filter, FILTER.types)
    ? hits.filter(isType).map((hit) => symbol(GROUP.types, hit))
    : [];
  const members = shows(filter, FILTER.members)
    ? hits.filter((hit) => !isType(hit)).map((hit) => symbol(GROUP.members, hit))
    : [];
  return [...types, ...members, ...commandEntries(commands, filter, text)];
}

function commandEntries(commands: readonly UiCommand[], filter: Filter, text: string): Entry[] {
  const needle = text.toLowerCase();
  const wanted = filter === FILTER.commands || (filter === FILTER.all && needle !== "");
  return commands
    .filter((command) => wanted && t(command.label).toLowerCase().includes(needle))
    .map((command) => ({
      key: `${GROUP.commands}-${command.label}`,
      group: GROUP.commands,
      label: t(command.label),
      hit: null,
      command,
    }));
}

/** The entries with the title of each group in front of its first entry. */
export function listRows(entries: Entry[]): ListRow[] {
  const rows: ListRow[] = [];
  entries.forEach((entry, index) => {
    if (index === 0 || entries[index - 1]?.group !== entry.group) {
      rows.push({ key: `header-${entry.group}`, group: entry.group, entry: null, index: NONE });
    }
    rows.push({ key: entry.key, group: entry.group, entry, index });
  });
  return rows;
}
