import { createMemo, For, Match, onCleanup, onMount, Show, Switch } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { cx } from "../../shared/ui/cx";
import { Icon } from "../../shared/ui/Icon";
import { ROW_HEIGHT_PX } from "../explorer/consts";
import { KindBadge } from "../explorer/KindBadge";
import { useWindow } from "../explorer/useWindow";
import { UI_COMMANDS } from "./commands";
import {
  CHIPS,
  COMMAND_LETTER,
  DIALOG,
  FILTER,
  ICON_SEARCH,
  GROUP_TITLE,
  INPUT_ID,
  LIST_ID,
  OPTION_ID_PREFIX,
  PREFIX_OF,
  ROLE,
  TRUE,
} from "./consts";
import { buildEntries, listRows, type Entry, type ListRow } from "./entries";
import { nextChip, onSearchKey, type SearchKeyActions } from "./keys";
import { parseQuery, splitMatch } from "./parse";
import { Preview } from "./Preview";
import {
  choose,
  chosen,
  closeSearch,
  openHit,
  pickChip,
  searchChip,
  searchFailed,
  searchHits,
  searchMs,
  searchOpen,
  searchScanned,
  searchText,
  typeSearch,
} from "./search";
import styles from "./QuickSearch.module.css";

function activate(entry: Entry): void {
  if (entry.hit) {
    openHit(entry.hit);
  } else {
    closeSearch();
    entry.command?.run();
  }
}

/** Copies the name of an entry; `false` if there is nothing to copy it with. */
function copyName(entry: Entry): boolean {
  if (!entry.hit || typeof navigator.clipboard === "undefined") {
    return false;
  }
  void navigator.clipboard.writeText(entry.hit.qualified_name);
  return true;
}

const ACTIONS: SearchKeyActions = {
  close: closeSearch,
  choose,
  activate,
  chip: (step) => pickChip(nextChip(searchChip(), step)),
  copy: copyName,
};

function Row(props: { row: ListRow; text: string }) {
  return (
    <Show
      when={props.row.entry}
      fallback={<div class={styles.header}>{t(GROUP_TITLE[props.row.group])}</div>}
    >
      {(entry) => <Option entry={entry()} index={props.row.index} text={props.text} />}
    </Show>
  );
}

function Option(props: { entry: Entry; index: number; text: string }) {
  const parts = () => splitMatch(props.entry.label, props.text);
  return (
    <div
      id={`${OPTION_ID_PREFIX}${props.index}`}
      role={ROLE.option}
      class={cx(styles.option, chosen() === props.index && styles.chosen)}
      aria-selected={chosen() === props.index ? TRUE : undefined}
      onClick={() => activate(props.entry)}
      onMouseMove={() => choose(props.index)}
    >
      <Show when={props.entry.hit} fallback={<span class={styles.letter}>{COMMAND_LETTER}</span>}>
        {(hit) => <KindBadge kind={hit().kind} />}
      </Show>
      <span class={styles.label}>
        {parts().pre}
        <mark>{parts().hit}</mark>
        {parts().post}
      </span>
      <span class={styles.context}>{props.entry.hit?.qualified_name ?? ""}</span>
    </div>
  );
}

function Chips() {
  return (
    <div class={styles.chips}>
      <For each={CHIPS}>
        {(chip) => (
          <button
            type="button"
            tabIndex={-1}
            class={cx(styles.chip, searchChip() === chip.filter && styles.on)}
            aria-pressed={searchChip() === chip.filter}
            onClick={() => pickChip(chip.filter)}
          >
            <Show when={chip.prefix}>
              {(prefix) => <span class={styles.prefix}>{prefix()}</span>}
            </Show>
            {t(chip.label)}
          </button>
        )}
      </For>
    </div>
  );
}

/** What the list says while it has no entries. */
function Note(props: { empty: boolean }) {
  return (
    <p class={styles.note}>
      <Switch>
        <Match when={activeProject() === null}>{t(I18N_KEYS.PALETTE_NO_PROJECT)}</Match>
        <Match when={searchFailed()}>{t(I18N_KEYS.PALETTE_FAILED)}</Match>
        <Match when={!searchScanned()}>{t(I18N_KEYS.PALETTE_NOT_SCANNED)}</Match>
        <Match when={props.empty}>{t(I18N_KEYS.PALETTE_START, PREFIX_OF)}</Match>
        <Match when={true}>{t(I18N_KEYS.PALETTE_EMPTY)}</Match>
      </Switch>
    </p>
  );
}

