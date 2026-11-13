import { createEffect, createMemo, createSignal, Match, on, Show, Switch } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { activeProject } from "../../shared/project/active";
import { ROW } from "./consts";
import { DetailPane } from "./DetailPane";
import {
  collapseAll,
  currentMember,
  currentType,
  explorerFailure,
  loadOutline,
  openRow,
  selectType,
  symbolDetails,
  toggleRow,
  treeOutline,
} from "./explorer";
import { stepIndex, useExplorerKeys } from "./keys";
import { revealKeys, typeKey, type TreeRow } from "./tree";
import { TreePane, useRows } from "./TreePane";
import styles from "./ExplorerScreen.module.css";

function Message(props: { title?: string; text: string }) {
  return (
    <div class={styles.message}>
      <Show when={props.title}>{(title) => <h2>{title()}</h2>}</Show>
      <p>{props.text}</p>
    </div>
  );
}

/** What the middle column shows while there is no type to show. */
function Empty(props: { snapshot: number | null | undefined }) {
  return (
    <Switch fallback={<Message text={t(I18N_KEYS.EXPLORER_NOTHING_SELECTED)} />}>
      <Match when={props.snapshot === null}>
        <Message text={t(I18N_KEYS.EXPLORER_NOT_SCANNED)} />
      </Match>
    </Switch>
  );
}

/** A click on a row: a type is shown first and folded on the second click, a member is shown,
 * every other row folds. */
function act(project: number, row: TreeRow): void {
  const shown = row.kind === ROW.type && row.type === currentType() && currentMember() === null;
  if ((row.kind === ROW.type || row.kind === ROW.member) && row.type !== null && !shown) {
    void selectType(project, row.type, row.member);
  } else if (row.expandable && row.kind !== ROW.member) {
    toggleRow(row.key);
  }
}

/** Opens (`close` false) or closes (`close` true) the row under the cursor, if it can. */
function fold(rows: TreeRow[], cursor: string | null, close: boolean): void {
  const row = rows.find((candidate) => candidate.key === cursor);
  if (row?.expandable && row.open === close) {
    toggleRow(row.key);
  }
}

/** Screen 2, the explorer: tree and details (blueprint B.2, mock `Explorer.dc.html`). The source
 * column of the mock follows with the next work package. */
export function ExplorerScreen() {
  const project = () => activeProject();
  const rows = useRows();
  const [cursor, setCursor] = createSignal<string | null>(null);
  createEffect(
    on(project, (id) => {
      if (id !== null) {
        void loadOutline(id);
      }
    }),
  );
  const info = createMemo(() => {
    const type = currentType();
    return type === null ? null : (symbolDetails().get(type) ?? null);
  });
  const pick = (row: TreeRow) => {
    setCursor(row.key);
    const id = project();
    if (id !== null) {
      act(id, row);
    }
  };
  const reveal = () => {
    const outline = treeOutline();
    const type = currentType();
    if (outline && type !== null) {
      revealKeys(outline, type).forEach((key) => key !== typeKey(type) && openRow(key));
    }
  };
  useExplorerKeys({
    move: (step) => {
      const list = rows();
      const next =
        list[
          stepIndex(
            list.map((row) => row.key),
            cursor(),
            step,
          )
        ];
      if (next) {
        pick({ ...next, expandable: false });
      }
    },
    open: () => fold(rows(), cursor(), false),
    close: () => fold(rows(), cursor(), true),
    collapseAll,
    reveal,
  });
  return (
    <Switch>
      <Match when={project() === null}>
        <div class={styles.centered}>
          <Message
            title={t(I18N_KEYS.EXPLORER_NO_PROJECT_TITLE)}
            text={t(I18N_KEYS.EXPLORER_NO_PROJECT_TEXT)}
          />
        </div>
      </Match>
      <Match when={explorerFailure()}>
        {(failure) => (
          <div class={styles.centered}>
            <Message
              text={
                failure().code === null
                  ? t(I18N_KEYS.EXPLORER_LOAD_FAILED)
                  : errorText(failure().code as ErrorCode)
              }
            />
          </div>
        )}
      </Match>
      <Match when={true}>
        <div class={styles.screen}>
          <TreePane rows={rows()} onPick={pick} onReveal={reveal} />
          <Show
            when={info()}
            fallback={
              <div class={styles.centered}>
                <Empty snapshot={treeOutline()?.snapshot} />
              </div>
            }
          >
            {(detail) => (
              <DetailPane
                info={detail()}
                onPickMember={(member) => {
                  const id = project();
                  if (id !== null) {
                    void selectType(id, detail().id, member);
                  }
                }}
              />
            )}
          </Show>
        </div>
      </Match>
    </Switch>
  );
}
