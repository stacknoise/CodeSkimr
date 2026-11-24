import { For, Show, createSignal, onMount } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import { TOOL_ROLE_TOOLCHAIN, TOOL_STATE } from "../projects/consts";
import { ago } from "../projects/format";
import { languageList, toolTexts } from "../projects/tools";
import { stateText } from "../projects/ToolsPanel";
import type { ToolInfo } from "../projects/types";
import { useNow } from "../projects/useNow";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, HeadRow } from "../../shared/table/ResizableHead";
import { INDEXER_COLUMNS, INDEXER_TABLE_KEY } from "./consts";
import { checkIndexers, commandOf, indexers, indexersChecking, indexersFailed } from "./indexers";
import { COPIED_MS, NO_VALUE } from "./consts";
import styles from "./IndexersPanel.module.css";

/** The short word in the pill: the long text of a crash goes below it. */
function pillText(tool: ToolInfo): string {
  if (tool.state === TOOL_STATE.ready) {
    return t(I18N_KEYS.SETTINGS_INDEXERS_STATE_READY);
  }
  if (tool.state === TOOL_STATE.notRunnable) {
    return t(I18N_KEYS.SETTINGS_INDEXERS_STATE_NOT_RUNNABLE);
  }
  return stateText(tool);
}

function toneOf(tool: ToolInfo) {
  if (tool.state === TOOL_STATE.ready) {
    return TONE.good;
  }
  return tool.state === TOOL_STATE.notRunnable ? TONE.bad : TONE.warn;
}

/** The version a tool reported; a tool that does not start has none. */
function versionOf(tool: ToolInfo): string {
  const usable = tool.state !== TOOL_STATE.notRunnable && tool.state !== TOOL_STATE.missing;
  return (usable ? tool.found : null) ?? NO_VALUE;
}

/** The install text with a button that copies the command in it. */
function Install(props: { tool: ToolInfo }) {
  const text = () => toolTexts(props.tool).install;
  const [copied, setCopied] = createSignal(false);
  const copy = () => {
    const command = commandOf(text());
    if (command === null) {
      return;
    }
    void navigator.clipboard
      .writeText(command)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), COPIED_MS);
      })
      .catch(() => undefined);
  };
  return (
    <div class={styles.hint}>
      <span>{text()}</span>
      <Show when={commandOf(text()) !== null}>
        <button type="button" class={styles.button} onClick={copy}>
          {copied() ? t(I18N_KEYS.SETTINGS_INDEXERS_COPIED) : t(I18N_KEYS.SETTINGS_INDEXERS_COPY)}
        </button>
      </Show>
    </div>
  );
}

function Row(props: { tool: ToolInfo }) {
  const broken = () => props.tool.state === TOOL_STATE.notRunnable;
  return (
    <tr>
      <td>
        <strong class={styles.mono}>{props.tool.tool}</strong>
        <div class={styles.sub}>
          {props.tool.role === TOOL_ROLE_TOOLCHAIN
            ? t(I18N_KEYS.SETTINGS_INDEXERS_TOOLCHAIN, {
                languages: languageList(props.tool.languages),
              })
            : languageList(props.tool.languages)}
        </div>
      </td>
      <td>
        <Pill tone={toneOf(props.tool)}>{pillText(props.tool)}</Pill>
        <Show when={broken()}>
          <div class={`${styles.mono} ${styles.error}`}>
            {props.tool.found ?? t(I18N_KEYS.SETTINGS_INDEXERS_NO_ERROR)}
          </div>
        </Show>
        <Show when={props.tool.state !== TOOL_STATE.ready}>
          <Install tool={props.tool} />
        </Show>
      </td>
      <td class={styles.mono}>{versionOf(props.tool)}</td>
      <td class={`${styles.mono} ${styles.path}`}>{props.tool.path ?? NO_VALUE}</td>
    </tr>
  );
}

function CheckedAt() {
  const now = useNow();
  return (
    <Show when={indexers()?.checked_at}>
      {(seconds) => (
        <span class={styles.muted}>
          {t(I18N_KEYS.SETTINGS_INDEXERS_CHECKED, {
            when: ago(seconds(), now(), currentLanguage()),
          })}
        </span>
      )}
    </Show>
  );
}

/** The title, the time of the last check and the button that checks again. */
function Header() {
  return (
    <div class={styles.top}>
      <div>
        <h2>{t(I18N_KEYS.SETTINGS_INDEXERS_TITLE)}</h2>
        <p class={styles.note}>{t(I18N_KEYS.SETTINGS_INDEXERS_NOTE)}</p>
      </div>
      <div class={styles.actions}>
        <CheckedAt />
        <button
          type="button"
          class={`${styles.button} ${styles.primary}`}
          disabled={indexersChecking()}
          onClick={() => void checkIndexers()}
        >
          {indexersChecking()
            ? t(I18N_KEYS.SETTINGS_INDEXERS_CHECKING)
            : t(I18N_KEYS.SETTINGS_INDEXERS_CHECK_AGAIN)}
        </button>
      </div>
    </div>
  );
}

const columns = createTableColumns(
  INDEXER_TABLE_KEY,
  INDEXER_COLUMNS.map((column) => column.id),
);

function Table(props: { tools: ToolInfo[] }) {
  return (
    <div class={styles.scroll}>
      <table class={styles.table} style={columns.style()}>
        <ColumnWidths columns={columns} />
        <thead>
          <HeadRow columns={columns} heads={INDEXER_COLUMNS} />
        </thead>
        <tbody>
          <For each={props.tools}>{(tool) => <Row tool={tool} />}</For>
        </tbody>
      </table>
    </div>
  );
}

/** The indexers of level 2 on this computer: found or not, version, path and, for one that does
 * not start, the line of its error (mock 17). The check runs when the screen opens. */
export function IndexersPanel() {
  onMount(() => void checkIndexers());
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.SETTINGS_INDEXERS_TITLE)}>
      <Header />
      <Show when={indexersFailed()}>
        <p role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_INDEXERS_FAILED)}
        </p>
      </Show>
      <Show when={indexers()}>{(info) => <Table tools={info().tools} />}</Show>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_INDEXERS_SEARCH_NOTE)}</p>
    </section>
  );
}
