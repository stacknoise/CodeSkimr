import { For, onCleanup, onMount, Show } from "solid-js";
import { cx } from "../../shared/ui/cx";
import {
  ARROW,
  BOX_COLOR_PROPERTY,
  EDGE,
  MARKER_FILL,
  MARKER_IDS,
  NO_FILL,
  PX,
  URL_CLOSE,
  URL_OPEN,
} from "./consts";
import { edgeKind, edgePath, rankColor, shown, styleOf } from "./geometry";
import type { GraphData, LayerLink, Size, View } from "./types";
import { useNavigation } from "./useNavigation";
import { transformOf } from "./viewport";
import styles from "./GraphCanvas.module.css";

function ArrowHeads() {
  return (
    <defs>
      <For each={MARKER_IDS}>
        {(id) => (
          <marker
            id={id}
            viewBox={ARROW.viewBox}
            refX={ARROW.refX}
            refY={ARROW.refY}
            markerWidth={ARROW.size}
            markerHeight={ARROW.size}
            orient={ARROW.orient}
          >
            <path d={ARROW.shape} fill={MARKER_FILL[id]} />
          </marker>
        )}
      </For>
    </defs>
  );
}

function Edge(props: {
  link: LayerLink;
  data: GraphData;
  selected: string | null;
  violationsOnly: boolean;
  weak: boolean;
}) {
  const from = () => props.data.boxes.find((box) => box.name === props.link.from);
  const to = () => props.data.boxes.find((box) => box.name === props.link.to);
  const kind = () => edgeKind(props.link, props.selected);
  const look = () => styleOf(kind());
  const opacity = () => (kind() === EDGE.weak && !props.weak ? 1 : look().opacity);
  return (
    <Show when={shown(props.link, props.violationsOnly) ? [from(), to()] : undefined}>
      {(ends) => (
        <Show when={ends()[0] && ends()[1]}>
          <path
            d={edgePath(ends()[0]!, ends()[1]!)}
            fill={NO_FILL}
            stroke={look().color}
            stroke-width={look().width}
            stroke-dasharray={look().dash}
            marker-end={URL_OPEN + look().marker + URL_CLOSE}
            opacity={opacity()}
          />
        </Show>
      )}
    </Show>
  );
}

/** The drawing of the map: boxes as buttons, edges as curves, moved and scaled by `view`. The
 * wheel zooms around the pointer, and a drag with the space key or the middle button pans. */
export function GraphCanvas(props: {
  data: GraphData;
  selected: string | null;
  onSelect: (name: string) => void;
  view: View;
  onView: (view: View) => void;
  onSize: (size: Size) => void;
  violationsOnly: boolean;
  weak: boolean;
}) {
  let area: HTMLDivElement | undefined;
  const navigation = useNavigation(
    () => area,
    () => props.view,
    props.onView,
  );
  onMount(() => {
    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => area && props.onSize(sizeOf(area)));
    if (area) {
      observer?.observe(area);
      props.onSize(sizeOf(area));
    }
    onCleanup(() => observer?.disconnect());
  });
  return (
    <div
      ref={area}
      class={cx(styles.area, navigation.grabbing() && styles.grab)}
      onPointerDown={navigation.start}
      onPointerMove={navigation.move}
      onPointerUp={navigation.stop}
      onPointerCancel={navigation.stop}
    >
      <div
        class={styles.stage}
        style={{
          width: props.data.width + PX,
          height: props.data.height + PX,
          transform: transformOf(props.view),
        }}
      >
        <svg class={styles.edges} width={props.data.width} height={props.data.height} aria-hidden>
          <ArrowHeads />
          <For each={props.data.links}>
            {(link) => (
              <Edge
                link={link}
                data={props.data}
                selected={props.selected}
                violationsOnly={props.violationsOnly}
                weak={props.weak}
              />
            )}
          </For>
        </svg>
        <For each={props.data.boxes}>
          {(box) => (
            <button
              type="button"
              class={cx(styles.box, props.selected === box.name && styles.selected)}
              style={{
                left: box.x + PX,
                top: box.y + PX,
                width: box.width + PX,
                height: box.height + PX,
                [BOX_COLOR_PROPERTY]: rankColor(box.rank),
              }}
              aria-pressed={props.selected === box.name}
              onClick={() => props.onSelect(box.name)}
            >
              <span class={styles.name}>{box.name}</span>
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

function sizeOf(element: HTMLElement): Size {
  return { width: element.clientWidth, height: element.clientHeight };
}
