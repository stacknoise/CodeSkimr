import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { CriticalLabel } from "./Marks";
import { markAsCritical, removeCriticalMark, report } from "./panel";
import { RichText } from "./RichText";
import { SHIELD } from "./consts";
import type { Annotation, Target } from "./types";
import styles from "./Tabs.module.css";

/** The mark of a symbol that has one: the label, the reason and the button that takes it off. */
function Marked(props: { project: number; target: Target; mark: Annotation | undefined }) {
  const remove = async () =>
    report(
      await removeCriticalMark(props.project, props.target),
      t(I18N_KEYS.ANNOTATIONS_NOTICE_UNMARKED),
    );
  return (
    <>
      <p class={styles.statement}>
        <CriticalLabel shield size={SHIELD.label} />
        {t(I18N_KEYS.ANNOTATIONS_CRITICAL_ON)}
      </p>
      <Show when={props.mark?.text}>
        {(reason) => (
          <p class={styles.reason}>
            <RichText text={reason()} />
          </p>
        )}
      </Show>
      <p class={styles.explain}>{t(I18N_KEYS.ANNOTATIONS_CRITICAL_EXPLAIN)}</p>
      <button type="button" class={styles.primary} onClick={() => void remove()}>
        {t(I18N_KEYS.ANNOTATIONS_UNMARK)}
      </button>
    </>
  );
}

/** The symbol has no mark yet: the optional reason and the button that sets it. */
function Unmarked(props: { project: number; target: Target }) {
  const [reason, setReason] = createSignal("");
  const mark = async () =>
    report(
      await markAsCritical(props.project, props.target, reason()),
      t(I18N_KEYS.ANNOTATIONS_NOTICE_MARKED),
    );
  return (
    <>
      <p class={styles.statement}>{t(I18N_KEYS.ANNOTATIONS_CRITICAL_OFF)}</p>
      <p class={styles.explain}>{t(I18N_KEYS.ANNOTATIONS_CRITICAL_EXPLAIN)}</p>
      <label class={styles.field}>
        <span>{t(I18N_KEYS.ANNOTATIONS_CRITICAL_REASON)}</span>
        <input
          class={styles.input}
          value={reason()}
          onInput={(event) => setReason(event.currentTarget.value)}
        />
      </label>
      <button type="button" class={styles.primary} onClick={() => void mark()}>
        {t(I18N_KEYS.ANNOTATIONS_MARK)}
      </button>
    </>
  );
}

/** The tab "Critical" of the side panel (mock 14: the shield alone is its name). */
export function CriticalTab(props: {
  project: number;
  target: Target;
  marked: boolean;
  mark: Annotation | undefined;
}) {
  return (
    <div class={styles.content}>
      <Show
        when={props.marked}
        fallback={<Unmarked project={props.project} target={props.target} />}
      >
        <Marked project={props.project} target={props.target} mark={props.mark} />
      </Show>
      <p class={styles.keys}>{t(I18N_KEYS.ANNOTATIONS_KEYS_HINT)}</p>
    </div>
  );
}
