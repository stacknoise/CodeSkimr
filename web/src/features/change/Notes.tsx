import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { OUTCOME_TEXT } from "./consts";
import type { ChangeAnswer, LayerEdgeOut } from "./types";
import parts from "./Parts.module.css";

/** A boxed note under the table: a bold title and the lines that belong to it. */
function Note(props: { title: string; info?: boolean; lines: readonly string[] }) {
  return (
    <div class={cx(parts.note, props.info && parts.noteInfo)}>
      <b>{props.title}</b>
      <ul>
        <For each={props.lines}>{(line) => <li>{line}</li>}</For>
      </ul>
    </div>
  );
}

/** The lines of the edges between layers: who uses whom, and by how many references. */
function edgeLines(edges: readonly LayerEdgeOut[]): string[] {
  return edges.map((edge) =>
    t(I18N_KEYS.CHANGE_VIEW_EDGE, { from: edge.from, to: edge.to, count: edge.count }),
  );
}

/** The notes under the table of the impact: new layer violations and cycles, the critical symbols
 * the difference touches, and a line if the answer was cut. */
export function Notes(props: { answer: ChangeAnswer }) {
  const layers = () => props.answer.layers;
  const any = () =>
    layers().new_violations.length + layers().new_cycles.length + props.answer.critical.length >
      0 || props.answer.is_cut;
  return (
    <Show when={any()}>
      <div class={parts.notes}>
        <Show when={layers().new_violations.length > 0}>
          <Note
            title={t(I18N_KEYS.CHANGE_VIEW_VIOLATIONS, { count: layers().new_violations.length })}
            lines={edgeLines(layers().new_violations)}
          />
        </Show>
        <Show when={layers().new_cycles.length > 0}>
          <Note
            title={t(I18N_KEYS.CHANGE_VIEW_CYCLES, { count: layers().new_cycles.length })}
            lines={edgeLines(layers().new_cycles)}
          />
        </Show>
        <Show when={props.answer.critical.length > 0}>
          <Note
            info
            title={t(I18N_KEYS.CHANGE_VIEW_CRITICAL)}
            lines={props.answer.critical.map((critical) =>
              t(I18N_KEYS.CHANGE_VIEW_CRITICAL_LINE, {
                name: critical.qualified_name,
                outcome: t(OUTCOME_TEXT[critical.outcome] ?? I18N_KEYS.CHANGE_VIEW_OUTCOME_CHANGED),
              }),
            )}
          />
        </Show>
        <Show when={props.answer.is_cut}>
          <p class={parts.plain}>{t(I18N_KEYS.CHANGE_VIEW_CUT)}</p>
        </Show>
      </div>
    </Show>
  );
}
