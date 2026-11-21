import { For } from "solid-js";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, ResizableTh, RestHead } from "../../shared/table/ResizableHead";
import { LANGUAGE_COLUMNS, LANGUAGE_TABLE_KEY } from "./consts";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, languageName, parseTime } from "../projects/format";
import type { LanguageInfo } from "../projects/types";
import styles from "./Languages.module.css";

const TIER_TEXT = {
  full: I18N_KEYS.SCAN_TIER_FULL,
  basic: I18N_KEYS.SCAN_TIER_BASIC,
} as const;

const columns = createTableColumns(
  LANGUAGE_TABLE_KEY,
  LANGUAGE_COLUMNS.map((column) => column.id),
);

/** The head of the language table. */
function LanguageHead() {
  return (
    <thead>
      <tr>
        <For each={LANGUAGE_COLUMNS}>
          {(column) => (
            <ResizableTh
              columns={columns}
              id={column.id}
              name={t(column.label)}
              class={column.number ? styles.number : undefined}
            >
              {t(column.label)}
            </ResizableTh>
          )}
        </For>
        <RestHead columns={columns} />
      </tr>
    </thead>
  );
}

/** The languages of the project with tier, files, lines, symbols and the parse time of the last full
 * scan (mock 5, language table). Every column can be widened and narrowed. */
export function Languages(props: { languages: readonly LanguageInfo[] }) {
  return (
    <div class={styles.wrap}>
      <table class={styles.table} style={columns.style()}>
        <ColumnWidths columns={columns} />
        <LanguageHead />
        <tbody>
          <For each={props.languages}>
            {(item) => (
              <tr>
                <td class={styles.name}>{languageName(item.language)}</td>
                <td>
                  {item.tier === null ? t(I18N_KEYS.SCAN_UNAVAILABLE) : t(TIER_TEXT[item.tier])}
                </td>
                <td class={styles.number}>{count(item.files, currentLanguage())}</td>
                <td class={styles.number}>{count(item.loc, currentLanguage())}</td>
                <td class={styles.number}>{count(item.symbols, currentLanguage())}</td>
                <td class={styles.number}>
                  {item.parse_micros === null
                    ? t(I18N_KEYS.SCAN_UNAVAILABLE)
                    : parseTime(item.parse_micros, currentLanguage())}
                </td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
