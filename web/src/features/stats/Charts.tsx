import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { HISTOGRAM, HOTSPOTS, LIMIT_DASH, ROLE, TEXT_ANCHOR } from "./consts";
import { complexityTicks, histogramShapes, spots, spotScale, usageTicks } from "./geometry";
import type { BarShape, Spot, SpotScale } from "./geometry";
import type { Bar, Row } from "./types";
import styles from "./Charts.module.css";

function BarMark(props: { shape: BarShape; limit: number }) {
  return (
    <g>
      <title>{t(I18N_KEYS.STATS_HISTOGRAM_BAR, { ...props.shape.bar })}</title>
      <rect
        class={cx(styles.bar, props.shape.bar.complexity > props.limit && styles.over)}
        x={props.shape.x}
        y={props.shape.y}
        width={props.shape.width}
        height={props.shape.height}
        rx={HISTOGRAM.radius}
      />
      <text
        x={props.shape.middle}
        y={props.shape.y - HISTOGRAM.labelGap}
        text-anchor={TEXT_ANCHOR.middle}
      >
        {props.shape.bar.count}
      </text>
      <text
        x={props.shape.middle}
        y={HISTOGRAM.height - HISTOGRAM.bottom + HISTOGRAM.axisGap}
        text-anchor={TEXT_ANCHOR.middle}
      >
        {props.shape.bar.complexity}
      </text>
    </g>
  );
}

/** How many functions have each complexity; the bars above the limit are marked and named. */
export function Histogram(props: { bars: Bar[]; limit: number }) {
  return (
    <section class={styles.card}>
      <h2>{t(I18N_KEYS.STATS_HISTOGRAM_TITLE)}</h2>
      <p class={styles.detail}>{t(I18N_KEYS.STATS_HISTOGRAM_DETAIL, { limit: props.limit })}</p>
      <svg
        class={styles.chart}
        viewBox={HISTOGRAM.viewBox}
        role={ROLE.img}
        aria-label={t(I18N_KEYS.STATS_HISTOGRAM_TITLE)}
      >
        <For each={histogramShapes(props.bars)}>
          {(shape) => <BarMark shape={shape} limit={props.limit} />}
        </For>
      </svg>
    </section>
  );
}

type Scale = Pick<SpotScale, "maxX" | "maxY" | "x" | "y">;

function UsageAxis(props: Scale) {
  const right = HOTSPOTS.width - HOTSPOTS.right;
  return (
    <>
      <For each={usageTicks(props.maxY)}>
        {(tick) => (
          <>
            <line
              class={styles.grid}
              x1={HOTSPOTS.left}
              x2={right}
              y1={props.y(tick)}
              y2={props.y(tick)}
            />
            <text
              x={HOTSPOTS.left - HOTSPOTS.labelGap}
              y={props.y(tick)}
              text-anchor={TEXT_ANCHOR.end}
            >
              {tick}
            </text>
          </>
        )}
      </For>
      <text x={0} y={HOTSPOTS.axisTitle}>
        {t(I18N_KEYS.STATS_AXIS_USAGES)}
      </text>
    </>
  );
}

function ComplexityAxis(props: Scale) {
  const floor = HOTSPOTS.height - HOTSPOTS.bottom;
  const right = HOTSPOTS.width - HOTSPOTS.right;
  return (
    <>
      <For each={complexityTicks(props.maxX)}>
        {(tick) => (
          <text x={props.x(tick)} y={floor + HOTSPOTS.tickGap} text-anchor={TEXT_ANCHOR.middle}>
            {tick}
          </text>
        )}
      </For>
      <line class={styles.axis} x1={HOTSPOTS.left} x2={right} y1={floor} y2={floor} />
      <text x={right} y={floor + HOTSPOTS.titleGap} text-anchor={TEXT_ANCHOR.end}>
        {t(I18N_KEYS.STATS_AXIS_COMPLEXITY)}
      </text>
    </>
  );
}

/** The area right of the limit, with its line and its name. */
function LimitZone(props: { edge: number }) {
  return (
    <>
      <rect
        class={styles.zone}
        x={props.edge}
        y={HOTSPOTS.top}
        width={HOTSPOTS.width - HOTSPOTS.right - props.edge}
        height={HOTSPOTS.height - HOTSPOTS.top - HOTSPOTS.bottom}
      />
      <line
        class={styles.limit}
        x1={props.edge}
        x2={props.edge}
        y1={HOTSPOTS.top}
        y2={HOTSPOTS.height - HOTSPOTS.bottom}
        stroke-dasharray={LIMIT_DASH}
      />
      <text
        class={styles.limitText}
        x={props.edge + HOTSPOTS.labelGap}
        y={HOTSPOTS.top + HOTSPOTS.tickGap}
      >
        {t(I18N_KEYS.STATS_OVER_LIMIT)}
      </text>
    </>
  );
}

function SpotMark(props: { spot: Spot; selected: boolean; onSelect: (row: Row) => void }) {
  return (
    <g class={styles.spot} onClick={() => props.onSelect(props.spot.row)}>
      <title>
        {t(I18N_KEYS.STATS_HOTSPOTS_POINT, {
          name: props.spot.row.name,
          complexity: props.spot.row.complexity ?? 0,
          certain: props.spot.row.certain,
          total: props.spot.row.total,
        })}
      </title>
      <line
        class={styles.whisker}
        x1={props.spot.x}
        x2={props.spot.x}
        y1={props.spot.y}
        y2={props.spot.top}
        stroke-width={HOTSPOTS.whisker}
      />
      <circle
        class={cx(styles.dot, props.selected && styles.dotSelected)}
        cx={props.spot.x}
        cy={props.spot.y}
        r={props.selected ? HOTSPOTS.dotSelected : HOTSPOTS.dot}
      />
    </g>
  );
}

/** The functions by complexity and usages. The dot stands at the certain usages, the line reaches
 * up to all of them: the upper bound. Right of the limit the area is marked and named. */
export function Hotspots(props: {
  rows: Row[];
  limit: number;
  selected: number | null;
  onSelect: (row: Row) => void;
}) {
  const scale = () => spotScale(props.rows);
  const edge = () => Math.min(scale().x(props.limit), HOTSPOTS.width - HOTSPOTS.right);
  return (
    <section class={styles.card}>
      <h2>{t(I18N_KEYS.STATS_HOTSPOTS_TITLE)}</h2>
      <p class={styles.detail}>{t(I18N_KEYS.STATS_HOTSPOTS_DETAIL)}</p>
      <svg
        class={styles.chart}
        viewBox={HOTSPOTS.viewBox}
        role={ROLE.group}
        aria-label={t(I18N_KEYS.STATS_HOTSPOTS_TITLE)}
      >
        <Show when={scale().maxX > props.limit}>
          <LimitZone edge={edge()} />
        </Show>
        <UsageAxis {...scale()} />
        <ComplexityAxis {...scale()} />
        <For each={spots(props.rows, scale())}>
          {(spot) => (
            <SpotMark
              spot={spot}
              selected={spot.row.id === props.selected}
              onSelect={props.onSelect}
            />
          )}
        </For>
      </svg>
    </section>
  );
}
