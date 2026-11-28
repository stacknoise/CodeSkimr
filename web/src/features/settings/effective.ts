import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  EDIT,
  EFFECTIVE_COMMAND,
  GROUP_TEXTS,
  KEY_SEPARATOR,
  LIST_SEPARATOR,
  NO_VALUE,
  ORIGIN,
  SET_VALUE_COMMAND,
  UNIT_SUFFIXES,
} from "./consts";

/** One setting as `settings.effective` answers it. */
export interface EffectiveSetting {
  /** The dotted key path. */
  key: string;
  /** The value as TOML text; empty if it is secret. */
  value: string;
  origin: (typeof ORIGIN)[keyof typeof ORIGIN];
  hidden: boolean;
  /** The value of the shipped default, as TOML text. */
  default: string;
  /** Whether the value can be changed here and from when it would count. */
  edit: (typeof EDIT)[keyof typeof EDIT];
  /** A saved value that counts from the next start; `null` if there is none. */
  pending: string | null;
}

/** The answer of `settings.effective`. */
export interface EffectiveSettings {
  user_file: { path: string; found: boolean } | null;
  overrides: string[];
  settings: EffectiveSetting[];
}

/** A table of the configuration with its settings. */
export interface SettingGroup {
  name: string;
  settings: EffectiveSetting[];
}

const [loaded, setLoaded] = createSignal<EffectiveSettings | null>(null);
const [failed, setFailed] = createSignal(false);
const [search, setSearch] = createSignal("");

/** The text the list of effective settings is narrowed to; it comes from the tab bar. */
export const effectiveSearch = search;
/** Narrows the list of effective settings. */
export const setEffectiveSearch = setSearch;

/** The settings as the last call got them; `null` before the first answer. */
export const effectiveSettings = loaded;
/** Whether the last call got no answer. */
export const effectiveFailed = failed;

/** Asks the server for the settings in effect. A failure keeps what the screen shows. */
export async function loadEffectiveSettings(): Promise<void> {
  try {
    setLoaded(await runCommand<EffectiveSettings>(EFFECTIVE_COMMAND));
    setFailed(false);
  } catch {
    setFailed(true);
  }
}

/** Saves a number or a switch of one setting, or forgets the saved value with `null`, and reads
 * the settings again.
 *
 * @throws ApiError if the server refuses the value or cannot write it.
 */
export async function saveSettingValue(key: string, value: number | boolean | null): Promise<void> {
  await runCommand(SET_VALUE_COMMAND, { key, value });
  await loadEffectiveSettings();
}

/** The table a key belongs to: its first part. */
export function groupOf(key: string): string {
  return key.split(KEY_SEPARATOR)[0] ?? key;
}

/** The settings that match the search text (in the key or the value) and, if `onlyChanged`, were
 * not left at the default. */
export function filterSettings(
  all: EffectiveSetting[],
  search: string,
  onlyChanged: boolean,
): EffectiveSetting[] {
  const needle = search.trim().toLowerCase();
  return all.filter(
    (setting) =>
      (!onlyChanged || setting.origin !== ORIGIN.default) &&
      (needle === "" ||
        setting.key.toLowerCase().includes(needle) ||
        setting.value.toLowerCase().includes(needle)),
  );
}

/** The settings by table, in the order of `GROUP_TEXTS`; a table that is not listed comes last. */
export function groupSettings(settings: EffectiveSetting[]): SettingGroup[] {
  const names = Object.keys(GROUP_TEXTS);
  const rank = (name: string) => {
    const index = names.indexOf(name);
    return index < 0 ? names.length : index;
  };
  const groups = new Map<string, EffectiveSetting[]>();
  for (const setting of settings) {
    const name = groupOf(setting.key);
    groups.set(name, [...(groups.get(name) ?? []), setting]);
  }
  return [...groups.entries()]
    .map(([name, entries]) => ({ name, settings: entries }))
    .sort((a, b) => rank(a.name) - rank(b.name));
}

/** The heading of a table of the configuration; its own name if no text is known. */
export function groupTitle(name: string): string {
  const key: MessageKey | undefined = GROUP_TEXTS[name];
  return key === undefined ? name : t(key);
}

/** TOML text of a value in the way a person reads it: strings without quotes, lists joined,
 * switches in words, and the unit that the end of the key name tells. */
export function displayValue(setting: EffectiveSetting): string {
  if (setting.hidden) {
    return t(I18N_KEYS.SETTINGS_EFFECTIVE_HIDDEN);
  }
  const text = setting.value;
  if (text === "true" || text === "false") {
    return t(text === "true" ? I18N_KEYS.SETTINGS_EFFECTIVE_ON : I18N_KEYS.SETTINGS_EFFECTIVE_OFF);
  }
  const plain = plainString(text);
  if (plain !== null) {
    return plain;
  }
  const list = plainList(text);
  if (list !== null) {
    return list;
  }
  const unit = UNIT_SUFFIXES.find(([suffix]) => setting.key.endsWith(suffix));
  return unit === undefined ? text : t(unit[1], { value: text });
}

/** The content of a TOML string in quotes; `null` if the text is not one. */
function plainString(text: string): string | null {
  if (text.startsWith("'") && text.endsWith("'") && text.length > 1) {
    return text.slice(1, text.length - 1);
  }
  try {
    const parsed: unknown = JSON.parse(text);
    return typeof parsed === "string" ? parsed : null;
  } catch {
    return null;
  }
}

/** The strings of a TOML list, joined; `null` if the text is no list of strings. */
function plainList(text: string): string | null {
  try {
    const parsed: unknown = JSON.parse(text);
    if (Array.isArray(parsed) && parsed.every((item) => typeof item === "string")) {
      return parsed.length === 0 ? NO_VALUE : parsed.join(LIST_SEPARATOR);
    }
  } catch {
    return null;
  }
  return null;
}
