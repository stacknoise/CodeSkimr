import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { BOX_DONE, BOX_OPEN, KIND, STATUS } from "./consts";
import { dueDate, shortDate } from "./format";
import { RichText } from "./RichText";
import { setShowAnnotationText, showAnnotationText } from "./flags";
import type { Annotation } from "./types";
import styles from "./InlineText.module.css";

/** The line of one note or task under a row of the member table (mock 14, second board): the word
 * "Note" or "Task", the text, and who and when or the due date. */
function Line(props: { item: Annotation }) {
  const item = () => props.item;
  const isNote = () => item().kind === KIND.note;
  const done = () => item().status === STATUS.done;
  return (
    <div class={styles.line}>
      <b>{t(isNote() ? I18N_KEYS.ANNOTATIONS_INLINE_NOTE : I18N_KEYS.ANNOTATIONS_INLINE_TASK)}</b>
      <Show when={!isNote()}>
        <span aria-hidden={true}>{done() ? BOX_DONE : BOX_OPEN}</span>
      </Show>
      <RichText text={isNote() ? (item().text ?? "") : (item().title ?? "")} />
      <span class={styles.muted}>
        <Show
          when={isNote()}
          fallback={
            <Show when={item().due}>
              {(due) =>
                t(I18N_KEYS.ANNOTATIONS_INLINE_DUE, { date: dueDate(due(), currentLanguage()) })
              }
            </Show>
          }
        >
          {t(I18N_KEYS.ANNOTATIONS_BY, {
            author: item().author,
            date: shortDate(item().updated, currentLanguage()),
          })}
        </Show>
      </span>
    </div>
  );
}

/** The notes and tasks of one symbol as text, for the second board of mock 14. */
export function InlineText(props: { items: readonly Annotation[] }) {
  return (
    <div class={styles.inline}>
      <For each={props.items}>{(item) => <Line item={item} />}</For>
    </div>
  );
}

/** The switch "Show annotations": the text of notes and tasks under their rows. */
export function ShowTextSwitch() {
  return (
    <label class={styles.switch}>
      <input
        type="checkbox"
        checked={showAnnotationText()}
        onChange={(event) => setShowAnnotationText(event.currentTarget.checked)}
      />
      {t(I18N_KEYS.ANNOTATIONS_SHOW_TEXT)}
    </label>
  );
}
