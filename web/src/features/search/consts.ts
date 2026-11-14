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
  files: "files",
  projects: "projects",
} as const;
export type Filter = (typeof FILTER)[keyof typeof FILTER];

/** The prefixes of the input that choose a filter. */
export const PREFIX_OF = {
  types: "t:",
  members: "m:",
  commands: ">",
  files: "f:",
  projects: "@",
} as const;
export const PREFIXES: readonly { filter: Filter; prefix: string }[] = [
  { filter: FILTER.types, prefix: PREFIX_OF.types },
  { filter: FILTER.members, prefix: PREFIX_OF.members },
  { filter: FILTER.commands, prefix: PREFIX_OF.commands },
  { filter: FILTER.files, prefix: PREFIX_OF.files },
  { filter: FILTER.projects, prefix: PREFIX_OF.projects },
];

/** The chips under the input, in the order of the mock. */
export const CHIPS: readonly { filter: Filter; label: MessageKey; prefix?: string }[] = [
  { filter: FILTER.all, label: I18N_KEYS.PALETTE_FILTER_ALL },
  { filter: FILTER.types, label: I18N_KEYS.PALETTE_FILTER_TYPES, prefix: PREFIX_OF.types },
  { filter: FILTER.members, label: I18N_KEYS.PALETTE_FILTER_MEMBERS, prefix: PREFIX_OF.members },
  { filter: FILTER.commands, label: I18N_KEYS.PALETTE_FILTER_COMMANDS, prefix: PREFIX_OF.commands },
  { filter: FILTER.files, label: I18N_KEYS.PALETTE_FILTER_FILES, prefix: PREFIX_OF.files },
  {
    filter: FILTER.projects,
    label: I18N_KEYS.PALETTE_SCOPE_ALL_PROJECTS,
    prefix: PREFIX_OF.projects,
  },
];

/** The groups of the result list. */
export const GROUP = {
  types: "types",
  members: "members",
  files: "files",
  commands: "commands",
} as const;
export type GroupId = (typeof GROUP)[keyof typeof GROUP];

/** Title of each group. */
export const GROUP_TITLE: Record<GroupId, MessageKey> = {
  types: I18N_KEYS.PALETTE_GROUP_TYPES,
  members: I18N_KEYS.PALETTE_GROUP_MEMBERS,
  files: I18N_KEYS.PALETTE_GROUP_FILES,
  commands: I18N_KEYS.PALETTE_GROUP_COMMANDS,
};

/** The letter of the badge of a command entry. */
export const COMMAND_LETTER = ">";
/** The letter of the badge of a file entry (mock 4). */
export const FILE_LETTER = "D";

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

/** Separates the folders of a path; the name of a file is what follows the last one. */
export const PATH_SEPARATOR = "/";
