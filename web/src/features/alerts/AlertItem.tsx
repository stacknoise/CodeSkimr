import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { deleteAnnotation, updateAnnotation } from "../annotations/api";
import { showTarget } from "../annotations/jump";
import { SHIELD } from "../annotations/consts";
import { Shield } from "../annotations/Marks";
import { CLOSE_MARK, KIND, KIND_TEXT, MS_PER_SECOND, STATE } from "./consts";
import { changeMark, dismiss, hideDrawer } from "./store";
import { metaOf, sentenceOf, whenText } from "./text";
import type { Alert } from "./types";
import styles from "./Drawer.module.css";

/** What an orphaned target offers: assign the mark to the closest symbol, or take it off. */
function OrphanActions(props: { project: number; alert: Alert }) {
  const closest = () => props.alert.closest;
  return (
    <div class={styles.actions}>
      <Show when={closest()}>
        {(name) => (
          <button
            type="button"
            class={styles.link}
            onClick={() =>
              void changeMark(props.project, () =>
                updateAnnotation(props.project, props.alert.annotation, {
                  target: { kind: props.alert.target.kind, name: name() },
                }),
              )
            }
          >
            {t(I18N_KEYS.ALERTS_REASSIGN)}
          </button>
        )}
      </Show>
      <button
        type="button"
        class={styles.link}
        onClick={() =>
          void changeMark(props.project, () =>
            deleteAnnotation(props.project, props.alert.annotation),
          )
        }
      >
        {t(I18N_KEYS.ALERTS_REMOVE_MARK)}
      </button>
    </div>
  );
}

/** The name of the symbol of an alert: a button that shows it in the explorer. */
function Name(props: { project: number; alert: Alert }) {
  const jump = () => {
    void showTarget(props.project, props.alert.target).then((found) => {
      if (found) {
        hideDrawer();
      }
    });
  };
  return (
    <button type="button" class={styles.name} onClick={jump}>
      <span
        class={cx(styles.tag, props.alert.kind === KIND.changed ? styles.changed : styles.gone)}
      >
        {t(KIND_TEXT[props.alert.kind])}
      </span>
      <b class={styles.symbol}>{props.alert.target.name}</b>
    </button>
  );
}

/** One alert of the drawer (mock 15): the shield, what happened to which symbol, when and how it
 * was found, and the cross that dismisses it. */
export function AlertItem(props: { project: number; alert: Alert }) {
  const now = () => Date.now() / MS_PER_SECOND;
  const open = () => props.alert.state === STATE.open;
  return (
    <li class={cx(styles.msg, open() && styles.unread)}>
      <span class={styles.icon}>
        <Shield size={SHIELD.tab} />
      </span>
      <div class={styles.text}>
        <Name project={props.project} alert={props.alert} />
        <p class={styles.sentence}>{sentenceOf(props.alert)}</p>
        <Show when={props.alert.reason}>
          {(reason) => (
            <p class={styles.reason}>{t(I18N_KEYS.ALERTS_REASON, { reason: reason() })}</p>
          )}
        </Show>
        <p class={styles.meta}>
          {open()
            ? metaOf(props.alert, now())
            : t(I18N_KEYS.ALERTS_DISMISSED_ON, {
                when: whenText(props.alert.dismissed ?? props.alert.last_seen, now()),
              })}
        </p>
        <Show when={open() && props.alert.kind === KIND.orphaned}>
          <OrphanActions project={props.project} alert={props.alert} />
        </Show>
      </div>
      <Show when={open()}>
        <button
          type="button"
          class={styles.x}
          aria-label={t(I18N_KEYS.ALERTS_DISMISS, { name: props.alert.target.name })}
          onClick={() => void dismiss(props.project, props.alert.id)}
        >
          {CLOSE_MARK}
        </button>
      </Show>
    </li>
  );
}
