import { textOf } from "../../shared/i18n/i18n";
import { LANGUAGE_NAMES, TOOL_LANGUAGE_SEPARATOR, TOOL_STATE } from "./consts";
import type { ToolInfo, ToolsInfo } from "./types";

/** The tools that are not ready: what the user has to install or update. */
export function problemsOf(info: ToolsInfo | null): ToolInfo[] {
  return info?.tools.filter((tool) => tool.state !== TOOL_STATE.ready) ?? [];
}

/** The display name of a language, the id of one without a name. */
export function languageName(language: string): string {
  return LANGUAGE_NAMES[language] ?? language;
}

/** The names of languages in one line. */
export function languageList(languages: readonly string[]): string {
  return languages.map(languageName).join(TOOL_LANGUAGE_SEPARATOR);
}

/** The languages for which level 2 can run. */
export function readyLanguages(info: ToolsInfo | null): string[] {
  return info?.languages.filter((entry) => entry.level2_ready).map((entry) => entry.language) ?? [];
}

/** What a tool is for and how to install it, in the current language (the pack names the keys). */
export function toolTexts(tool: ToolInfo): { purpose: string; install: string } {
  return { purpose: textOf(tool.purpose_key), install: textOf(tool.install_key) };
}
