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

/** Opens the declaration of a symbol in the editor of the settings (`E`, `Alt+Enter`). Nothing
 * happens for a project that is not a folder or while the settings are not loaded. */
export async function openInEditor(project: number, symbol: number): Promise<void> {
  const config = uiConfig();
  if (config === null) {
    return;
  }
  const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol });
  const path = absolutePath(project, info.path);
  const url = path === null ? null : buildEditorUrl(config.editor, path, info.first_line);
  if (url !== null) {
    openLink(url);
  }
}
