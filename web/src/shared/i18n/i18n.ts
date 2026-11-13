import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle";
import { createSignal } from "solid-js";
import { LANGUAGE_STORAGE_KEY } from "../settings/consts";
import { readSetting, writeSetting } from "../settings/storage";
import { ERROR_CODES, type ErrorCode } from "../api/error-codes.generated";
import {
  FALLBACK_LANGUAGE,
  FTL_FILES,
  GERMAN_TAG,
  LANGUAGE_ATTRIBUTE,
  LANGUAGES,
  PATH_SEPARATOR,
} from "./consts";
import { I18N_KEYS } from "./keys";

/** Id of a message in the Fluent files, such as `app-name`. */
export type MessageKey = (typeof I18N_KEYS)[keyof typeof I18N_KEYS];

/** A language the UI offers. */
export type Language = (typeof LANGUAGES)[number];

/** Whether `value` is a language the UI offers. */
export function isLanguage(value: string | null): value is Language {
  return LANGUAGES.some((language) => language === value);
}

/** The folder name in front of the file name: `locales/de/app.ftl` is `de`. */
function languageOf(path: string): string | undefined {
  const folders = path.split(PATH_SEPARATOR);
  folders.pop();
  return folders.pop();
}

function loadBundles(files: Record<string, string>): Map<string, FluentBundle> {
  const bundles = new Map<string, FluentBundle>();
  for (const [path, text] of Object.entries(files)) {
    const language = languageOf(path);
    if (language === undefined) {
      continue;
    }
    const bundle = bundles.get(language) ?? new FluentBundle(language, { useIsolating: false });
    bundle.addResource(new FluentResource(text));
    bundles.set(language, bundle);
  }
  return bundles;
}

const bundles = loadBundles(FTL_FILES);

/** The language of the browser, as far as the UI offers it. */
export function systemLanguage(tags: readonly string[] = navigator.languages): Language {
  return tags.some((tag) => tag.toLowerCase().startsWith(GERMAN_TAG))
    ? GERMAN_TAG
    : FALLBACK_LANGUAGE;
}

function initialLanguage(): Language {
  const stored = readSetting(LANGUAGE_STORAGE_KEY);
  return isLanguage(stored) ? stored : systemLanguage();
}

const [language, setLanguageSignal] = createSignal<Language>(initialLanguage());

/** The language of the texts. */
export const currentLanguage = language;

/** Stores the language and switches all texts at once. */
export function setLanguage(next: Language): void {
  writeSetting(LANGUAGE_STORAGE_KEY, next);
  setLanguageSignal(next);
  document.documentElement.setAttribute(LANGUAGE_ATTRIBUTE, next);
}

/** The text of message `key` in the current language, else in English, else `undefined`. */
function lookup(key: string, args?: Record<string, FluentVariable>): string | undefined {
  for (const code of [language(), FALLBACK_LANGUAGE]) {
    const bundle = bundles.get(code);
    const pattern = bundle?.getMessage(key)?.value;
    if (bundle && pattern) {
      return bundle.formatPattern(pattern, args);
    }
  }
  return undefined;
}

/** The text of `key` in the current language; the English text if a translation is missing,
 * and the key itself if there is none at all. */
export function t(key: MessageKey, args?: Record<string, FluentVariable>): string {
  return lookup(key, args) ?? key;
}

/** The user-facing text of an error code, as in the Fluent file `errors.ftl`. */
export function errorText(code: ErrorCode): string {
  return lookup(ERROR_CODES[code].messageKey) ?? code;
}
