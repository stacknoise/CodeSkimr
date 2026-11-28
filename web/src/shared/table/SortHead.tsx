import type { SortState } from "./sort";
import { pressSort } from "./sort";
import { SortableTh } from "./SortableTh";
import type { TableColumns } from "./columns";
import styles from "./SortHead.module.css";

/** The head of a column of a sortable table: it shows the sort of its column, sorts when pressed
 * (a number column starts with the largest) and has the handle for its width. */
export function SortHead(props: {
  columns: TableColumns;
  id: string;
  label: string;
  sort: SortState | null;
  numeric: boolean;
  /** The class of the cell of a number column, where it is not the usual one. */
  numberClass?: string | undefined;
  onSort: (next: SortState | null) => void;
  title?: string | undefined;
}) {
  const own = () => props.sort?.column === props.id;
  return (
    <SortableTh
      columns={props.columns}
      id={props.id}
      label={props.label}
      class={props.numeric ? (props.numberClass ?? styles.num) : undefined}
      title={props.title}
      direction={own() ? props.sort?.direction : undefined}
      onPress={() => props.onSort(pressSort(props.sort, props.id, props.numeric))}
    />
  );
}
