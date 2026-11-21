import { HATCH, USAGE_BAR } from "./consts";
import { usageWidths } from "./geometry";
import type { Row } from "./types";
import styles from "./UsageBar.module.css";

/** The hatch of the probable part; drawn once per screen. */
export function HatchPattern() {
  return (
    <svg class={styles.defs} aria-hidden={true}>
      <defs>
        <pattern
          id={HATCH.id}
          width={HATCH.size}
          height={HATCH.size}
          patternUnits={HATCH.units}
          patternTransform={HATCH.turn}
        >
          <rect class={styles.hatch} width={HATCH.line} height={HATCH.size} />
        </pattern>
      </defs>
    </svg>
  );
}

/** The usages of a row as a bar: the certain part solid, the probable part hatched, so the
 * difference does not hang on colour alone. `most` usages fill the bar. */
export function UsageBar(props: { row: Row; most: number }) {
  const widths = () => usageWidths(props.row, props.most);
  return (
    <svg
      class={styles.bar}
      width={USAGE_BAR.width}
      height={USAGE_BAR.height}
      viewBox={USAGE_BAR.viewBox}
      aria-hidden={true}
    >
      <rect
        class={styles.track}
        width={USAGE_BAR.width}
        height={USAGE_BAR.height}
        rx={USAGE_BAR.radius}
      />
      <rect
        class={styles.certain}
        width={widths().certain}
        height={USAGE_BAR.height}
        rx={USAGE_BAR.radius}
      />
      <rect
        x={widths().certain}
        width={widths().probable}
        height={USAGE_BAR.height}
        fill={HATCH.fill}
      />
    </svg>
  );
}
