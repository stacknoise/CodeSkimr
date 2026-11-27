import { Show, type JSX } from "solid-js";
import { cx } from "../ui/cx";
import type { TableColumns } from "./columns";
import { SORT_DIRECTION, SORT_MARK, type SortDirection } from "./consts";
import { ResizableTh } from "./ResizableHead";
import styles from "./SortableTh.module.css";

/** A head cell that sorts the table when it is pressed and has the handle for its width. `direction`
 * is the sort of this column, `undefined` while another column (or none) sorts the table. */
export function SortableTh(props: {
  columns: TableColumns;
  id: string;
  label: string;
  direction: SortDirection | undefined;
  onPress: () => void;
  class?: string | undefined;
  title?: string | undefined;
}): JSX.Element {
  return (
    <ResizableTh
      columns={props.columns}
      id={props.id}
      name={props.label}
      class={props.class}
      title={props.title}
      sort={props.direction}
    >
      <button
        type="button"
        class={cx(styles.sort, props.direction !== undefined && styles.sorted)}
        onClick={props.onPress}
      >
        {props.label}
        <Show when={props.direction}>
          {(direction) => (
            <span class={styles.mark} aria-hidden={true}>
              {SORT_MARK[direction() ?? SORT_DIRECTION.ascending]}
            </span>
          )}
        </Show>
      </button>
    </ResizableTh>
  );
}
