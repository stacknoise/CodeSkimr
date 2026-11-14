import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";

/** Id of the command of the server that searches the names. */
export const SEARCH_COMMAND = "search.query";

/** What a search can be limited to (blueprint B.4). */
export const FILTER = {
  all: "all",
  types: "types",
  members: "members",
  commands: "commands",
} as const;
export type Filter = (typeof FILTER)[keyof typeof FILTER];

/** The prefixes of the input that choose a filter. */
export const PREFIX_OF = { types: "t:", members: "m:", commands: ">" } as const;
export const PREFIXES: readonly { filter: Filter; prefix: string }[] = [
  { filter: FILTER.types, prefix: PREFIX_OF.types },
  { filter: FILTER.members, prefix: PREFIX_OF.members },
  { filter: FILTER.commands, prefix: PREFIX_OF.commands },
];

/** The chips under the input, in the order of the mock. */
export const CHIPS: readonly { filter: Filter; label: MessageKey; prefix?: string }[] = [
  { filter: FILTER.all, label: I18N_KEYS.PALETTE_FILTER_ALL },
  { filter: FILTER.types, label: I18N_KEYS.PALETTE_FILTER_TYPES, prefix: PREFIX_OF.types },
  { filter: FILTER.members, label: I18N_KEYS.PALETTE_FILTER_MEMBERS, prefix: PREFIX_OF.members },
  { filter: FILTER.commands, label: I18N_KEYS.PALETTE_FILTER_COMMANDS, prefix: PREFIX_OF.commands },
];

/** The groups of the result list. */
export const GROUP = { types: "types", members: "members", commands: "commands" } as const;
export type GroupId = (typeof GROUP)[keyof typeof GROUP];

/** Title of each group. */
export const GROUP_TITLE: Record<GroupId, MessageKey> = {
  types: I18N_KEYS.PALETTE_GROUP_TYPES,
  members: I18N_KEYS.PALETTE_GROUP_MEMBERS,
  commands: I18N_KEYS.PALETTE_GROUP_COMMANDS,
};

/** The letter of the badge of a command entry. */
export const COMMAND_LETTER = ">";

/** Keys of the overlay. */
export const KEY = {
  open: "k",
  close: "Escape",
  down: "ArrowDown",
  up: "ArrowUp",
  enter: "Enter",
  tab: "Tab",
  copy: "c",
} as const;
export const STEP_DOWN = 1;
export const STEP_UP = -1;
export const NONE = -1;

/** Id prefix of the options of the list, for `aria-activedescendant`. */
export const OPTION_ID_PREFIX = "search-option-";
/** Id of the input of the overlay. */
export const INPUT_ID = "search-input";
/** Id of the list of results. */
export const LIST_ID = "search-list";
/** Roles of the list. */
export const ROLE = { listbox: "listbox", option: "option", combobox: "combobox" } as const;
/** Value of `aria-modal` and `aria-selected`. */
export const TRUE = "true";
/** The role of the overlay. */
export const DIALOG = "dialog";
/** Digits after the point of the time of a search. */
export const MS_DIGITS = 0;

/** The tiles of the metric strip that the preview of a hit shows, in this order (mock 4). */
export const PREVIEW_TILES: readonly string[] = ["loc", "members", "mi"];
