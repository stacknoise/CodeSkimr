import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import { CRUMB_SEPARATOR, HOTKEY, PILL_SEPARATOR } from "./consts";
import { MemberTable } from "./MemberTable";
import { MetricStrip } from "./MetricStrip";
import type { MemberRow, SymbolInfo, Thresholds } from "./types";
import styles from "./DetailPane.module.css";

/** Id of the text that says why the three buttons are off. */
const SOON_ID = "explorer-action-soon";

function Breadcrumb(props: { info: SymbolInfo }) {
  return (
    <div class={styles.crumbs}>
      <For each={props.info.breadcrumb}>
        {(crumb) => (
          <>
            <span>{crumb.name}</span>
            <span class={styles.separator}>{CRUMB_SEPARATOR}</span>
          </>
        )}
      </For>
      <span class={styles.here}>{props.info.name}</span>
    </div>
  );
}

function Action(props: { label: string; hotkey: string }) {
  return (
    <button type="button" aria-disabled={true} aria-describedby={SOON_ID}>
      {props.label} <kbd class={styles.kbd}>{props.hotkey}</kbd>
    </button>
  );
}

/** The three buttons of the header. Usages and graph have no screen yet, so they are shown but
 * off, stay reachable by keyboard and say why (deviation from the mock, see docs/ui.md). */
function Actions() {
  return (
    <div class={styles.actions}>
      <Action label={t(I18N_KEYS.EXPLORER_ACTION_USES)} hotkey={HOTKEY.usages} />
      <Action label={t(I18N_KEYS.EXPLORER_ACTION_GRAPH)} hotkey={HOTKEY.graph} />
      <Action label={t(I18N_KEYS.EXPLORER_ACTION_EDITOR)} hotkey={HOTKEY.editor} />
      <span id={SOON_ID} class={styles.hidden}>
        {t(I18N_KEYS.EXPLORER_ACTION_SOON)}
      </span>
    </div>
  );
}

function Title(props: { info: SymbolInfo }) {
  const kind = () => [props.info.kind, props.info.visibility].filter(Boolean).join(PILL_SEPARATOR);
  return (
    <div class={styles.title}>
      <h1>{props.info.name}</h1>
      <Pill tone={TONE.accent}>{kind()}</Pill>
      <Actions />
    </div>
  );
}

/** The middle column: breadcrumb, name, documentation, metrics and the member table (mock 2). */
export function DetailPane(props: {
  info: SymbolInfo;
  thresholds: Thresholds | null;
  onPickMember: (row: MemberRow) => void;
}) {
  return (
    <section class={styles.pane} aria-label={t(I18N_KEYS.EXPLORER_DETAILS_LABEL)}>
      <Breadcrumb info={props.info} />
      <Title info={props.info} />
      <Show when={props.info.doc}>{(doc) => <p class={styles.doc}>{doc()}</p>}</Show>
      <MetricStrip info={props.info} thresholds={props.thresholds} />
      <MemberTable info={props.info} thresholds={props.thresholds} onPick={props.onPickMember} />
    </section>
  );
}
