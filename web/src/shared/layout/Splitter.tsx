import { createSignal } from "solid-js";
import { t } from "../i18n/i18n";
import { I18N_KEYS } from "../i18n/keys";
import { cx } from "../ui/cx";
import {
  KEY_STEP_LARGE_PX,
  KEY_STEP_PX,
  ORIENTATION,
  SEPARATOR_KEY,
  SIDE,
  type Orientation,
  type Side,
} from "./consts";
import type { PaneSize } from "./paneSize";
import styles from "./Splitter.module.css";

/** How far a key press moves the separator, and in which direction (`0` for none). */
export function keyDelta(event: KeyboardEvent): number {
  const step = event.shiftKey ? KEY_STEP_LARGE_PX : KEY_STEP_PX;
  if ((SEPARATOR_KEY.increase as readonly string[]).includes(event.key)) {
    return step;
  }
  return (SEPARATOR_KEY.decrease as readonly string[]).includes(event.key) ? -step : 0;
}

/** The arrow keys move the handle, `Home` and `End` take it to its ends. */
function onKey(event: KeyboardEvent, pane: PaneSize, side: Side): void {
  const delta = keyDelta(event);
  if (delta !== 0) {
    event.preventDefault();
    pane.set(pane.size() + side * delta);
  } else if (event.key === SEPARATOR_KEY.smallest || event.key === SEPARATOR_KEY.largest) {
    event.preventDefault();
    const smallest = event.key === SEPARATOR_KEY.smallest;
    pane.set(smallest === (side === SIDE.before) ? pane.bounds.min : pane.bounds.max);
  }
}

/** The drag of the handle: where it started and the handlers of the pointer. */
function useDrag(pane: PaneSize, orientation: Orientation, side: Side) {
  const [dragging, setDragging] = createSignal(false);
  let origin = { pointer: 0, size: 0 };
  const along = (event: PointerEvent) =>
    orientation === ORIENTATION.vertical ? event.clientX : event.clientY;
  return {
    dragging,
    down: (event: PointerEvent & { currentTarget: HTMLElement }) => {
      event.preventDefault();
      origin = { pointer: along(event), size: pane.size() };
      event.currentTarget.setPointerCapture?.(event.pointerId);
      setDragging(true);
    },
    move: (event: PointerEvent) => {
      if (dragging()) {
        pane.set(origin.size + side * (along(event) - origin.pointer));
      }
    },
    stop: () => setDragging(false),
  };
}

/** The thin handle between two panes. Dragging it, or pressing the arrow keys on it, changes the
 * size of `pane`; a double click goes back to the first size. `side` says on which side of the
 * handle the pane lies. The handle takes no room of its own: it lies over the border. */
export function Splitter(props: { pane: PaneSize; orientation: Orientation; side: Side }) {
  const drag = useDrag(props.pane, props.orientation, props.side);
  const vertical = () => props.orientation === ORIENTATION.vertical;
  const label = () => t(vertical() ? I18N_KEYS.APP_RESIZE_WIDTH : I18N_KEYS.APP_RESIZE_HEIGHT);
  return (
    <div
      class={cx(
        styles.splitter,
        vertical() ? styles.vertical : styles.horizontal,
        drag.dragging() && styles.dragging,
      )}
      role="separator"
      tabindex={0}
      aria-orientation={props.orientation}
      aria-label={label()}
      aria-valuenow={props.pane.size()}
      aria-valuemin={props.pane.bounds.min}
      aria-valuemax={props.pane.bounds.max}
      onPointerDown={drag.down}
      onPointerMove={drag.move}
      onPointerUp={drag.stop}
      onPointerCancel={drag.stop}
      onDblClick={() => props.pane.reset()}
      onKeyDown={(event) => onKey(event, props.pane, props.side)}
    />
  );
}
