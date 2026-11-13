import { createEffect, Match, on, Show, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { DetailPane } from "./DetailPane";
import {
  applyFocus,
  collapseAll,
  currentType,
  loadOutline,
  outlineError,
  outlineLoading,
  pendingFocus,
  symbolDetails,
  symbolError,
  treeOutline,
  type Failure,
} from "./explorer";
import { useExplorerKeys } from "./keys";
import { Overview } from "./Overview";
import { TreePane } from "./TreePane";
import { useExplorer } from "./useExplorer";
import styles from "./ExplorerScreen.module.css";

function Message(props: { title?: string; text: string; alert?: boolean }) {
  return (
    <div class={styles.message} role={props.alert ? "alert" : undefined}>
      <Show when={props.title}>{(title) => <h2>{title()}</h2>}</Show>
      <p>{props.text}</p>
    </div>
  );
}

function failureText(failure: Failure): string {
  return failure.code === null
    ? t(I18N_KEYS.EXPLORER_LOAD_FAILED)
    : errorText(failure.code as ErrorCode);
}

/** The note under the filter while the tree has no rows: reading, or nothing to show. */
function treeNote(rowCount: number): string | null {
  if (outlineLoading()) {
    return t(I18N_KEYS.EXPLORER_LOADING);
  }
  return treeOutline() !== null && rowCount === 0 ? t(I18N_KEYS.EXPLORER_NO_TYPES) : null;
}

/** The middle column: the failure of a read, the selected type, or the overview. */
function Details(props: { onPickMember: ReturnType<typeof useExplorer>["pickMember"] }) {
  const info = () => {
    const type = currentType();
    return type === null ? null : (symbolDetails().get(type) ?? null);
  };
  return (
    <Switch>
      <Match when={symbolError()}>
        {(failure) => (
          <div class={styles.centered}>
            <Message text={failureText(failure())} alert />
          </div>
        )}
      </Match>
      <Match when={info()}>
        {(detail) => (
          <DetailPane
            info={detail()}
            thresholds={treeOutline()?.thresholds ?? null}
            onPickMember={(row) => props.onPickMember(detail().id, row)}
          />
        )}
      </Match>
      <Match when={true}>
        <div class={styles.centered}>
          <Show when={treeOutline()?.snapshot === null}>
            <Message text={t(I18N_KEYS.EXPLORER_NOT_SCANNED)} />
          </Show>
          <Overview outline={treeOutline()?.snapshot === null ? null : treeOutline()} />
        </div>
      </Match>
    </Switch>
  );
}

/** Reads the tree when the active project changes, and goes where another screen asked once the
 * tree is there. */
function useReadingOfTheProject(): void {
  createEffect(
    on(activeProject, (id) => {
      if (id !== null) {
        void loadOutline(id);
      }
    }),
  );
  createEffect(() => {
    const tree = treeOutline();
    const project = activeProject();
    if (tree !== null && project !== null && pendingFocus() !== null) {
      void applyFocus(project, tree);
    }
  });
}

/** Screen 2, the explorer: tree and details (blueprint B.2, mock `Explorer.dc.html`). The source
 * column of the mock follows with the next work package. */
export function ExplorerScreen() {
  const explorer = useExplorer();
  useReadingOfTheProject();
  useExplorerKeys({ collapseAll, reveal: explorer.reveal });
  return (
    <Switch>
      <Match when={activeProject() === null}>
        <div class={styles.centered}>
          <Message
            title={t(I18N_KEYS.EXPLORER_NO_PROJECT_TITLE)}
            text={t(I18N_KEYS.EXPLORER_NO_PROJECT_TEXT)}
          />
        </div>
      </Match>
      <Match when={outlineError()}>
        {(failure) => (
          <div class={styles.centered}>
            <Message text={failureText(failure())} alert />
          </div>
        )}
      </Match>
      <Match when={true}>
        <div class={styles.screen}>
          <TreePane
            rows={explorer.rows()}
            cursor={explorer.cursor()}
            actions={explorer.actions}
            note={treeNote(explorer.rows().length)}
          />
          <Details onPickMember={explorer.pickMember} />
        </div>
      </Match>
    </Switch>
  );
}