function Results(props: { rows: ListRow[]; chosenRow: number; text: string; empty: boolean }) {
  const view = useWindow(
    () => props.rows,
    () => props.chosenRow,
  );
  return (
    <div class={styles.results}>
      <Show when={props.rows.length === 0}>
        <Note empty={props.empty} />
      </Show>
      <div
        id={LIST_ID}
        ref={view.attach}
        class={styles.list}
        role={ROLE.listbox}
        aria-label={t(I18N_KEYS.PALETTE_RESULTS_LABEL)}
        onScroll={view.onScroll}
      >
        <div class={styles.window} style={{ height: `${view.total()}px` }}>
          <For each={view.visible()}>
            {(place) => (
              <div class={styles.place} style={{ top: `${place.index * ROW_HEIGHT_PX}px` }}>
                <Row row={place.item} text={props.text} />
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}

/** The input with the hit count; the keys of the overlay are handled here. */
function SearchInput(props: { entries: Entry[]; current: Entry | null }) {
  let input: HTMLInputElement | undefined;
  onMount(() => input?.focus());
  const selecting = () => input?.selectionStart !== input?.selectionEnd;
  return (
    <div class={styles.top}>
      <span class={styles.lens}>
        <Icon name={ICON_SEARCH} />
      </span>
      <input
        id={INPUT_ID}
        ref={input}
        class={styles.input}
        role={ROLE.combobox}
        aria-expanded={TRUE}
        aria-controls={LIST_ID}
        aria-activedescendant={
          props.current === null ? undefined : `${OPTION_ID_PREFIX}${chosen()}`
        }
        placeholder={t(I18N_KEYS.APP_SEARCH_PLACEHOLDER)}
        aria-label={t(I18N_KEYS.PALETTE_PLACEHOLDER)}
        value={searchText()}
        spellcheck={false}
        autocomplete="off"
        onInput={(event) => typeSearch(event.currentTarget.value)}
        onKeyDown={(event) => onSearchKey(event, props.entries, chosen(), ACTIONS, selecting())}
      />
      <Show when={searchMs() !== null}>
        <span class={styles.count}>
          {t(I18N_KEYS.PALETTE_RESULT_COUNT, { count: props.entries.length, ms: searchMs() ?? 0 })}
        </span>
      </Show>
      <kbd class={styles.key}>{t(I18N_KEYS.PALETTE_KEY_CLOSE)}</kbd>
    </div>
  );
}

function Footer() {
  return (
    <div class={styles.footer}>
      <span>
        <kbd class={styles.key}>{t(I18N_KEYS.PALETTE_KEY_UP)}</kbd>{" "}
        <kbd class={styles.key}>{t(I18N_KEYS.PALETTE_KEY_DOWN)}</kbd>{" "}
        {t(I18N_KEYS.PALETTE_HINT_SELECT)}
      </span>
      <span>
        <kbd class={styles.key}>{t(I18N_KEYS.PALETTE_KEY_TAB)}</kbd>{" "}
        {t(I18N_KEYS.PALETTE_HINT_FILTER)}
      </span>
      <span>{t(I18N_KEYS.PALETTE_HINT_FUZZY)}</span>
    </div>
  );
}

function Overlay() {
  const previous = document.activeElement;
  onCleanup(() => previous instanceof HTMLElement && previous.focus());
  const parsed = createMemo(() => parseQuery(searchText(), searchChip()));
  const entries = createMemo(() =>
    buildEntries(searchHits(), UI_COMMANDS, parsed().filter, parsed().text),
  );
  const rows = createMemo(() => listRows(entries()));
  const current = () => entries()[chosen()] ?? null;
  const chosenRow = () => rows().findIndex((row) => row.entry && row.index === chosen());
  return (
    <div
      class={styles.backdrop}
      onMouseDown={(event) => event.target === event.currentTarget && closeSearch()}
    >
      <div
        class={styles.dialog}
        role={DIALOG}
        aria-modal={TRUE}
        aria-label={t(I18N_KEYS.PALETTE_TITLE)}
      >
        <SearchInput entries={entries()} current={current()} />
        <Chips />
        <div class={styles.body}>
          <Results
            rows={rows()}
            chosenRow={chosenRow()}
            text={parsed().text}
            empty={parsed().text === "" && parsed().filter !== FILTER.commands}
          />
          <Preview hit={current()?.hit ?? null} />
        </div>
        <Footer />
      </div>
    </div>
  );
}

/** The quick search (Ctrl+K, blueprint B.4, mock 4): input, filter chips, grouped hits with the
 * match marked, preview and keys. */
export function QuickSearch() {
  return (
    <Show when={searchOpen()}>
      <Overlay />
    </Show>
  );
}
