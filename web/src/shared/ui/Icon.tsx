import { For } from "solid-js";
import {
  ICON_NONE,
  ICON_PATHS,
  ICON_ROUND,
  ICON_SIZE,
  ICON_STROKE,
  ICON_STROKE_WIDTH,
  ICON_VIEW_BOX,
} from "./consts";

/** Name of an icon. */
export type IconName = keyof typeof ICON_PATHS;

/** An icon of the rail. It is decoration: the text next to it, or the label of its button,
 * carries the meaning. */
export function Icon(props: { name: IconName; size?: number }) {
  return (
    <svg
      width={props.size ?? ICON_SIZE}
      height={props.size ?? ICON_SIZE}
      viewBox={ICON_VIEW_BOX}
      fill={ICON_NONE}
      stroke={ICON_STROKE}
      stroke-width={ICON_STROKE_WIDTH}
      stroke-linecap={ICON_ROUND}
      stroke-linejoin={ICON_ROUND}
      aria-hidden={true}
    >
      <For each={ICON_PATHS[props.name]}>{(d) => <path d={d} />}</For>
    </svg>
  );
}
