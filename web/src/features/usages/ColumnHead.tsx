import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { ORIENTATION } from "../../shared/layout/consts";
import { columnWidths, gridColumns } from "./columns";
import { COLUMN_KEY_STEP_PX, HIT_COLUMNS } from "./consts";
import styles from "./HitList.module.css";

type Width = NonNullable<(typeof columnWidths)[string]>;

/** The handle on the right edge of a head cell: drag it or use the arrow keys to widen the column. */
function Handle(props: { width: Width; name: string }) {
  let origin = { pointer: 0, size: 0 };
  let dragging = false;
  return (
    <span
      class={styles.handle}
      role="separator"
      tabindex="0"
      aria-orientation={ORIENTATION.vertical}
      aria-label={t(I18N_KEYS.USAGES_COL_RESIZE, { name: props.name })}
      aria-valuenow={props.width.size()}
      aria-valuemin={props.width.bounds.min}
      aria-valuemax={props.width.bounds.max}
      onPointerDown={(event) => {
        event.preventDefault();
        origin = { pointer: event.clientX, size: props.width.size() };
        event.currentTarget.setPointerCapture?.(event.pointerId);
        dragging = true;
      }}
      onPointerMove={(event) => {
        if (dragging) {
          props.width.set(origin.size + event.clientX - origin.pointer);
        }
      }}
      onPointerUp={() => {
        dragging = false;
      }}
      onKeyDown={(event) => {
        const delta = { ArrowRight: 1, ArrowLeft: -1 }[event.key] ?? 0;
        if (delta !== 0) {
          event.preventDefault();
          props.width.set(props.width.size() + delta * COLUMN_KEY_STEP_PX);
        }
      }}
      onDblClick={() => props.width.reset()}
    />
  );
}

/** The labels of the columns of the hit list; the columns with a width can be widened. */
export function ColumnHead() {
  return (
    <div class={styles.columns} style={{ "grid-template-columns": gridColumns() }}>
      <For each={HIT_COLUMNS}>
        {(column) => (
          <span class={styles.cell}>
            {t(column.label)}
            <Show when={columnWidths[column.id]}>
              {(width) => <Handle width={width()} name={t(column.label)} />}
            </Show>
          </span>
        )}
      </For>
    </div>
  );
}
