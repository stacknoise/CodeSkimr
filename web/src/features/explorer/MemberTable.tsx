import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import { showUsages } from "../../shared/navigation/navigation";
import {
  CC_METRIC,
  CHIP,
  MISSING,
  SORT_COLUMN,
  SORT_DIRECTION,
  SORT_MARK,
  type SortColumn,
  TRUE,
  type ChipId,
} from "./consts";
import { currentMember, memberChip, memberSort, setMemberChip, setMemberSort } from "./explorer";
import { lineRange, metricValue } from "./format";
import { KindBadge } from "./KindBadge";
import { tipFor } from "./metrics";
import { pressColumn, tableRows } from "./memberSort";
import { ratingClass } from "./ratingClass";
import type { MemberRow, SymbolInfo, Thresholds } from "./types";
import styles from "./MemberTable.module.css";

const CHIPS: { id: ChipId; label: MessageKey }[] = [
  { id: CHIP.all, label: I18N_KEYS.EXPLORER_FILTER_ALL },
  { id: CHIP.properties, label: I18N_KEYS.EXPLORER_FILTER_PROPERTIES },
  { id: CHIP.methods, label: I18N_KEYS.EXPLORER_FILTER_METHODS },
  { id: CHIP.public, label: I18N_KEYS.EXPLORER_FILTER_PUBLIC_ONLY },
];

function lines(row: MemberRow): string {
  return lineRange(row.first_line, row.last_line);
}

function Row(props: { row: MemberRow; onPick: (row: MemberRow) => void }) {
  const row = () => props.row;
  return (
    <tr class={cx(styles.row, currentMember() === row().id && styles.selected)}>
      <td class={styles.name}>
        <button
          type="button"
          class={styles.pick}
          aria-current={currentMember() === row().id ? TRUE : undefined}
          onClick={() => props.onPick(row())}
        >
          <KindBadge kind={row().kind} /> <span>{row().name}</span>
        </button>
      </td>
      <td class={styles.signature}>{row().signature ?? MISSING}</td>
      <td class={styles.access}>{row().visibility ?? MISSING}</td>
      <td class={cx(styles.number, ratingClass(row().complexity_rating, styles))}>
        {metricValue(row().complexity ?? undefined, currentLanguage())}
      </td>
      <td class={styles.number}>
        <button
          type="button"
          class={styles.uses}
          title={t(I18N_KEYS.EXPLORER_USES_OPEN)}
          onClick={() => showUsages(row().id)}
        >
          {metricValue(row().usages, currentLanguage())}
        </button>
      </td>
      <td class={styles.number}>{lines(row())}</td>
    </tr>
  );
}

function Chips(props: { chip: ChipId; onChip: (chip: ChipId) => void }) {
  return (
    <div class={styles.chips}>
      <For each={CHIPS}>
        {(item) => (
          <button
            type="button"
            class={cx(styles.chip, props.chip === item.id && styles.on)}
            aria-pressed={props.chip === item.id}
            onClick={() => props.onChip(item.id)}
          >
            {t(item.label)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The head of a column that sorts the table when it is pressed. */
function SortHead(props: {
  column: SortColumn;
  label: MessageKey;
  number?: boolean;
  tip?: string;
}) {
  const current = () => memberSort();
  const sorted = () => current()?.column === props.column;
  const direction = () => current()?.direction;
  return (
    <th class={props.number ? styles.number : undefined} title={props.tip} aria-sort={direction()}>
      <button
        type="button"
        class={cx(styles.sort, sorted() && styles.sorted)}
        onClick={() => setMemberSort(pressColumn(current(), props.column))}
      >
        {t(props.label)}
        <Show when={sorted()}>
          <span class={styles.mark} aria-hidden={true}>
            {SORT_MARK[direction() ?? SORT_DIRECTION.ascending]}
          </span>
        </Show>
      </button>
    </th>
  );
}

function Head(props: { thresholds: Thresholds | null }) {
  return (
    <thead>
      <tr>
        <SortHead column={SORT_COLUMN.name} label={I18N_KEYS.EXPLORER_COLUMN_NAME} />
        <SortHead column={SORT_COLUMN.signature} label={I18N_KEYS.EXPLORER_COLUMN_SIGNATURE} />
        <SortHead column={SORT_COLUMN.access} label={I18N_KEYS.EXPLORER_COLUMN_ACCESS} />
        <SortHead
          column={SORT_COLUMN.complexity}
          label={I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY}
          tip={tipFor(CC_METRIC, I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY_TIP, props.thresholds)}
          number
        />
        <SortHead
          column={SORT_COLUMN.usages}
          label={I18N_KEYS.EXPLORER_COLUMN_USES}
          tip={t(I18N_KEYS.EXPLORER_COLUMN_USES_TIP)}
          number
        />
        <SortHead column={SORT_COLUMN.lines} label={I18N_KEYS.EXPLORER_COLUMN_LINES} number />
      </tr>
    </thead>
  );
}

/** Tab, filter chips and member table of the selected type (mock 2). Choosing a member is a real
 * button, so the table works with the keyboard. */
export function MemberTable(props: {
  info: SymbolInfo;
  thresholds: Thresholds | null;
  onPick: (row: MemberRow) => void;
}) {
  const rows = () => tableRows(props.info.members, memberChip(), memberSort());
  return (
    <div class={styles.table}>
      <div class={styles.bar} role="tablist">
        <button type="button" role="tab" aria-selected={true} class={styles.tab}>
          {t(I18N_KEYS.EXPLORER_TAB_MEMBERS, {
            count: metricValue(props.info.members.length, currentLanguage()),
          })}
        </button>
        <Chips chip={memberChip()} onChip={setMemberChip} />
      </div>
      <Show
        when={props.info.members.length > 0}
        fallback={<p class={styles.empty}>{t(I18N_KEYS.EXPLORER_NO_MEMBERS)}</p>}
      >
        <div class={styles.scroll}>
          <table>
            <Head thresholds={props.thresholds} />
            <tbody>
              <For each={rows()}>{(row) => <Row row={row} onPick={props.onPick} />}</For>
            </tbody>
          </table>
        </div>
      </Show>
    </div>
  );
}
