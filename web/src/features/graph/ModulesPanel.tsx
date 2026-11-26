import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { count } from "../projects/format";
import { panelStyle } from "../../shared/layout/panelStyle";
import { modulesPanelWidth } from "./layout";
import { CYCLE_JOIN, PERCENT, STABILITY_DIGITS, STABILITY_ONE } from "./consts";
import type { ModuleCycle, ModuleRow, ModulesInfo } from "./types";
import styles from "./ModulesPanel.module.css";

/** A stability in thousandths as a number with two digits in the language (`0,63`). */
export function stabilityText(thousandths: number, language: string): string {
  return (thousandths / STABILITY_ONE).toLocaleString(language, {
    minimumFractionDigits: STABILITY_DIGITS,
    maximumFractionDigits: STABILITY_DIGITS,
  });
}

function Stability(props: { value: number | null }) {
  const width = () => `${((props.value ?? 0) / STABILITY_ONE) * PERCENT}%`;
  return (
    <span class={styles.stability}>
      <span class={styles.bar} aria-hidden="true">
        <i style={{ width: width() }} />
      </span>
      <span class={styles.mono}>
        {props.value === null ? "–" : stabilityText(props.value, currentLanguage())}
      </span>
    </span>
  );
}

function Badges(props: { row: ModuleRow }) {
  return (
    <>
      <Show when={props.row.in_cycle}>
        <span class={cx(styles.pill, styles.bad)}>{t(I18N_KEYS.GRAPH_MODULES_BADGE_CYCLE)}</span>
      </Show>
      <Show when={props.row.violations > 0}>
        <span class={cx(styles.pill, styles.warn)}>
          {t(I18N_KEYS.GRAPH_MODULES_BADGE_VIOLATIONS, { count: props.row.violations })}
        </span>
      </Show>
    </>
  );
}

function Row(props: { row: ModuleRow; selected: boolean; onSelect: (name: string) => void }) {
  const language = () => currentLanguage();
  return (
    <tr class={cx(props.selected && styles.on)}>
      <td>
        <button type="button" class={styles.name} onClick={() => props.onSelect(props.row.name)}>
          {props.row.name}
        </button>
        <div class={styles.badges}>
          <Badges row={props.row} />
        </div>
      </td>
      <td class={cx(styles.num, styles.mono)}>{count(props.row.incoming, language())}</td>
      <td class={cx(styles.num, styles.mono)}>{count(props.row.outgoing, language())}</td>
      <td>
        <Stability value={props.row.stability} />
      </td>
      <td class={cx(styles.num, styles.mono)}>{count(props.row.types, language())}</td>
    </tr>
  );
}

function Table(props: {
  rows: readonly ModuleRow[];
  selected: string | null;
  onSelect: (name: string) => void;
}) {
  return (
    <table class={styles.table}>
      <thead>
        <tr>
          <th>{t(I18N_KEYS.GRAPH_MODULES_COL_MODULE)}</th>
          <th class={styles.num}>{t(I18N_KEYS.GRAPH_MODULES_COL_IN)}</th>
          <th class={styles.num}>{t(I18N_KEYS.GRAPH_MODULES_COL_OUT)}</th>
          <th>{t(I18N_KEYS.GRAPH_MODULES_COL_STABILITY)}</th>
          <th class={styles.num}>{t(I18N_KEYS.GRAPH_MODULES_COL_TYPES)}</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.rows}>
          {(row) => (
            <Row row={row} selected={row.name === props.selected} onSelect={props.onSelect} />
          )}
        </For>
      </tbody>
    </table>
  );
}

function Cycle(props: { cycle: ModuleCycle }) {
  return (
    <div class={styles.item}>
      <div class={styles.mono}>{props.cycle.modules.join(CYCLE_JOIN)}</div>
      <For each={props.cycle.edges}>
        {(edge) => (
          <div class={styles.detail}>
            {t(I18N_KEYS.GRAPH_MODULES_CYCLE_EDGE, {
              from: edge.from,
              to: edge.to,
              count: edge.count,
            })}
          </div>
        )}
      </For>
      <For each={props.cycle.resolution}>
        {(edge) => (
          <div class={styles.resolve}>
            {t(I18N_KEYS.GRAPH_MODULES_CYCLE_RESOLVE, {
              from: edge.from,
              to: edge.to,
              count: edge.count,
            })}
          </div>
        )}
      </For>
    </div>
  );
}

function Cycles(props: { info: ModulesInfo }) {
  return (
    <section class={styles.group}>
      <h3>{t(I18N_KEYS.GRAPH_MODULES_CYCLES_TITLE)}</h3>
      <Show
        when={props.info.cycles.length > 0}
        fallback={<p class={styles.detail}>{t(I18N_KEYS.GRAPH_MODULES_CYCLES_NONE)}</p>}
      >
        <For each={props.info.cycles}>{(cycle) => <Cycle cycle={cycle} />}</For>
      </Show>
    </section>
  );
}

function Violations(props: { info: ModulesInfo }) {
  return (
    <Show when={props.info.violations.length > 0}>
      <section class={styles.group}>
        <h3>{t(I18N_KEYS.GRAPH_MODULES_VIOLATIONS_TITLE)}</h3>
        <For each={props.info.violations}>
          {(found) => (
            <div class={styles.item}>
              {t(I18N_KEYS.GRAPH_MODULES_VIOLATION, {
                from: found.edge.from,
                to: found.edge.to,
                fromLayer: found.from_layer,
                toLayer: found.to_layer,
                count: found.edge.count,
              })}
            </div>
          )}
        </For>
      </section>
    </Show>
  );
}

function Risks(props: { info: ModulesInfo }) {
  const language = () => currentLanguage();
  return (
    <Show when={props.info.risks.length > 0}>
      <section class={styles.group}>
        <h3>{t(I18N_KEYS.GRAPH_MODULES_RISKS_TITLE)}</h3>
        <For each={props.info.risks}>
          {(risk) => (
            <div class={styles.risk} role="note">
              {t(I18N_KEYS.GRAPH_MODULES_RISK, {
                module: risk.module,
                stability: stabilityText(risk.stability, language()),
                incoming: count(risk.incoming, language()),
                other: risk.depends_on,
                otherStability: stabilityText(risk.depends_on_stability, language()),
              })}
            </div>
          )}
        </For>
      </section>
    </Show>
  );
}

/** The panel of the level "Modules" (mock `erw-module`): the table with coupling and stability,
 * the cycles with the edge that ends them, the dependencies against the layering and the modules
 * that stand out. */
export function ModulesPanel(props: {
  info: ModulesInfo;
  selected: string | null;
  onSelect: (name: string) => void;
}) {
  return (
    <aside
      class={styles.panel}
      style={panelStyle(modulesPanelWidth)}
      aria-label={t(I18N_KEYS.GRAPH_MODULES_TABLE_TITLE)}
    >
      <Table rows={props.info.modules} selected={props.selected} onSelect={props.onSelect} />
      <p class={styles.explain}>{t(I18N_KEYS.GRAPH_MODULES_EXPLAIN)}</p>
      <Show when={props.info.modules_more > 0}>
        <p class={styles.explain}>
          {t(I18N_KEYS.GRAPH_MODULES_MORE, { count: props.info.modules_more })}
        </p>
      </Show>
      <Cycles info={props.info} />
      <Violations info={props.info} />
      <Risks info={props.info} />
    </aside>
  );
}
