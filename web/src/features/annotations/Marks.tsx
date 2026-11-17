import { Show, type JSX } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import {
  CHIP_ICON_SIZE,
  ICON_BOX,
  ICON_STROKE_WIDTH,
  NOTE_PATH,
  SHIELD,
  SHIELD_PATH,
  SHIELD_TICK,
  SHIELD_TICK_WIDTH,
  TASK_RECT,
  TASK_TICK,
} from "./consts";
import type { Counters } from "./types";
import styles from "./Marks.module.css";

const NONE = "none";
const ROUND = "round";
const CURRENT = "currentColor";
/** The tick is drawn in the colour of the surface, so that it cuts into the shield. */
const TICK_COLOUR = "var(--surface)";

/** The shield of a system-critical symbol. It is decoration: the label or the `aria-label` of
 * what carries it says "critical". */
export function Shield(props: { size: number }) {
  return (
    <svg
      class={styles.shield}
      width={props.size}
      height={props.size}
      viewBox={ICON_BOX}
      fill={CURRENT}
      stroke={CURRENT}
      stroke-width={ICON_STROKE_WIDTH}
      stroke-linejoin={ROUND}
      aria-hidden={true}
    >
      <path d={SHIELD_PATH} />
      <path
        d={SHIELD_TICK}
        stroke={TICK_COLOUR}
        fill={NONE}
        stroke-width={SHIELD_TICK_WIDTH}
        stroke-linecap={ROUND}
      />
    </svg>
  );
}

/** The label "Critical" (the word, so the mark never rests on colour alone). `shield` puts the
 * shield in front of it (details and diagrams; the member table shows the label alone, variant B);
 * `short` is the single letter of a diagram node, with the word as its name. */
export function CriticalLabel(props: { shield?: boolean; short?: boolean; size?: number }) {
  const word = () => t(I18N_KEYS.ANNOTATIONS_CRITICAL_LABEL);
  return (
    <span class={styles.label} title={word()}>
      <Show when={props.shield}>
        <Shield size={props.size ?? SHIELD.label} />
      </Show>
      <Show when={props.short} fallback={word()}>
        <span aria-hidden={true}>{t(I18N_KEYS.ANNOTATIONS_CRITICAL_INITIAL)}</span>
        <span class={styles.hidden}>{word()}</span>
      </Show>
    </span>
  );
}

/** The shield alone, for the tree and the tab, with the word as its name. */
export function CriticalShield(props: { size: number; label?: string }) {
  const word = () => props.label ?? t(I18N_KEYS.ANNOTATIONS_CRITICAL_LABEL);
  return (
    <span class={styles.badge} role="img" aria-label={word()} title={word()}>
      <Shield size={props.size} />
    </span>
  );
}

/** The frame of a small outline icon: the notes and the tasks icon draw into it. */
function OutlineIcon(props: { size?: number | undefined; children: JSX.Element }) {
  return (
    <svg
      width={props.size ?? CHIP_ICON_SIZE}
      height={props.size ?? CHIP_ICON_SIZE}
      viewBox={ICON_BOX}
      fill={NONE}
      stroke={CURRENT}
      stroke-width={ICON_STROKE_WIDTH}
      stroke-linejoin={ROUND}
      stroke-linecap={ROUND}
      aria-hidden={true}
    >
      {props.children}
    </svg>
  );
}

/** The icon of the notes chip and the notes tab. */
export function NoteIcon(props: { size?: number }) {
  return (
    <OutlineIcon size={props.size}>
      <path d={NOTE_PATH} />
    </OutlineIcon>
  );
}

/** The icon of the tasks chip and the tasks tab. */
export function TaskIcon(props: { size?: number }) {
  return (
    <OutlineIcon size={props.size}>
      <rect {...TASK_RECT} />
      <path d={TASK_TICK} />
    </OutlineIcon>
  );
}

/** The counter of the notes of a symbol. */
export function NotesChip(props: { count: number }) {
  const name = () => t(I18N_KEYS.ANNOTATIONS_NOTES_COUNT, { count: props.count });
  return (
    <span class={styles.chip} title={name()} role="img" aria-label={name()}>
      <NoteIcon />
      {props.count}
    </span>
  );
}

/** The counter of the tasks of a symbol: done of all. */
export function TasksChip(props: { done: number; total: number }) {
  const name = () => t(I18N_KEYS.ANNOTATIONS_TASKS_COUNT, { done: props.done, total: props.total });
  return (
    <span class={cx(styles.chip, styles.tasks)} title={name()} role="img" aria-label={name()}>
      <TaskIcon />
      {props.done}/{props.total}
    </span>
  );
}

/** The two counters of a symbol, each only if it has something to count. */
export function CounterChips(props: { counters: Counters | undefined }) {
  return (
    <Show when={props.counters}>
      {(counters) => (
        <>
          <Show when={counters().notes > 0}>
            <NotesChip count={counters().notes} />
          </Show>
          <Show when={counters().tasksTotal > 0}>
            <TasksChip done={counters().tasksDone} total={counters().tasksTotal} />
          </Show>
        </>
      )}
    </Show>
  );
}

/** The number of critical symbols below a collapsed row of the tree. */
export function CriticalCount(props: { count: number }) {
  const name = () => t(I18N_KEYS.ANNOTATIONS_CRITICAL_BELOW, { count: props.count });
  return (
    <span class={styles.count} title={name()} role="img" aria-label={name()}>
      <Shield size={CHIP_ICON_SIZE} />
      {props.count}
    </span>
  );
}
