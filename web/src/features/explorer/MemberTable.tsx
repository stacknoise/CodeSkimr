import { createSignal, For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import { showUsages } from "../../shared/navigation/navigation";
import {
  CC_METRIC,
  CHIP,
  METHOD_KINDS,
  MISSING,
  PROPERTY_KINDS,
  PUBLIC,
  TRUE,
  type ChipId,
} from "./consts";
import { currentMember } from "./explorer";
import { lineRange, metricValue } from "./format";
import { KindBadge } from "./KindBadge";
import { tipFor } from "./metrics";
import { ratingClass } from "./ratingClass";
import type { MemberRow, SymbolInfo, Thresholds } from "./types";
import styles from "./MemberTable.module.css";

const CHIPS: { id: ChipId; label: MessageKey }[] = [
  { id: CHIP.all, label: I18N_KEYS.EXPLORER_FILTER_ALL },
  { id: CHIP.properties, label: I18N_KEYS.EXPLORER_FILTER_PROPERTIES },
  { id: CHIP.methods, label: I18N_KEYS.EXPLORER_FILTER_METHODS },
  { id: CHIP.public, label: I18N_KEYS.EXPLORER_FILTER_PUBLIC_ONLY },
];

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

function Head(props: { thresholds: Thresholds | null }) {
  return (
    <thead>
      <tr>
        <th>{t(I18N_KEYS.EXPLORER_COLUMN_NAME)}</th>
        <th>{t(I18N_KEYS.EXPLORER_COLUMN_SIGNATURE)}</th>
        <th>{t(I18N_KEYS.EXPLORER_COLUMN_ACCESS)}</th>
        <th
          class={styles.number}
          title={tipFor(CC_METRIC, I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY_TIP, props.thresholds)}
        >
          {t(I18N_KEYS.EXPLORER_COLUMN_COMPLEXITY)}
        </th>
        <th class={styles.number} title={t(I18N_KEYS.EXPLORER_COLUMN_USES_TIP)}>
          {t(I18N_KEYS.EXPLORER_COLUMN_USES)}
        </th>
        <th class={styles.number}>{t(I18N_KEYS.EXPLORER_COLUMN_LINES)}</th>
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
  const [chip, setChip] = createSignal<ChipId>(CHIP.all);
  const rows = () => props.info.members.filter((row) => keeps(chip(), row));
  return (
    <div class={styles.table}>
      <div class={styles.bar} role="tablist">
        <button type="button" role="tab" aria-selected={true} class={styles.tab}>
          {t(I18N_KEYS.EXPLORER_TAB_MEMBERS, {
            count: metricValue(props.info.members.length, currentLanguage()),
          })}
        </button>
        <Chips chip={chip()} onChip={setChip} />
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
