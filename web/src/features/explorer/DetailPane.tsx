import { For, type JSX, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { CounterChips, CriticalLabel } from "../annotations/Marks";
import { countersFor } from "../annotations/store";
import { CRUMB_SEPARATOR, HOTKEY, PILL_SEPARATOR } from "./consts";
import { languageName } from "../projects/format";
import { showInGraph } from "../graph/show";
import { showUsages } from "../../shared/navigation/navigation";
import { openInEditor } from "./editor";
import { memberFromTable, selectType, shownHasLayer, shownMember } from "./explorer";
import { MemberTable } from "./MemberTable";
import { MetricStrip } from "./MetricStrip";
import type { MemberRow, SymbolInfo, Thresholds } from "./types";
import styles from "./DetailPane.module.css";

/** The way to the shown symbol: the parents, and for a member its type, then the symbol. */
function Breadcrumb(props: { info: SymbolInfo; member: MemberRow | null }) {
  const trail = () => [
    ...props.info.breadcrumb.map((crumb) => crumb.name),
    ...(props.member === null ? [] : [props.info.name]),
  ];
  return (
    <div class={styles.crumbs}>
      <For each={trail()}>
        {(name) => (
          <>
            <span>{name}</span>
            <span class={styles.separator}>{CRUMB_SEPARATOR}</span>
          </>
        )}
      </For>
      <span class={styles.here}>{props.member?.name ?? props.info.name}</span>
    </div>
  );
}

function Action(props: { label: string; hotkey: string; onClick: () => void; disabled?: boolean }) {
  return (
    <button type="button" disabled={props.disabled} onClick={() => props.onClick()}>
      {props.label} <kbd class={styles.kbd}>{props.hotkey}</kbd>
    </button>
  );
}

/** The three buttons of the header: usages, the layer on the dependency map, the editor. */
function Actions(props: { symbol: number }) {
  return (
    <div class={styles.actions}>
      <Action
        label={t(I18N_KEYS.EXPLORER_ACTION_USES)}
        hotkey={HOTKEY.usages}
        onClick={() => showUsages(props.symbol)}
      />
      <Action
        label={t(I18N_KEYS.EXPLORER_ACTION_GRAPH)}
        hotkey={HOTKEY.graph}
        disabled={!shownHasLayer()}
        onClick={() => {
          const project = activeProject();
          if (project !== null) {
            void showInGraph(project, props.symbol);
          }
        }}
      />
      <Action
        label={t(I18N_KEYS.EXPLORER_ACTION_EDITOR)}
        hotkey={HOTKEY.editor}
        onClick={() => {
          const project = activeProject();
          if (project !== null) {
            void openInEditor(project, props.symbol);
          }
        }}
      />
    </div>
  );
}

/** Name, kind and the buttons of the shown symbol: the member if one is selected, else the type. */
function Title(props: { info: SymbolInfo; member: MemberRow | null }) {
  const shown = () => props.member ?? props.info;
  const back = () => {
    const project = activeProject();
    if (project !== null) {
      void selectType(project, props.info.id);
    }
  };
  const kind = () => [shown().kind, shown().visibility].filter(Boolean).join(PILL_SEPARATOR);
  const counters = () => countersFor({ kind: shown().kind, name: shown().qualified_name });
  return (
    <div class={styles.title}>
      <Show when={props.member !== null && memberFromTable()}>
        <button type="button" class={styles.back} onClick={back}>
          {t(I18N_KEYS.EXPLORER_BACK_TO_MEMBERS)}
        </button>
      </Show>
      <h1>{shown().name}</h1>
      <Show when={counters()?.critical}>
        <CriticalLabel shield />
      </Show>
      <CounterChips counters={counters()} />
      <Pill tone={TONE.accent}>{kind()}</Pill>
      <Pill>{languageName(props.info.language)}</Pill>
      <Actions symbol={shown().id} />
    </div>
  );
}

/** The middle column: breadcrumb, name and buttons, and for a type its documentation, metrics and
 * member table (mock 2). With a member selected the source takes the place of the facts and the member table. */
export function DetailPane(props: {
  info: SymbolInfo;
  thresholds: Thresholds | null;
  onPickMember: (row: MemberRow) => void;
  /** The source, shown instead of the facts and the member table when a member is selected. */
  source: JSX.Element;
}) {
  const member = () => shownMember();
  return (
    <section
      class={cx(styles.pane, member() !== null && styles.compact)}
      aria-label={t(I18N_KEYS.EXPLORER_DETAILS_LABEL)}
    >
      <Breadcrumb info={props.info} member={member()} />
      <Title info={props.info} member={member()} />
      <Show when={member() === null} fallback={props.source}>
        <Show when={props.info.doc}>{(doc) => <p class={styles.doc}>{doc()}</p>}</Show>
        <MetricStrip info={props.info} thresholds={props.thresholds} />
        <MemberTable info={props.info} thresholds={props.thresholds} onPick={props.onPickMember} />
      </Show>
    </section>
  );
}
