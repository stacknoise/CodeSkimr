import {
  END,
  FIT_PADDING,
  HALF,
  PADDING_SIDES,
  MAX_SCALE,
  MIN_SCALE,
  PX,
  SCALE,
  SEPARATOR,
  TRANSLATE,
  WHEEL_NOTCH,
  ZOOM_STEP,
} from "./consts";
import type { Size, View } from "./types";

function clamp(scale: number): number {
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, scale));
}

/** The view that shows the whole picture, centred, as large as fits (never larger than 1:1). */
export function fit(content: Size, area: Size): View {
  if (content.width <= 0 || content.height <= 0 || area.width <= 0 || area.height <= 0) {
    return { scale: 1, x: 0, y: 0 };
  }
  const scale = clamp(
    Math.min(
      (area.width - FIT_PADDING * PADDING_SIDES) / content.width,
      (area.height - FIT_PADDING * PADDING_SIDES) / content.height,
      1,
    ),
  );
  return {
    scale,
    x: (area.width - content.width * scale) * HALF,
    y: (area.height - content.height * scale) * HALF,
  };
}

/** Zooms by `notches` wheel steps (positive zooms out) so that the point `at` of the area stays
 * where it is. */
export function zoomAt(view: View, notches: number, at: { x: number; y: number }): View {
  const scale = clamp(view.scale * Math.pow(ZOOM_STEP, -notches));
  const ratio = scale / view.scale;
  return { scale, x: at.x - (at.x - view.x) * ratio, y: at.y - (at.y - view.y) * ratio };
}

/** The number of wheel notches of a wheel event distance. */
export function notches(deltaY: number): number {
  return deltaY / WHEEL_NOTCH;
}

/** Moves the picture by a pointer distance. */
export function pan(view: View, dx: number, dy: number): View {
  return { ...view, x: view.x + dx, y: view.y + dy };
}

/** The CSS `transform` that puts `view` on the picture. */
export function transformOf(view: View): string {
  return TRANSLATE + view.x + PX + SEPARATOR + view.y + PX + SCALE + view.scale + END;
}
