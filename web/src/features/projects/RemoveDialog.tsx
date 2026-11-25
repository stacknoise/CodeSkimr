import { onCleanup, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { CANCEL_KEY } from "../scan/consts";
import { FOCUSABLE, REMOVE_DIALOG_TITLE_ID, REMOVE_MARKS, SOURCE_KIND, TAB_KEY } from "./consts";
import { size } from "./format";
import type { ProjectRow } from "./types";
import styles from "./RemoveDialog.module.css";

/** One consequence: a sign for "goes" or "stays" and the sentence. */
function Effect(props: { goes?: boolean; text: string }) {
  return (
    <li class={styles.effect}>
      <span class={props.goes ? styles.goes : styles.stays} aria-hidden="true">
        {props.goes ? REMOVE_MARKS.goes : REMOVE_MARKS.stays}
      </span>
      <span>{props.text}</span>
    </li>
  );
}

/** What the removal deletes: the project file, with its size when it is known. */
function dataText(row: ProjectRow): string {
  const bytes = row.database?.size_bytes;
  return bytes === undefined
    ? t(I18N_KEYS.PROJECTS_REMOVE_EFFECT_DATA)
    : t(I18N_KEYS.PROJECTS_REMOVE_EFFECT_DATA_SIZE, { size: size(bytes, currentLanguage()) });
}

/** The head: the warning sign, the question and where the project lies. */
function Head(props: { row: ProjectRow }) {
  return (
    <div class={styles.head}>
      <span class={styles.sign} aria-hidden="true">
        {REMOVE_MARKS.warning}
      </span>
      <div>
        <h2 id={REMOVE_DIALOG_TITLE_ID} class={styles.title}>
          {t(I18N_KEYS.PROJECTS_REMOVE_TITLE, { name: props.row.name })}
        </h2>
        <p class={styles.place}>{props.row.location}</p>
      </div>
    </div>
  );
}

/** The consequences: what goes and what stays. */
function Effects(props: { row: ProjectRow }) {
  return (
    <ul class={styles.effects}>
      <Effect goes text={dataText(props.row)} />
      <Effect text={t(I18N_KEYS.PROJECTS_REMOVE_EFFECT_FILES)} />
      <Show when={props.row.kind === SOURCE_KIND.git}>
        <Effect text={t(I18N_KEYS.PROJECTS_REMOVE_EFFECT_CREDENTIALS)} />
      </Show>
    </ul>
  );
}

/** Keeps Tab inside the dialog: from the last button it goes to the first, and back. */
function trapTab(event: KeyboardEvent, dialog: HTMLElement | undefined): void {
  if (event.key !== TAB_KEY || dialog === undefined) {
    return;
  }
  const buttons = Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE));
  const first = buttons[0];
  const last = buttons[buttons.length - 1];
  const edge = event.shiftKey ? first : last;
  if (first !== undefined && last !== undefined && document.activeElement === edge) {
    event.preventDefault();
    (event.shiftKey ? last : first).focus();
  }
}

/** Focus of the dialog: it starts on "Cancel", stays inside, and goes back to where it was. */
function useFocus(dialog: () => HTMLElement | undefined, cancel: () => HTMLElement | undefined) {
  const before = document.activeElement;
  useKeydown((event) => trapTab(event, dialog()));
  onMount(() => cancel()?.focus());
  onCleanup(() => {
    if (before instanceof HTMLElement && before.isConnected) {
      before.focus();
    }
  });
}

/** The dialog "Remove project" (mock 24, D1): it says what goes and what stays. Esc and "Cancel"
 * close it, the focus starts on "Cancel", so a stray Enter removes nothing. */
export function RemoveDialog(props: { row: ProjectRow; onYes: () => void; onNo: () => void }) {
  let dialog: HTMLDivElement | undefined;
  let cancel: HTMLButtonElement | undefined;
  useLayer(() => true);
  useKeydown((event) => {
    if (event.key === CANCEL_KEY) {
      props.onNo();
    }
  });
  useFocus(
    () => dialog,
    () => cancel,
  );
  return (
    <Portal>
      <div class={styles.scrim} onClick={props.onNo} />
      <div
        ref={dialog}
        class={styles.dialog}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={REMOVE_DIALOG_TITLE_ID}
      >
        <Head row={props.row} />
        <Effects row={props.row} />
        <div class={styles.foot}>
          <span class={styles.keys}>{t(I18N_KEYS.PROJECTS_REMOVE_KEYS)}</span>
          <button type="button" class={styles.cancel} ref={cancel} onClick={props.onNo}>
            {t(I18N_KEYS.PROJECTS_REMOVE_CANCEL)}
          </button>
          <button type="button" class={styles.danger} onClick={props.onYes}>
            {t(I18N_KEYS.PROJECTS_REMOVE_CONFIRM)}
          </button>
        </div>
      </div>
    </Portal>
  );
}
