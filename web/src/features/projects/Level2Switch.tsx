import { t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { LEVEL2_PHASE, LEVEL2_STATE, LEVEL2_SWITCH_ID_PREFIX } from "./consts";
import { askRevoke, level2Of, switchOn, type Level2View } from "./level2";
import type { ProjectRow } from "./types";
import styles from "./Level2Switch.module.css";

/** The words and the tone of a state. */
export function stateLook(view: Level2View): { text: MessageKey; tone: PillTone } {
  switch (view.status?.state) {
    case LEVEL2_STATE.on:
      return { text: I18N_KEYS.PROJECTS_LEVEL2_STATE_ON, tone: TONE.good };
    case LEVEL2_STATE.off:
      return { text: I18N_KEYS.PROJECTS_LEVEL2_STATE_OFF, tone: TONE.neutral };
    case LEVEL2_STATE.notEnabled:
      return { text: I18N_KEYS.PROJECTS_LEVEL2_STATE_NOT_ENABLED, tone: TONE.warn };
    case LEVEL2_STATE.locked:
      return { text: I18N_KEYS.PROJECTS_LEVEL2_STATE_LOCKED, tone: TONE.neutral };
    default:
      return { text: I18N_KEYS.PROJECTS_LEVEL2_STATE_UNKNOWN, tone: TONE.neutral };
  }
}

/** Whether the switch stands on: level 2 is on, or a confirmation exists that the missing
 * setting keeps from working (it can be taken back). */
export function isChecked(view: Level2View): boolean {
  const status = view.status;
  return (
    status?.state === LEVEL2_STATE.on ||
    (status?.state === LEVEL2_STATE.notEnabled && status.confirmed)
  );
}

const BUSY: readonly string[] = [
  LEVEL2_PHASE.opening,
  LEVEL2_PHASE.waiting,
  LEVEL2_PHASE.revoking,
  LEVEL2_PHASE.askRevoke,
];

/** Whether a click does nothing now: no state yet, locked, not enabled without a confirmation,
 * or the switch is busy. */
export function isInactive(view: Level2View): boolean {
  const state = view.status?.state;
  const offered = state === LEVEL2_STATE.on || state === LEVEL2_STATE.off || isChecked(view);
  return !offered || BUSY.includes(view.phase);
}

function press(project: number): void {
  const view = level2Of(project);
  if (isInactive(view)) {
    return;
  }
  if (isChecked(view)) {
    askRevoke(project);
  } else {
    void switchOn(project);
  }
}

/** The switch of one project in the table: a button with the role `switch`, so that the keyboard
 * (Tab, Space, Enter) and screen readers work as for any switch. The consent is not given here. */
export function Level2Switch(props: { row: ProjectRow }) {
  const view = () => level2Of(props.row.id);
  return (
    <div class={styles.cell}>
      <button
        type="button"
        role="switch"
        id={`${LEVEL2_SWITCH_ID_PREFIX}${props.row.id}`}
        class={styles.switch}
        aria-checked={isChecked(view())}
        aria-disabled={isInactive(view())}
        aria-busy={view().phase === LEVEL2_PHASE.opening}
        aria-label={t(I18N_KEYS.PROJECTS_LEVEL2_SWITCH_LABEL, { name: props.row.name })}
        onClick={() => press(props.row.id)}
      >
        <span class={styles.thumb} />
      </button>
      <Pill tone={stateLook(view()).tone}>{t(stateLook(view()).text)}</Pill>
    </div>
  );
}
