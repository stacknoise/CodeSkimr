import { runCommand } from "../../shared/api/commands";
import { buildEditorUrl, openLink } from "../../shared/editor/editor";
import { uiConfig } from "../../shared/settings/ui-config";
import { projects } from "../projects/projects";
import { SOURCE_KIND } from "../projects/consts";
import { PATH_JOINER, COMMAND } from "./consts";
import type { SymbolInfo } from "./types";

/** The absolute path of a file of a project that is a folder on this machine, or `null`. */
function absolutePath(project: number, path: string): string | null {
  const row = projects().find((candidate) => candidate.id === project);
  if (!row || row.kind !== SOURCE_KIND.folder) {
    return null;
  }
  return `${row.location.replace(/[\\/]+$/, "")}${PATH_JOINER}${path}`;
}

/** Opens a file of a project at a line in the editor of the settings. Nothing happens for a
 * project that is not a folder or while the settings are not loaded. */
export function openPlaceInEditor(project: number, path: string, line: number): void {
  const config = uiConfig();
  const absolute = config === null ? null : absolutePath(project, path);
  const url =
    config === null || absolute === null ? null : buildEditorUrl(config.editor, absolute, line);
  if (url !== null) {
    openLink(url);
  }
}

/** Opens the declaration of a symbol in the editor of the settings (`E`, `Alt+Enter`). Nothing
 * happens for a project that is not a folder or while the settings are not loaded. */
export async function openInEditor(project: number, symbol: number): Promise<void> {
  if (uiConfig() === null) {
    return;
  }
  try {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol });
    openPlaceInEditor(project, info.path, info.first_line);
  } catch {
    // The callers fire and forget; a symbol that cannot be read just opens nothing.
  }
}
