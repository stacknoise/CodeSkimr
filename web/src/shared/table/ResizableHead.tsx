import { For, Show, type JSX } from "solid-js";
import { ColumnGrip } from "./ColumnGrip";
import type { TableColumns } from "./columns";
import { COLUMN_ATTRIBUTE, PIXEL_UNIT } from "./consts";
import styles from "./ResizableHead.module.css";

/** A head cell with the handle on its right edge. */
export function ResizableTh(props: {
  columns: TableColumns;
  id: string;
  name: string;
  class?: string | undefined;
  title?: string | undefined;
  sort?: JSX.HTMLAttributes<HTMLTableCellElement>["aria-sort"] | undefined;
  children: JSX.Element;
}) {
  return (
    <th
      {...{ [COLUMN_ATTRIBUTE]: props.id }}
      class={`${styles.cell} ${props.class ?? ""}`}
      title={props.title}
      aria-sort={props.sort}
    >
      {props.children}
      <ColumnGrip
        width={props.columns.column(props.id)}
        name={props.name}
        before={props.columns.freeze}
      />
    </th>
  );
}

/** The `colgroup` of a table with fixed widths: a width for every column, none for the empty column
 * at the end that takes what is left, so that the last real column can be widened too. */
export function ColumnWidths(props: { columns: TableColumns }) {
  return (
    <Show when={props.columns.fixed()}>
      <colgroup>
        <For each={props.columns.ids}>
          {(id) => <col style={{ width: `${props.columns.width(id)}${PIXEL_UNIT}` }} />}
        </For>
        <col />
      </colgroup>
    </Show>
  );
}

/** The empty head cell at the end of a table with fixed widths. */
export function RestHead(props: { columns: TableColumns }) {
  return (
    <Show when={props.columns.fixed()}>
      <th class={styles.rest} aria-hidden={true} />
    </Show>
  );
}
