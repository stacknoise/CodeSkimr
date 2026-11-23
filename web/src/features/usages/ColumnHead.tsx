import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { ColumnGrip } from "../../shared/table/ColumnGrip";
import { columnWidths, gridColumns, resetColumnWidths } from "./columns";
import { HIT_COLUMNS } from "./consts";
import styles from "./HitList.module.css";

/** The labels of the columns of the hit list; every column can be widened and narrowed. */
export function ColumnHead() {
  return (
    <div class={styles.columns} style={{ "grid-template-columns": gridColumns() }}>
      <For each={HIT_COLUMNS}>
        {(column) => (
          <span class={styles.cell}>
            {t(column.label)}
            <Show when={columnWidths[column.id]}>
              {(width) => (
                <ColumnGrip
                  width={{ ...width(), reset: resetColumnWidths }}
                  name={t(column.label)}
                />
              )}
            </Show>
          </span>
        )}
      </For>
    </div>
  );
}
