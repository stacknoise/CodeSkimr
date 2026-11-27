import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { languageName } from "../projects/format";
import { KINDS, LANGUAGE_ALL, ORDERS, ROLE, TAB, UNUSED_FILTERS } from "./consts";
import {
  setStatsAssessment,
  setStatsKinds,
  setStatsLanguage,
  setStatsOrder,
  statsAssessment,
  statsKinds,
  statsLanguage,
  statsOrder,
} from "./stats";
import type { RankingName } from "./types";
import styles from "./Filters.module.css";

/** A row of buttons of which one is pressed. */
function Choice<Id extends string | null>(props: {
  label: MessageKey;
  options: readonly { id: Id; label: MessageKey }[];
  current: Id;
  onChoose: (id: Id) => void;
}) {
  return (
    <div class={styles.field}>
      <span class={styles.label}>{t(props.label)}</span>
      <div class={styles.group} role={ROLE.group} aria-label={t(props.label)}>
        <For each={props.options}>
          {(option) => (
            <button
              type="button"
              class={cx(props.current === option.id && styles.on)}
              aria-pressed={props.current === option.id}
              onClick={() => props.onChoose(option.id)}
            >
              {t(option.label)}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

/** The assessment or reason the unused symbols are cut to, as a list (it holds more choices than
 * a row of buttons shows). */
function UnusedFilterField() {
  return (
    <label class={styles.field}>
      <span class={styles.label}>{t(I18N_KEYS.STATS_FILTER_ASSESSMENT)}</span>
      <select
        value={statsAssessment() ?? LANGUAGE_ALL}
        onChange={(event) =>
          setStatsAssessment(
            UNUSED_FILTERS.find((entry) => entry.id === event.currentTarget.value)?.id ?? null,
          )
        }
      >
        <option value={LANGUAGE_ALL}>{t(I18N_KEYS.STATS_ASSESSMENT_ALL)}</option>
        <For each={UNUSED_FILTERS}>
          {(entry) => (
            <option value={entry.id} selected={entry.id === statsAssessment()}>
              {t(entry.label)}
            </option>
          )}
        </For>
      </select>
    </label>
  );
}

function LanguageField(props: { languages: string[] }) {
  return (
    <label class={styles.field}>
      <span class={styles.label}>{t(I18N_KEYS.STATS_FILTER_LANGUAGE)}</span>
      <select
        value={statsLanguage() ?? LANGUAGE_ALL}
        onChange={(event) => setStatsLanguage(event.currentTarget.value || null)}
      >
        <option value={LANGUAGE_ALL}>{t(I18N_KEYS.STATS_LANGUAGE_ALL)}</option>
        <For each={props.languages}>
          {(language) => (
            <option value={language} selected={language === statsLanguage()}>
              {languageName(language)}
            </option>
          )}
        </For>
      </select>
    </label>
  );
}

/** The filters of a ranking: kind and language always, the sort order for the most used symbols
 * and the assessment for the ones without usages. */
export function Filters(props: { ranking: RankingName; languages: string[] }) {
  return (
    <div class={styles.bar}>
      <Show when={props.ranking !== TAB.complexity}>
        <Choice
          label={I18N_KEYS.STATS_FILTER_KINDS}
          options={KINDS}
          current={statsKinds()}
          onChoose={setStatsKinds}
        />
      </Show>
      <LanguageField languages={props.languages} />
      <Show when={props.ranking === TAB.mostUsed}>
        <Choice
          label={I18N_KEYS.STATS_FILTER_ORDER}
          options={ORDERS}
          current={statsOrder()}
          onChoose={setStatsOrder}
        />
        <span class={styles.legend}>
          <i class={styles.certain} /> {t(I18N_KEYS.STATS_LEGEND_CERTAIN)}
          <i class={styles.probable} /> {t(I18N_KEYS.STATS_LEGEND_PROBABLE)}
        </span>
      </Show>
      <Show when={props.ranking === TAB.unused}>
        <UnusedFilterField />
      </Show>
    </div>
  );
}
