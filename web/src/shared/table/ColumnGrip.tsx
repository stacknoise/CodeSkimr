import { createSignal } from "solid-js";
import { t } from "../i18n/i18n";
import { I18N_KEYS } from "../i18n/keys";
import { ORIENTATION } from "../layout/consts";
import { cx } from "../ui/cx";
import type { ColumnWidth } from "./columns";
import {
  COLUMN_KEY_DELTA,
  COLUMN_RESET_KEY,
  COLUMN_KEY_STEP_LARGE_PX,
  COLUMN_KEY_STEP_PX,
  NO_DELTA,
} from "./consts";
import styles from "./ColumnGrip.module.css";

/** How far a key press moves the handle (`0` for a key that does nothing). */
function keyMove(event: KeyboardEvent): number {
  const delta = COLUMN_KEY_DELTA[event.key] ?? NO_DELTA;
  return delta * (event.shiftKey ? COLUMN_KEY_STEP_LARGE_PX : COLUMN_KEY_STEP_PX);
}

/** The keys of a handle: `Home` goes back to the first widths, the arrows move the handle. */
function onKey(
  event: KeyboardEvent & { currentTarget: HTMLElement },
  props: { width: ColumnWidth; before?: ((grip: Element) => void) | undefined },
  current: number,
): void {
  if (event.key === COLUMN_RESET_KEY) {
    event.preventDefault();
    props.width.reset();
    return;
  }
  const move = keyMove(event);
  if (move !== NO_DELTA) {
    event.preventDefault();
    props.before?.(event.currentTarget);
    props.width.set(current + move);
  }
}

/** The drag of a handle: where it started and the handlers of the pointer. */
function useDrag(props: { width: ColumnWidth; before?: ((grip: Element) => void) | undefined }) {
  const [dragging, setDragging] = createSignal(false);
  let origin = { pointer: 0, size: 0 };
  return {
    dragging,
    down: (event: PointerEvent & { currentTarget: HTMLElement }) => {
      event.preventDefault();
      event.stopPropagation();
      props.before?.(event.currentTarget);
      origin = { pointer: event.clientX, size: props.width.size() };
      event.currentTarget.setPointerCapture?.(event.pointerId);
      setDragging(true);
    },
    move: (event: PointerEvent) => {
      if (dragging()) {
        props.width.set(origin.size + event.clientX - origin.pointer);
      }
    },
    stop: () => setDragging(false),
  };
}

/** The handle on the right edge of a head cell: drag it or use the arrow keys to change the width of
 * the column, double-click to go back to the first widths. `before` runs ahead of every change, so
 * that a table that sizes itself can fix its widths first. */
export function ColumnGrip(props: {
  width: ColumnWidth;
  name: string;
  before?: (grip: Element) => void;
}) {
  const drag = useDrag(props);
  // A table that sizes itself has no width yet: the width on the screen is the value until it has.
  const [shown, setShown] = createSignal(0);
  const value = () => (props.width.size() > 0 ? props.width.size() : shown());
  return (
    <span
      class={cx(styles.grip, drag.dragging() && styles.dragging)}
      role="separator"
      tabindex="0"
      aria-orientation={ORIENTATION.vertical}
      aria-label={t(I18N_KEYS.TABLE_COL_RESIZE, { name: props.name })}
      aria-valuenow={value() > 0 ? value() : undefined}
      title={t(I18N_KEYS.TABLE_COL_RESET_TIP)}
      onFocus={(event) => setShown(event.currentTarget.parentElement?.offsetWidth ?? 0)}
      aria-valuemin={props.width.bounds.min}
      aria-valuemax={props.width.bounds.max}
      onClick={(event) => event.stopPropagation()}
      onPointerDown={drag.down}
      onPointerMove={drag.move}
      onPointerUp={drag.stop}
      onPointerCancel={drag.stop}
      onKeyDown={(event) => onKey(event, props, value())}
      onDblClick={() => props.width.reset()}
    />
  );
}
