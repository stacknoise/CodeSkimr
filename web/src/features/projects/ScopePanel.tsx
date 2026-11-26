import { createEffect, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { PART_SEPARATOR } from "./consts";
import { failureText } from "./failure";
import {
  baseName,
  consequences,
  coveredFolders,
  depthOf,
  differs,
  folderCovered,
  leftOutByParent,
  takeFoldersBack,
  takesOut,
  toggleFolder,
  toggleLanguage,
  type Consequences,
  type ScopeAnswer,
  type ScopeChoice,
  type ScopeFolder,
} from "./scope";
import { useScopeState, type ScopeState } from "./scopeState";
import { languageList, languageName } from "./tools";
import type { ProjectRow } from "./types";
import styles from "./ScopePanel.module.css";

/** The chips of the languages: each with a checkbox, the name and the number of files. */
function Languages(props: {
  answer: ScopeAnswer;
  choice: ScopeChoice;
  onChoice: (choice: ScopeChoice) => void;
}) {
  return (
    <section class={styles.section} aria-labelledby="scope-languages">
      <h3 id="scope-languages" class={styles.heading}>
        {t(I18N_KEYS.PROJECTS_SCOPE_LANGUAGES)}
      </h3>
      <ul class={styles.chips} aria-label={t(I18N_KEYS.PROJECTS_SCOPE_LANGUAGES_LABEL)}>
        <For each={props.answer.languages}>
          {(entry) => (
            <li>
              <label class={styles.chip}>
                <input
                  type="checkbox"
                  checked={!props.choice.languages.includes(entry.language)}
                  onChange={() => props.onChoice(toggleLanguage(props.choice, entry.language))}
                />
                <span>{languageName(entry.language)}</span>
                <Show when={entry.files !== null}>
                  <span class={styles.count}>{entry.files}</span>
                </Show>
              </label>
            </li>
          )}
        </For>
      </ul>
    </section>
  );
}

/** The note after the name of a folder: left out here or by a folder above it. */
function folderNote(choice: ScopeChoice, path: string): string | null {
  if (leftOutByParent(choice, path)) {
    return t(I18N_KEYS.PROJECTS_SCOPE_FOLDER_INHERITED);
  }
  return folderCovered(choice, path) ? null : t(I18N_KEYS.PROJECTS_SCOPE_FOLDER_LEFT_OUT);
}

/** One folder of the tree: checkbox, name, the files of the exact check, files and languages. */
function FolderRow(props: {
  folder: ScopeFolder;
  choice: ScopeChoice;
  onChoice: (choice: ScopeChoice) => void;
}) {
  const covered = () => folderCovered(props.choice, props.folder.path);
  const note = () => folderNote(props.choice, props.folder.path);
  return (
    <li class={cx(styles.folder, !covered() && styles.off)}>
      <label
        class={styles.folderLabel}
        style={{ "padding-inline-start": `${depthOf(props.folder.path)}rem` }}
      >
        <input
          type="checkbox"
          checked={covered()}
          disabled={leftOutByParent(props.choice, props.folder.path)}
          onChange={() => props.onChoice(toggleFolder(props.choice, props.folder.path))}
        />
        <span class={styles.name}>{baseName(props.folder.path)}</span>
      </label>
      <Show when={props.folder.project_files.length > 0}>
        <span class={styles.badge}>
          {t(I18N_KEYS.PROJECTS_SCOPE_STAGE2, {
            files: props.folder.project_files.join(PART_SEPARATOR),
          })}
        </span>
      </Show>
      <Show when={note()}>{(text) => <span class={styles.note}>{text()}</span>}</Show>
      <Show when={props.folder.files !== null}>
        <span class={styles.count}>
          {t(I18N_KEYS.PROJECTS_SCOPE_FILES, { count: props.folder.files ?? 0 })}
        </span>
      </Show>
      <span class={styles.languagesOf}>
        {languageList(props.folder.languages.map((entry) => entry.language))}
      </span>
    </li>
  );
}

/** The root row: how many folders are covered; a click takes all folders back in. */
function RootRow(props: {
  answer: ScopeAnswer;
  choice: ScopeChoice;
  row: ProjectRow;
  onChoice: (choice: ScopeChoice) => void;
}) {
  const covered = () => coveredFolders(props.answer, props.choice);
  const total = () => props.answer.folders.length;
  let box: HTMLInputElement | undefined;
  createEffect(() => {
    if (box !== undefined) {
      box.indeterminate = covered() < total() && covered() > 0;
      box.checked = covered() === total();
    }
  });
  return (
    <div class={styles.root}>
      <label class={styles.folderLabel} title={t(I18N_KEYS.PROJECTS_SCOPE_ROOT_ALL)}>
        <input
          ref={box}
          type="checkbox"
          aria-label={t(I18N_KEYS.PROJECTS_SCOPE_ROOT_ALL)}
          disabled={covered() === total()}
          onChange={() => props.onChoice(takeFoldersBack(props.choice))}
        />
        <span class={styles.name}>{props.row.name}</span>
      </label>
      <span class={styles.count}>
        {t(I18N_KEYS.PROJECTS_SCOPE_ROOT_COUNT, { covered: covered(), total: total() })}
      </span>
      <Show when={props.answer.root_project_files.length > 0}>
        <span class={styles.badge}>
          {t(I18N_KEYS.PROJECTS_SCOPE_STAGE2, {
            files: props.answer.root_project_files.join(PART_SEPARATOR),
          })}
        </span>
      </Show>
    </div>
  );
}

/** The tree of the folders of the project. */
function Folders(props: {
  answer: ScopeAnswer;
  choice: ScopeChoice;
  row: ProjectRow;
  onChoice: (choice: ScopeChoice) => void;
}) {
  return (
    <section class={styles.section} aria-labelledby="scope-folders">
      <h3 id="scope-folders" class={styles.heading}>
        {t(I18N_KEYS.PROJECTS_SCOPE_FOLDERS)}
      </h3>
      <RootRow {...props} />
      <ul class={styles.tree} aria-label={t(I18N_KEYS.PROJECTS_SCOPE_FOLDERS_LABEL)}>
        <For each={props.answer.folders}>
          {(folder) => (
            <FolderRow folder={folder} choice={props.choice} onChoice={props.onChoice} />
          )}
        </For>
      </ul>
      <Show when={props.answer.folders_more > 0}>
        <p class={styles.muted}>
          {t(I18N_KEYS.PROJECTS_SCOPE_FOLDERS_MORE, { count: props.answer.folders_more })}
        </p>
      </Show>
    </section>
  );
}

/** The lines about what the change takes out of the project. */
function TakenOut(props: { result: Consequences }) {
  return (
    <>
      <For each={props.result.languages}>
        {(entry) => (
          <li>
            {t(I18N_KEYS.PROJECTS_SCOPE_CHANGE_LANGUAGE, {
              count: entry.files,
              language: languageName(entry.language),
            })}
          </li>
        )}
      </For>
      <For each={props.result.folders}>
        {(entry) => (
          <li>
            {t(I18N_KEYS.PROJECTS_SCOPE_CHANGE_FOLDER, { count: entry.files, folder: entry.path })}
          </li>
        )}
      </For>
      <Show when={takesOut(props.result)}>
        <li>{t(I18N_KEYS.PROJECTS_SCOPE_CHANGE_NOTES)}</li>
      </Show>
    </>
  );
}

/** What the change takes out and what it takes back in, before it is applied (mock 26, a2). */
function Changes(props: { answer: ScopeAnswer; choice: ScopeChoice }) {
  const result = () => consequences(props.answer, props.choice);
  return (
    <div role="status" class={styles.changes}>
      <strong>{t(I18N_KEYS.PROJECTS_SCOPE_CHANGES_TITLE)}</strong>
      <ul>
        <TakenOut result={result()} />
        <Show when={result().back.length > 0}>
          <li>
            {t(I18N_KEYS.PROJECTS_SCOPE_CHANGE_BACK, {
              names: result().back.map(languageName).join(PART_SEPARATOR),
            })}
          </li>
        </Show>
      </ul>
    </div>
  );
}

/** The foot: the note, "Reset" and "Save & rescan"; with consequences they read as in mock a2. */
function Foot(props: {
  changed: boolean;
  confirming: boolean;
  saving: boolean;
  onReset: () => void;
  onSave: () => void;
}) {
  return (
    <div class={styles.foot}>
      <span class={styles.muted}>{t(I18N_KEYS.PROJECTS_SCOPE_FOOTER)}</span>
      <button type="button" class={styles.button} disabled={!props.changed} onClick={props.onReset}>
        {props.confirming ? t(I18N_KEYS.PROJECTS_SCOPE_CANCEL) : t(I18N_KEYS.PROJECTS_SCOPE_RESET)}
      </button>
      <button
        type="button"
        class={cx(styles.button, styles.primary)}
        disabled={!props.changed || props.saving}
        onClick={props.onSave}
      >
        {props.saving
          ? t(I18N_KEYS.PROJECTS_SCOPE_SAVING)
          : props.confirming
            ? t(I18N_KEYS.PROJECTS_SCOPE_APPLY)
            : t(I18N_KEYS.PROJECTS_SCOPE_SAVE)}
      </button>
    </div>
  );
}

/** The page once the scope is read: languages, folders, info, consequences and the foot. */
function Loaded(props: { row: ProjectRow; answer: ScopeAnswer; state: ScopeState }) {
  const choice = () => props.state.choice() ?? props.answer.scope;
  const changed = () => differs(props.answer.scope, choice());
  const confirming = () => changed() && takesOut(consequences(props.answer, choice()));
  return (
    <>
      <Languages answer={props.answer} choice={choice()} onChoice={props.state.setChoice} />
      <Folders
        answer={props.answer}
        choice={choice()}
        row={props.row}
        onChoice={props.state.setChoice}
      />
      <p class={styles.info}>{t(I18N_KEYS.PROJECTS_SCOPE_INFO)}</p>
      <Show when={changed()}>
        <Changes answer={props.answer} choice={choice()} />
      </Show>
      <Foot
        changed={changed()}
        confirming={confirming()}
        saving={props.state.saving()}
        onReset={() => props.state.setChoice(props.answer.scope)}
        onSave={() => void props.state.save()}
      />
    </>
  );
}

/** The tab "Scope" (mock 26): which languages and folders the scans of the project cover. The
 * code stays one project; a change applies from the next scan, which it starts. */
export function ScopePanel(props: { row: ProjectRow }) {
  const state = useScopeState(() => props.row.id);
  createEffect(() => {
    void state.load();
  });
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.PROJECTS_SCOPE_TITLE)}>
      <header>
        <h2 class={styles.title}>{t(I18N_KEYS.PROJECTS_SCOPE_TITLE)}</h2>
        <p class={styles.muted}>{t(I18N_KEYS.PROJECTS_SCOPE_SUBTITLE)}</p>
      </header>
      <Show when={state.failure()}>
        {(failure) => (
          <p role="alert" class={styles.error}>
            {failureText(failure())}
          </p>
        )}
      </Show>
      <Show
        when={state.answer()}
        fallback={
          <Show when={!state.failure()}>
            <p class={styles.muted}>{t(I18N_KEYS.PROJECTS_SCOPE_LOADING)}</p>
          </Show>
        }
      >
        {(answer) => <Loaded row={props.row} answer={answer()} state={state} />}
      </Show>
    </section>
  );
}
