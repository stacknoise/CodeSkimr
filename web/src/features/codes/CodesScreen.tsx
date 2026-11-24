import { createSignal, For, Show } from "solid-js";
import { SETTINGS_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import type { ErrorCodeInfo } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { CopyDetails } from "../../shared/ui/CopyDetails";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import {
  CODE_SEPARATOR,
  GROUP_ALL,
  GROUP_FIELD_ID,
  LINE_BREAK,
  SEARCH_FIELD_ID,
  SEVERITIES,
  SEVERITY_ALL,
  SEVERITY_KEY,
  SEVERITY_TONE,
} from "./consts";
import { allCodes, filterCodes, groupsOf } from "./filter";
import styles from "./CodesScreen.module.css";

const retryText = (code: ErrorCodeInfo) =>
  t(code.retryable ? I18N_KEYS.CODES_RETRY_YES : I18N_KEYS.CODES_RETRY_NO);

/** The explanation of one code, as `codeskimr codes <id>` prints it. */
function explanation(code: ErrorCodeInfo): string {
  const facts = t(I18N_KEYS.CODES_FACTS, {
    group: code.group,
    severity: t(SEVERITY_KEY[code.severity]),
    retry: retryText(code),
    since: code.since,
  });
  const lines = [
    `${code.id}${CODE_SEPARATOR}${errorText(code.id)}`,
    t(I18N_KEYS.CODES_DESCRIPTION, { description: code.description }),
    facts,
  ];
  return (code.deprecated ? [...lines, t(I18N_KEYS.CODES_RETIRED)] : lines).join(LINE_BREAK);
}

/** The explanation below the table, with the button that copies it. */
function Detail(props: { code: ErrorCodeInfo }) {
  return (
    <div class={styles.detail}>
      <div class={styles.body}>
        <p>
          <b class={styles.mono}>{props.code.id}</b> {errorText(props.code.id)}
        </p>
        <p class={styles.note}>
          {t(I18N_KEYS.CODES_DESCRIPTION, { description: props.code.description })}
        </p>
        <p class={styles.note}>
          {t(I18N_KEYS.CODES_FACTS, {
            group: props.code.group,
            severity: t(SEVERITY_KEY[props.code.severity]),
            retry: retryText(props.code),
            since: props.code.since,
          })}
        </p>
        <Show when={props.code.deprecated}>
          <p class={styles.note}>{t(I18N_KEYS.CODES_RETIRED)}</p>
        </Show>
      </div>
      <CopyDetails text={explanation(props.code)} />
    </div>
  );
}

/** One row of the table; a click opens the explanation. */
function Row(props: { code: ErrorCodeInfo; open: boolean; onToggle: () => void }) {
  const code = () => props.code;
  return (
    <tr class={cx(props.open && styles.selected)}>
      <td>
        <button
          type="button"
          class={`${styles.id} ${styles.mono}`}
          aria-expanded={props.open}
          onClick={props.onToggle}
        >
          {code().id}
        </button>
      </td>
      <td>{errorText(code().id)}</td>
      <td class={styles.mono}>{code().group}</td>
      <td>
        <Pill tone={SEVERITY_TONE[code().severity]}>{t(SEVERITY_KEY[code().severity])}</Pill>
      </td>
      <td>{retryText(code())}</td>
      <td class={styles.mono}>{code().since}</td>
    </tr>
  );
}

/** The filter state of the screen: what the user typed and chose, and the codes that pass it. */
function useFilter() {
  const [query, setQuery] = createSignal("");
  const [severity, setSeverity] = createSignal<string>(SEVERITY_ALL);
  const [group, setGroup] = createSignal<string>(GROUP_ALL);
  const codes = allCodes();
  const shown = () =>
    filterCodes(codes, { query: query(), severity: severity(), group: group() }, (code) =>
      errorText(code.id),
    );
  return { codes, shown, query, setQuery, severity, setSeverity, group, setGroup };
}

type Filter = ReturnType<typeof useFilter>;

/** The choice of one group of codes. */
function GroupSelect(props: { filter: Filter }) {
  return (
    <select
      id={GROUP_FIELD_ID}
      class={styles.field}
      aria-label={t(I18N_KEYS.CODES_GROUP_LABEL)}
      value={props.filter.group()}
      onChange={(event) => props.filter.setGroup(event.currentTarget.value)}
    >
      <option value={GROUP_ALL}>{t(I18N_KEYS.CODES_GROUP_ALL)}</option>
      <For each={groupsOf(props.filter.codes)}>
        {(prefix) => <option value={prefix}>{prefix}</option>}
      </For>
    </select>
  );
}

/** The search field, the severity buttons, the group choice and the way back. */
function Tools(props: { filter: Filter }) {
  const filter = () => props.filter;
  const severityButton = (value: string, label: string) => (
    <button
      type="button"
      class={styles.chip}
      aria-pressed={filter().severity() === value}
      onClick={() => filter().setSeverity(value)}
    >
      {label}
    </button>
  );
  return (
    <div class={styles.tools}>
      <input
        id={SEARCH_FIELD_ID}
        class={cx(styles.field, styles.search)}
        type="search"
        value={filter().query()}
        placeholder={t(I18N_KEYS.CODES_SEARCH_PLACEHOLDER)}
        aria-label={t(I18N_KEYS.CODES_SEARCH_LABEL)}
        onInput={(event) => filter().setQuery(event.currentTarget.value)}
      />
      {severityButton(SEVERITY_ALL, t(I18N_KEYS.CODES_SEVERITY_ALL))}
      <For each={SEVERITIES}>{(value) => severityButton(value, t(SEVERITY_KEY[value]))}</For>
      <GroupSelect filter={filter()} />
      <button
        type="button"
        class={cx(styles.chip, styles.back)}
        onClick={() => navigate(SETTINGS_ROUTE.path)}
      >
        {t(I18N_KEYS.CODES_BACK)}
      </button>
    </div>
  );
}

/** The head of the table. */
function Head() {
  return (
    <thead>
      <tr>
        <th scope="col">{t(I18N_KEYS.CODES_COL_CODE)}</th>
        <th scope="col">{t(I18N_KEYS.CODES_COL_MESSAGE)}</th>
        <th scope="col">{t(I18N_KEYS.CODES_COL_GROUP)}</th>
        <th scope="col">{t(I18N_KEYS.CODES_COL_SEVERITY)}</th>
        <th scope="col">{t(I18N_KEYS.CODES_COL_RETRY)}</th>
        <th scope="col">{t(I18N_KEYS.CODES_COL_SINCE)}</th>
      </tr>
    </thead>
  );
}

/** The table of the codes that pass the filter, with the explanation of the opened one. */
function Table(props: { shown: ErrorCodeInfo[] }) {
  const [opened, setOpened] = createSignal<string | null>(null);
  const openedCode = () => props.shown.find((code) => code.id === opened());
  return (
    <div class={styles.card}>
      <table class={styles.table}>
        <Head />
        <tbody>
          <For each={props.shown}>
            {(code) => (
              <Row
                code={code}
                open={opened() === code.id}
                onToggle={() => setOpened(opened() === code.id ? null : code.id)}
              />
            )}
          </For>
        </tbody>
      </table>
      <Show when={props.shown.length === 0}>
        <p class={styles.note}>{t(I18N_KEYS.CODES_EMPTY)}</p>
      </Show>
      <Show when={openedCode()}>{(code) => <Detail code={code()} />}</Show>
    </div>
  );
}

/** The list of all error codes to look up, the counterpart of `codeskimr codes`. */
export function CodesScreen() {
  const filter = useFilter();
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.CODES_TITLE)}</h1>
        <p class={styles.sub}>{t(I18N_KEYS.CODES_SUBTITLE, { count: filter.shown().length })}</p>
      </header>
      <Tools filter={filter} />
      <Table shown={filter.shown()} />
      <p class={styles.note}>{t(I18N_KEYS.CODES_NOTE_RETIRED)}</p>
    </>
  );
}
