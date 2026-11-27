import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { KIND_LETTER, KIND_LETTER_OTHER } from "../explorer/consts";
import { CAUSE_CHOICES, FILTER_ALL, FILTER_FIELD_ID, KIND_JOIN } from "./consts";
import { changeFilter, isFiltering, patchFilter, resetFilter } from "./filter";
import styles from "./ChangeFilter.module.css";

/** The numbers under the filter: the rows that pass, the rows the screen holds and, if the answer
 * was cut, how many symbols the difference has in all. */
export interface FilterCount {
  shown: number;
  listed: number;
  total: number;
}

/** The text of the count: "123 of 2305", and the true total if the answer was cut. */
export function countText(count: FilterCount): string {
  return count.total > count.listed
    ? t(I18N_KEYS.CHANGE_VIEW_FILTER_COUNT_CUT, { ...count })
    : t(I18N_KEYS.CHANGE_VIEW_FILTER_COUNT, { shown: count.shown, listed: count.listed });
}

/** A button that is pressed or not, for a yes-or-no filter. */
function Toggle(props: { label: string; on: boolean; onToggle: (on: boolean) => void }) {
  return (
    <button
      type="button"
      class={cx(styles.toggle, props.on && styles.on)}
      aria-pressed={props.on}
      onClick={() => props.onToggle(!props.on)}
    >
      {props.label}
    </button>
  );
}

/** The two choices from a list: the kind of change and the kind of symbol. */
function Choices(props: { kinds: readonly string[] }) {
  return (
    <>
      <select
        class={styles.select}
        aria-label={t(I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_LABEL)}
        value={changeFilter().cause}
        onChange={(event) => patchFilter({ cause: event.currentTarget.value })}
      >
        <option value={FILTER_ALL}>{t(I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_ALL)}</option>
        <For each={CAUSE_CHOICES}>
          {(choice) => (
            <option value={choice.id} selected={changeFilter().cause === choice.id}>
              {t(choice.label)}
            </option>
          )}
        </For>
      </select>
      <select
        class={styles.select}
        aria-label={t(I18N_KEYS.CHANGE_VIEW_FILTER_KIND_LABEL)}
        value={changeFilter().kind}
        onChange={(event) => patchFilter({ kind: event.currentTarget.value })}
      >
        <option value={FILTER_ALL}>{t(I18N_KEYS.CHANGE_VIEW_FILTER_KIND_ALL)}</option>
        <For each={props.kinds}>
          {(kind) => (
            <option value={kind} selected={changeFilter().kind === kind}>
              {`${KIND_LETTER[kind] ?? KIND_LETTER_OTHER}${KIND_JOIN}${kind}`}
            </option>
          )}
        </For>
      </select>
    </>
  );
}

/** The yes-or-no filters and the button that clears the filter. */
function Toggles(props: { withOpen: boolean }) {
  return (
    <>
      <Toggle
        label={t(I18N_KEYS.CHANGE_VIEW_FILTER_PUBLIC)}
        on={changeFilter().onlyPublic}
        onToggle={(onlyPublic) => patchFilter({ onlyPublic })}
      />
      <Show when={props.withOpen}>
        <Toggle
          label={t(I18N_KEYS.CHANGE_VIEW_FILTER_ONLY_OPEN)}
          on={changeFilter().onlyOpen}
          onToggle={(onlyOpen) => patchFilter({ onlyOpen })}
        />
      </Show>
      <button
        type="button"
        class={styles.toggle}
        disabled={!isFiltering(changeFilter(), props.withOpen)}
        onClick={resetFilter}
      >
        {t(I18N_KEYS.CHANGE_VIEW_FILTER_RESET)}
      </button>
    </>
  );
}

/** The filter row over the list of the symbols, the same in both views: a search by name, the
 * kind of change, the kind of symbol, only the public ones and (`withOpen`, only the view that has
 * a checklist) only those with open places. The count says how many rows pass. */
export function ChangeFilterBar(props: {
  kinds: readonly string[];
  withOpen: boolean;
  count: FilterCount;
}) {
  return (
    <div class={styles.bar} role="search" aria-label={t(I18N_KEYS.CHANGE_VIEW_FILTER_BAR_LABEL)}>
      <input
        id={FILTER_FIELD_ID}
        class={styles.search}
        type="search"
        value={changeFilter().query}
        placeholder={t(I18N_KEYS.CHANGE_VIEW_FILTER_QUERY_PLACEHOLDER)}
        aria-label={t(I18N_KEYS.CHANGE_VIEW_FILTER_QUERY_LABEL)}
        onInput={(event) => patchFilter({ query: event.currentTarget.value })}
      />
      <Choices kinds={props.kinds} />
      <Toggles withOpen={props.withOpen} />
      <span class={styles.count} role="status">
        {countText(props.count)}
      </span>
    </div>
  );
}

/** What the list shows when no row passes the filter. */
export function FilterEmpty() {
  return (
    <p class={styles.empty} role="status">
      {t(I18N_KEYS.CHANGE_VIEW_FILTER_EMPTY)}
    </p>
  );
}
