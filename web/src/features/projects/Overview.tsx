import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { OUTCOME } from "../../shared/project/report";
import { TONE } from "../../shared/ui/consts";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { ago, clock, count, languageName, languageShares } from "./format";
import { DatabaseCard } from "./DatabaseCard";
import { LanguageBar } from "./LanguageBar";
import { PROJECT_TAB, MS_PER_SECOND } from "./consts";
import { showTab } from "./page";
import { toolsOf } from "./projects";
import { lastScanLine } from "./reportText";
import { level2Failure } from "./state";
import { problemsOf } from "./tools";
import type { LanguageInfo, ProjectRow } from "./types";
import styles from "./Overview.module.css";

function Tile(props: { label: string; value: string }) {
  return (
    <div class={styles.tile}>
      <span class={styles.tileLabel}>{props.label}</span>
      <span class={styles.tileValue}>{props.value}</span>
    </div>
  );
}

/** Types, lines, files and the time of the last scan. */
function Tiles(props: { row: ProjectRow }) {
  const language = currentLanguage;
  return (
    <div class={styles.tiles}>
      <Tile label={t(I18N_KEYS.PROJECTS_TILE_TYPES)} value={count(props.row.types, language())} />
      <Tile label={t(I18N_KEYS.PROJECTS_TILE_LINES)} value={count(props.row.loc, language())} />
      <Tile
        label={t(I18N_KEYS.PROJECTS_TILE_FILES)}
        value={count(props.row.scan?.files ?? 0, language())}
      />
      <Tile
        label={t(I18N_KEYS.PROJECTS_TILE_LAST_SCAN)}
        value={
          props.row.scan === null
            ? t(I18N_KEYS.PROJECTS_NEVER)
            : ago(props.row.scan.created, Date.now(), language())
        }
      />
    </div>
  );
}

/** What stage 2 did for a language, in words. */
function accuracy(row: ProjectRow, language: LanguageInfo): { text: string; tone: PillTone } {
  const line = row.report?.level2?.languages.find((item) => item.language === language.language);
  if (line?.outcome === OUTCOME.failed) {
    return { text: t(I18N_KEYS.PROJECTS_ACCURACY_FAILED), tone: TONE.bad };
  }
  return line?.outcome === OUTCOME.checked
    ? { text: t(I18N_KEYS.PROJECTS_ACCURACY_EXACT), tone: TONE.good }
    : { text: t(I18N_KEYS.PROJECTS_ACCURACY_APPROXIMATE), tone: TONE.neutral };
}

/** The head of the language table. */
function LanguageHead() {
  return (
    <thead>
      <tr>
        <th>{t(I18N_KEYS.SCAN_COLUMN_LANGUAGE)}</th>
        <th>{t(I18N_KEYS.PROJECTS_COLUMN_SHARE)}</th>
        <th>{t(I18N_KEYS.PROJECTS_COLUMN_ACCURACY)}</th>
        <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_FILES)}</th>
        <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_LINES)}</th>
        <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_SYMBOLS)}</th>
      </tr>
    </thead>
  );
}

/** One language: share, accuracy in words, files, lines, symbols. */
function LanguageRow(props: { row: ProjectRow; language: LanguageInfo }) {
  const share = () =>
    languageShares(props.row.languages).find((item) => item.language === props.language.language);
  const look = () => accuracy(props.row, props.language);
  return (
    <tr>
      <th scope="row">{languageName(props.language.language)}</th>
      <td class={styles.number}>
        {t(I18N_KEYS.PROJECTS_LANGUAGE_PERCENT, { percent: share()?.percent ?? 0 })}
      </td>
      <td>
        <Pill tone={look().tone}>{look().text}</Pill>
      </td>
      <td class={styles.number}>{count(props.language.files, currentLanguage())}</td>
      <td class={styles.number}>{count(props.language.loc, currentLanguage())}</td>
      <td class={styles.number}>{count(props.language.symbols, currentLanguage())}</td>
    </tr>
  );
}

/** The bar and one table of the languages. */
function Languages(props: { row: ProjectRow }) {
  return (
    <Show when={props.row.languages.length > 0}>
      <section aria-label={t(I18N_KEYS.PROJECTS_LANGUAGES_TITLE)}>
        <h3 class={styles.cardTitle}>{t(I18N_KEYS.PROJECTS_LANGUAGES_TITLE)}</h3>
        <LanguageBar languages={props.row.languages} />
        <table class={styles.table}>
          <LanguageHead />
          <tbody>
            <For each={props.row.languages}>
              {(language) => <LanguageRow row={props.row} language={language} />}
            </For>
          </tbody>
        </table>
      </section>
    </Show>
  );
}

/** The note for a failed stage 2, with the way to its details. */
function FailureNote(props: { row: ProjectRow }) {
  return (
    <Show when={level2Failure(props.row)}>
      {(failure) => (
        <div class={styles.note} role="status">
          <span>
            {t(I18N_KEYS.PROJECTS_NOTE_LEVEL2_FAILED, {
              languages: failure().languages.join(", "),
              code: failure().code,
            })}
          </span>{" "}
          <button type="button" class={styles.link} onClick={() => showTab(PROJECT_TAB.level2)}>
            {t(I18N_KEYS.PROJECTS_TAB_LEVEL2)}
          </button>
        </div>
      )}
    </Show>
  );
}

/** The cards "Last scan" and "Tools". */
function Cards(props: { row: ProjectRow }) {
  const problems = () => problemsOf(toolsOf(props.row)).length;
  return (
    <div class={styles.two}>
      <section class={styles.card} aria-label={t(I18N_KEYS.PROJECTS_CARD_LAST_SCAN)}>
        <h3 class={styles.cardTitle}>{t(I18N_KEYS.PROJECTS_CARD_LAST_SCAN)}</h3>
        <Show
          when={props.row.report}
          fallback={<p class={styles.muted}>{t(I18N_KEYS.PROJECTS_NEVER)}</p>}
        >
          {(report) => (
            <p class={styles.muted}>
              {lastScanLine(report())}
              {" · "}
              {clock(report().last_scan.at * MS_PER_SECOND, currentLanguage())}
            </p>
          )}
        </Show>
        <Show when={(props.row.left_out?.length ?? 0) > 0}>
          <p class={styles.muted}>
            {t(I18N_KEYS.PROJECTS_CARD_LEFT_OUT, { count: props.row.left_out?.length ?? 0 })}
          </p>
        </Show>
      </section>
      <section class={styles.card} aria-label={t(I18N_KEYS.PROJECTS_CARD_TOOLS)}>
        <div class={styles.cardHead}>
          <h3 class={styles.cardTitle}>{t(I18N_KEYS.PROJECTS_CARD_TOOLS)}</h3>
          <Pill tone={problems() > 0 ? TONE.warn : TONE.good}>
            {problems() > 0
              ? t(I18N_KEYS.PROJECTS_TOOLS_MISSING_COUNT, { count: problems() })
              : t(I18N_KEYS.PROJECTS_CARD_TOOLS_OK)}
          </Pill>
        </div>
      </section>
    </div>
  );
}

/** The tab "Overview" of the project page (mock 23). */
export function Overview(props: { row: ProjectRow }) {
  return (
    <div class={styles.overview}>
      <Tiles row={props.row} />
      <FailureNote row={props.row} />
      <Languages row={props.row} />
      <Cards row={props.row} />
      <DatabaseCard row={props.row} />
    </div>
  );
}
