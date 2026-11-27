import { createSignal, For, Show } from "solid-js";
import { currentLanguage, t, type MessageKey } from "../../shared/i18n/i18n";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, RestHead } from "../../shared/table/ResizableHead";
import { pressSort, type SortState } from "../../shared/table/sort";
import { SortableTh } from "../../shared/table/SortableTh";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { TONE } from "../../shared/ui/consts";
import { count } from "../projects/format";
import { panelStyle } from "../../shared/layout/panelStyle";
import { modulesPanelWidth } from "./layout";
import {
  CYCLE_JOIN,
  MODULE_COLUMN,
  MODULE_COLUMNS,
  MODULE_NUMBER_COLUMNS,
  MODULES_TABLE_KEY,
  NOTE_ROLE,
  NO_STABILITY,
  PERCENT,
  STABILITY_DIGITS,
  STABILITY_ONE,
  type ModuleColumn,
} from "./consts";
import { EdgePlaces } from "./ModulePlaces";
import type { ModuleCycle, ModuleRow, ModulesInfo } from "./types";
import { sortCycles, sortEdges, sortRisks, sortRows, sortViolations } from "./order";
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
        {props.value === null ? NO_STABILITY : stabilityText(props.value, currentLanguage())}
      </span>
    </span>
  );
}

function Badges(props: { row: ModuleRow }) {
  return (
    <>
      <Show when={props.row.in_cycle}>
        <Pill tone={TONE.bad} compact title={t(I18N_KEYS.GRAPH_MODULES_BADGE_CYCLE_HINT)}>
          {t(I18N_KEYS.GRAPH_MODULES_BADGE_CYCLE)}
        </Pill>
      </Show>
      <Show when={props.row.violations > 0}>
        <Pill tone={TONE.warn} compact>
          {t(I18N_KEYS.GRAPH_MODULES_BADGE_VIOLATIONS, { count: props.row.violations })}
        </Pill>
      </Show>
    </>
  );
}

function Row(props: { row: ModuleRow; selected: boolean; onSelect: (name: string) => void }) {
  const language = () => currentLanguage();
  return (
    <tr class={cx(props.selected && styles.on)}>
      <td>
        <button
          type="button"
          class={styles.name}
          aria-pressed={props.selected}
          onClick={() => props.onSelect(props.row.name)}
        >
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

const columns = createTableColumns(MODULES_TABLE_KEY, MODULE_COLUMNS);
const [sort, setSort] = createSignal<SortState | null>(null);

/** The head of a column of the table: it sorts when pressed and has the handle for its width. */
function Head(props: { column: ModuleColumn; label: MessageKey; tip?: MessageKey }) {
  const direction = () => (sort()?.column === props.column ? sort()?.direction : undefined);
  const numeric = () => MODULE_NUMBER_COLUMNS.includes(props.column);
  return (
    <SortableTh
      columns={columns}
      id={props.column}
      label={t(props.label)}
      class={numeric() ? styles.num : undefined}
      title={props.tip ? t(props.tip) : undefined}
      direction={direction()}
      onPress={() => setSort(pressSort(sort(), props.column, numeric()))}
    />
  );
}

/** The head row of the table of modules. */
function Heads() {
  return (
    <thead>
      <tr>
        <Head column={MODULE_COLUMN.name} label={I18N_KEYS.GRAPH_MODULES_COL_MODULE} />
        <Head
          column={MODULE_COLUMN.incoming}
          label={I18N_KEYS.GRAPH_MODULES_COL_IN}
          tip={I18N_KEYS.GRAPH_MODULES_COL_IN_HINT}
        />
        <Head
          column={MODULE_COLUMN.outgoing}
          label={I18N_KEYS.GRAPH_MODULES_COL_OUT}
          tip={I18N_KEYS.GRAPH_MODULES_COL_OUT_HINT}
        />
        <Head
          column={MODULE_COLUMN.stability}
          label={I18N_KEYS.GRAPH_MODULES_COL_STABILITY}
          tip={I18N_KEYS.GRAPH_MODULES_COL_STABILITY_HINT}
        />
        <Head column={MODULE_COLUMN.types} label={I18N_KEYS.GRAPH_MODULES_COL_TYPES} />
        <RestHead columns={columns} />
      </tr>
    </thead>
  );
}

function Table(props: {
  rows: readonly ModuleRow[];
  selected: string | null;
  onSelect: (name: string) => void;
}) {
  const language = () => currentLanguage();
  return (
    <table class={styles.table} style={columns.style()}>
      <ColumnWidths columns={columns} />
      <Heads />
      <tbody>
        <For each={sortRows(props.rows, language(), sort())}>
          {(row) => (
            <Row row={row} selected={row.name === props.selected} onSelect={props.onSelect} />
          )}
        </For>
      </tbody>
    </table>
  );
}

function Cycle(props: { project: number; cycle: ModuleCycle }) {
  return (
    <div class={styles.item}>
      <div class={styles.mono}>{props.cycle.modules.join(CYCLE_JOIN)}</div>
      <For each={sortEdges(props.cycle.edges, currentLanguage())}>
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
      <For each={sortEdges(props.cycle.resolution, currentLanguage())}>
        {(edge) => (
          <div class={styles.resolve}>
            {t(I18N_KEYS.GRAPH_MODULES_CYCLE_RESOLVE, {
              from: edge.from,
              to: edge.to,
              count: edge.count,
            })}
            <EdgePlaces project={props.project} edge={edge} />
          </div>
        )}
      </For>
    </div>
  );
}

function Cycles(props: { project: number; info: ModulesInfo }) {
  return (
    <section class={styles.group}>
      <h3>{t(I18N_KEYS.GRAPH_MODULES_CYCLES_TITLE)}</h3>
      <Show
        when={props.info.cycles.length > 0}
        fallback={<p class={styles.detail}>{t(I18N_KEYS.GRAPH_MODULES_CYCLES_NONE)}</p>}
      >
        <For each={sortCycles(props.info.cycles, currentLanguage())}>
          {(cycle) => <Cycle project={props.project} cycle={cycle} />}
        </For>
      </Show>
    </section>
  );
}

function Violations(props: { project: number; info: ModulesInfo }) {
  return (
    <Show when={props.info.violations.length > 0}>
      <section class={styles.group}>
        <h3>{t(I18N_KEYS.GRAPH_MODULES_VIOLATIONS_TITLE)}</h3>
        <For each={sortViolations(props.info.violations, currentLanguage())}>
          {(found) => (
            <div class={styles.item}>
              {t(I18N_KEYS.GRAPH_MODULES_VIOLATION, {
                from: found.edge.from,
                to: found.edge.to,
                fromLayer: found.from_layer,
                toLayer: found.to_layer,
                count: found.edge.count,
              })}
              <EdgePlaces project={props.project} edge={found.edge} />
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
        <For each={sortRisks(props.info.risks, language())}>
          {(risk) => (
            <div class={styles.risk} role={NOTE_ROLE}>
              {t(I18N_KEYS.GRAPH_MODULES_RISK, {
                module: risk.module,
                stability: stabilityText(risk.stability, language()),
                incoming: risk.incoming,
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
  project: number;
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
      <Show when={props.info.modules_more > 0}>
        <p class={styles.explain}>
          {t(I18N_KEYS.GRAPH_MODULES_MORE, { count: props.info.modules_more })}
        </p>
      </Show>
      <Cycles project={props.project} info={props.info} />
      <Violations project={props.project} info={props.info} />
      <Risks info={props.info} />
    </aside>
  );
}
