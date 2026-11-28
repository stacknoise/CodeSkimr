import { SCOPE_PATH_SEPARATOR } from "./consts";

/** A language on the scope page. */
export interface ScopeLanguage {
  language: string;
  files: number | null;
  covered: boolean;
}

/** A folder on the scope page. */
export interface ScopeFolder {
  path: string;
  files: number | null;
  covered: boolean;
  left_out: boolean;
  languages: { language: string; files: number }[];
  project_files: string[];
}

/** What a project leaves out: the languages and the folders (with everything below them). */
export interface ScopeChoice {
  languages: string[];
  folders: string[];
}

/** The number of objects of one type in an abapGit repository (`CLAS`, `INTF` …). */
export interface AbapGitKind {
  kind: string;
  objects: number;
}

/** What an abapGit repository holds (ADR 0113). */
export interface AbapGitInfo {
  starting_folder: string;
  folder_logic: "prefix" | "full";
  configured: boolean;
  kinds: AbapGitKind[];
  packages: number;
  source_files: number;
}

/** The answer of `scope.get`. */
export interface ScopeAnswer {
  project: number;
  live: boolean;
  scope: ScopeChoice;
  languages: ScopeLanguage[];
  folders: ScopeFolder[];
  folders_more: number;
  root_project_files: string[];
  abapgit: AbapGitInfo | null;
  /** ABAP files without a recognised abapGit layout (issue #845). */
  abapgit_unrecognised: boolean;
}

/** The folder that is left out and contains `path` (or is it), if any. */
export function leftOutAbove(choice: ScopeChoice, path: string): string | null {
  return (
    choice.folders.find(
      (folder) => folder === path || path.startsWith(`${folder}${SCOPE_PATH_SEPARATOR}`),
    ) ?? null
  );
}

/** Whether the choice covers the folder. */
export function folderCovered(choice: ScopeChoice, path: string): boolean {
  return leftOutAbove(choice, path) === null;
}

/** Whether the choice leaves the folder out only because a folder above it is left out. */
export function leftOutByParent(choice: ScopeChoice, path: string): boolean {
  const above = leftOutAbove(choice, path);
  return above !== null && above !== path;
}

/** The depth of a folder: 0 for a folder directly in the project folder. */
export function depthOf(path: string): number {
  return path.split(SCOPE_PATH_SEPARATOR).length - 1;
}

/** The name of a folder without its parents. */
export function baseName(path: string): string {
  return path.split(SCOPE_PATH_SEPARATOR).pop() ?? path;
}

/** The choice with a language left out, or taken back in. */
export function toggleLanguage(choice: ScopeChoice, language: string): ScopeChoice {
  const left = choice.languages.includes(language);
  return {
    ...choice,
    languages: left
      ? choice.languages.filter((entry) => entry !== language)
      : [...choice.languages, language].sort(),
  };
}

/** The choice with a folder left out (its folders below are covered by it), or taken back in. */
export function toggleFolder(choice: ScopeChoice, path: string): ScopeChoice {
  if (choice.folders.includes(path)) {
    return { ...choice, folders: choice.folders.filter((entry) => entry !== path) };
  }
  const below = `${path}${SCOPE_PATH_SEPARATOR}`;
  const kept = choice.folders.filter((entry) => !entry.startsWith(below));
  return { ...choice, folders: [...kept, path].sort() };
}

/** The choice that takes every folder back in. */
export function takeFoldersBack(choice: ScopeChoice): ScopeChoice {
  return { ...choice, folders: [] };
}

/** Whether two choices differ. */
export function differs(left: ScopeChoice, right: ScopeChoice): boolean {
  return (
    left.languages.join("\n") !== right.languages.join("\n") ||
    left.folders.join("\n") !== right.folders.join("\n")
  );
}

/** How many of the listed folders the choice covers. */
export function coveredFolders(answer: ScopeAnswer, choice: ScopeChoice): number {
  return answer.folders.filter((folder) => folderCovered(choice, folder.path)).length;
}

/** What a change of the choice takes out: languages and folders that are newly left out, with
 * their files (a folder below one that is newly left out is counted once, in the folder above). */
export interface Consequences {
  languages: { language: string; files: number }[];
  folders: { path: string; files: number }[];
  back: string[];
}

/** The consequences of going from the saved choice to `choice`. */
export function consequences(answer: ScopeAnswer, choice: ScopeChoice): Consequences {
  const saved = answer.scope;
  const languages = choice.languages
    .filter((language) => !saved.languages.includes(language))
    .map((language) => ({
      language,
      files: answer.languages.find((entry) => entry.language === language)?.files ?? 0,
    }));
  const newFolders = choice.folders.filter((path) => folderCovered(saved, path));
  const folders = newFolders.map((path) => ({
    path,
    files: answer.folders.find((entry) => entry.path === path)?.files ?? 0,
  }));
  const back = [
    ...saved.languages.filter((language) => !choice.languages.includes(language)),
    ...saved.folders.filter((path) => folderCovered(choice, path)),
  ];
  return { languages, folders, back };
}

/** Whether the change takes something out of the project. */
export function takesOut(result: Consequences): boolean {
  return result.languages.length > 0 || result.folders.length > 0;
}
