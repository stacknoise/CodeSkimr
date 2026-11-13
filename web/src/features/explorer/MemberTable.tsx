import { createSignal, For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import {
  CHIP,
  LINE_RANGE_SEPARATOR,
  METHOD_KINDS,
  MISSING,
  PROPERTY_KINDS,
  PUBLIC,
  RATING,
  type ChipId,
} from "./consts";
import { currentMember } from "./explorer";
import { metricValue } from "./format";
import { KindBadge } from "./KindBadge";
import type { MemberRow, SymbolInfo } from "./types";
import styles from "./MemberTable.module.css";

const CHIPS: { id: ChipId; label: MessageKey }[] = [
  { id: CHIP.all, label: I18N_KEYS.EXPLORER_FILTER_ALL },
  { id: CHIP.properties, label: I18N_KEYS.EXPLORER_FILTER_PROPERTIES },
  { id: CHIP.methods, label: I18N_KEYS.EXPLORER_FILTER_METHODS },
  { id: CHIP.public, label: I18N_KEYS.EXPLORER_FILTER_PUBLIC_ONLY },
];

const RATING_CLASS: Record<string, string | undefined> = {
  [RATING.good]: styles.good,
  [RATING.medium]: styles.medium,
  [RATING.poor]: styles.poor,
};

function keeps(chip: ChipId, row: MemberRow): boolean {
  switch (chip) {
    case CHIP.properties:
      return PROPERTY_KINDS.includes(row.kind);
    case CHIP.methods:
      return METHOD_KINDS.includes(row.kind);
    case CHIP.public:
      return row.visibility === PUBLIC;
    default:
      return true;
  }
}

function lines(row: MemberRow): string {
  return row.first_line === row.last_line
    ? String(row.first_line)
    : `${row.first_line}${LINE_RANGE_SEPARATOR}${row.last_line}`;
}

function Row(props: { row: MemberRow; onPick: (id: number) => void }) {
  const row = () => props.row;
  return (
    <tr
      class={cx(styles.row, currentMember() === row().id && styles.selected)}
      onClick={() => props.onPick(row().id)}
    >
      <td class={styles.name}>
        <KindBadge kind={row().kind} /> <span>{row().name}</span>
      </td>
      <td class={styles.signature}>{row().signature ?? MISSING}</td>
      <td class={styles.access}>{row().visibility ?? MISSING}</td>
      <td class={cx(styles.number, RATING_CLASS[row().complexity_rating ?? ""])}>
        {row().complexity === null
          ? ""
          : metricValue(row().complexity ?? undefined, currentLanguage())}
      </td>
      <td class={styles.number}>
        <span class={styles.uses}>{metricValue(row().usages, currentLanguage())}</span>
      </td>
      <td class={styles.number}>{lines(row())}</td>
    </tr>
  );
}

/** Tab, filter chips and member table of the selected type (mock 2). */
export function MemberTable(props: { info: SymbolInfo; onPick: (id: number) => void }) {
  const [chip, setChip] = createSignal<ChipId>(CHIP.all);
  const rows = () => props.info.members.filter((row) => keeps(chip(), row));
  return (
    <div class={styles.table}>
      <div class={styles.bar}>
        <span class={styles.tab}>
          {t(I18N_KEYS.EXPLORER_TAB_MEMBERS, {
            count: metricValue(props.info.members.length, currentLanguage()),
          })}
        </span>
        <div class={styles.chips}>
          <For each={CHIPS}>
            {(item) => (
              <button
                type="button"
                class={cx(styles.chip, chip() === item.id && styles.on)}
                aria-pressed={chip() === item.id}
                onClick={() => setChip(item.id)}
              >
                {t(item.label)}
              </button>
            )}
          </For>
        </div>
      </div>
      <div class={styles.scroll}>
        <table>
          <thead>
            <tr>
              <th>{t(I18N_KEYS.EXPLORER_COLUMN_NAME)}</th>
              <th>{t(I18N_KEYS.EXPLORER_COLUMN_SIGNATURE)}</th>
              <th>{t(I18N_KEYS.EXPLORER_COLUMN_ACCESS)}</th>
              <th class={styles.number} title={t(I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY_TIP)}>
                {t(I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY)}
              </th>
              <th class={styles.number} title={t(I18N_KEYS.EXPLORER_COLUMN_USES_TIP)}>
                {t(I18N_KEYS.EXPLORER_COLUMN_USES)}
              </th>
              <th class={styles.number}>{t(I18N_KEYS.EXPLORER_COLUMN_LINES)}</th>
            </tr>
          </thead>
          <tbody>
            <For each={rows()}>{(row) => <Row row={row} onPick={props.onPick} />}</For>
          </tbody>
        </table>
      </div>
    </div>
  );
}
