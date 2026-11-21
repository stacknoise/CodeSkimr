import { createSignal } from "solid-js";
import { t } from "../i18n/i18n";
import { I18N_KEYS } from "../i18n/keys";
import { ORIENTATION } from "../layout/consts";
import { cx } from "../ui/cx";
import type { ColumnWidth } from "./columns";
import { COLUMN_KEY_DELTA, COLUMN_KEY_STEP_LARGE_PX, COLUMN_KEY_STEP_PX, NO_DELTA } from "./consts";
import styles from "./ColumnGrip.module.css";

/** How far a key press moves the handle (`0` for a key that does nothing). */
function keyMove(event: KeyboardEvent): number {
  const delta = COLUMN_KEY_DELTA[event.key] ?? NO_DELTA;
  return delta * (event.shiftKey ? COLUMN_KEY_STEP_LARGE_PX : COLUMN_KEY_STEP_PX);
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
  return (
    <span
      class={cx(styles.grip, drag.dragging() && styles.dragging)}
      role="separator"
      tabindex="0"
      aria-orientation={ORIENTATION.vertical}
      aria-label={t(I18N_KEYS.TABLE_COL_RESIZE, { name: props.name })}
      aria-valuenow={props.width.size() > 0 ? props.width.size() : undefined}
      aria-valuemin={props.width.bounds.min}
      aria-valuemax={props.width.bounds.max}
      onClick={(event) => event.stopPropagation()}
      onPointerDown={drag.down}
      onPointerMove={drag.move}
      onPointerUp={drag.stop}
      onPointerCancel={drag.stop}
      onKeyDown={(event) => {
        const move = keyMove(event);
        if (move !== NO_DELTA) {
          event.preventDefault();
          props.before?.(event.currentTarget);
          props.width.set(props.width.size() + move);
        }
      }}
      onDblClick={() => props.width.reset()}
    />
  );
}
