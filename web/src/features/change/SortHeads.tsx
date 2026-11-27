import { For } from "solid-js";
import { t, type MessageKey } from "../../shared/i18n/i18n";
import type { TableColumns } from "../../shared/table/columns";
import { RestHead } from "../../shared/table/ResizableHead";
import { pressSort, type SortState } from "../../shared/table/sort";
import { SortableTh } from "../../shared/table/SortableTh";

/** The head row of a list of the screen: every head sorts the list when it is pressed and has the
 * handle for its width (`shared/table`). */
export function SortHeads(props: {
  columns: TableColumns;
  heads: readonly { id: string; label: MessageKey; number?: boolean }[];
  sort: SortState | null;
  onSort: (next: SortState | null) => void;
  /** The class of a head cell whose column holds numbers. */
  numberClass?: string | undefined;
}) {
  return (
    <tr>
      <For each={props.heads}>
        {(head) => (
          <SortableTh
            columns={props.columns}
            id={head.id}
            label={t(head.label)}
            class={head.number ? props.numberClass : undefined}
            direction={props.sort?.column === head.id ? props.sort.direction : undefined}
            onPress={() => props.onSort(pressSort(props.sort, head.id, head.number === true))}
          />
        )}
      </For>
      <RestHead columns={props.columns} />
    </tr>
  );
}
