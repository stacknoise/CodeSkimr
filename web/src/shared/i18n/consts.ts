/** All Fluent files of all languages, as text (blueprint 13). The path names the language. */
export const FTL_FILES = import.meta.glob("../../../../locales/*/*.ftl", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

/** Separates the folders of a path in `FTL_FILES`. */
export const PATH_SEPARATOR = "/";
/** Languages of the UI, the first is the fallback. */
export const LANGUAGES = ["en", "de"] as const;
/** Language used when a text is missing in another one. */
export const FALLBACK_LANGUAGE = "en";
/** Beginning of a language tag of German. */
export const GERMAN_TAG = "de";
/** Attribute of `<html>` that names the language. */
export const LANGUAGE_ATTRIBUTE = "lang";
