import { fireEvent } from "@solidjs/testing-library";

/** Sends a pointer event with a position and a movement. jsdom has no `PointerEvent`, so a mouse
 * event of the same type stands in; the screens only read the position, the button and the
 * movement. */
export function pointer(
  target: Element,
  type: "pointerdown" | "pointermove" | "pointerup",
  init: {
    clientX?: number;
    clientY?: number;
    button?: number;
    movementX?: number;
    movementY?: number;
  } = {},
): void {
  const { movementX, movementY, ...mouse } = init;
  const event = new MouseEvent(type, { bubbles: true, cancelable: true, ...mouse });
  Object.defineProperties(event, {
    movementX: { value: movementX ?? 0 },
    movementY: { value: movementY ?? 0 },
    pointerId: { value: 1 },
  });
  fireEvent(target, event);
}
