import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import { CRUMB_SEPARATOR, PILL_SEPARATOR } from "./consts";
import { MemberTable } from "./MemberTable";
import { MetricStrip } from "./MetricStrip";
import type { SymbolInfo } from "./types";
import styles from "./DetailPane.module.css";

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

/** The three buttons of the header. Usages and graph have no screen yet, so they are shown
 * but off, and say why (deviation from the mock, see docs/ui.md). */
function Actions() {
  const soon = t(I18N_KEYS.EXPLORER_ACTION_SOON);
  return (
    <div class={styles.actions}>
      <button type="button" disabled title={soon}>
        {t(I18N_KEYS.EXPLORER_ACTION_USES)}
      </button>
      <button type="button" disabled title={soon}>
        {t(I18N_KEYS.EXPLORER_ACTION_GRAPH)}
      </button>
      <button type="button" disabled title={soon}>
        {t(I18N_KEYS.EXPLORER_ACTION_EDITOR)}
      </button>
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
export function DetailPane(props: { info: SymbolInfo; onPickMember: (id: number) => void }) {
  return (
    <section class={styles.pane} aria-label={t(I18N_KEYS.EXPLORER_DETAILS_LABEL)}>
      <Breadcrumb info={props.info} />
      <Title info={props.info} />
      <Show when={props.info.doc}>{(doc) => <p class={styles.doc}>{doc()}</p>}</Show>
      <MetricStrip info={props.info} />
      <MemberTable info={props.info} onPick={props.onPickMember} />
    </section>
  );
}
