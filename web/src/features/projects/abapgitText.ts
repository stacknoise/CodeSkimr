import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { AbapGitInfo } from "./scope";

/** The folder below which the objects lie, in words: "the project folder" for the root. */
export function folderText(info: AbapGitInfo): string {
  return info.starting_folder === ""
    ? t(I18N_KEYS.PROJECTS_ABAPGIT_FOLDER_ROOT)
    : info.starting_folder;
}

/** The folder logic in words: "prefix" or "full". */
export function logicText(info: AbapGitInfo): string {
  return info.folder_logic === "full"
    ? t(I18N_KEYS.PROJECTS_ABAPGIT_LOGIC_FULL)
    : t(I18N_KEYS.PROJECTS_ABAPGIT_LOGIC_PREFIX);
}
