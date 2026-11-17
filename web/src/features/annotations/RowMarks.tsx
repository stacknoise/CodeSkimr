import { Show } from "solid-js";
import { SHIELD } from "./consts";
import { criticalHidden } from "./join";
import { CounterChips, CriticalCount, CriticalShield } from "./Marks";
import { annotationIndex, countersFor } from "./store";
import type { Target } from "./types";

/** What a row of the tree says about annotations (mock 14, variant B): a small shield on a critical
 * symbol, the counters of its notes and tasks, and on a collapsed row the number of critical
 * symbols it hides. The edge stripe is the class of the row (`isCritical`). */
export function RowMarks(props: {
  target: Target | null;
  scope: string | null;
  names: readonly string[];
  /** Whether the row is a folded parent: only then the number of hidden critical symbols shows. */
  folded: boolean;
}) {
  const counters = () => countersFor(props.target);
  const hidden = () =>
    props.folded ? criticalHidden(annotationIndex(), props.scope, props.names) : 0;
  return (
    <>
      <Show when={counters()?.critical}>
        <CriticalShield size={SHIELD.tree} />
      </Show>
      <CounterChips counters={counters()} />
      <Show when={hidden() > 0}>
        <CriticalCount count={hidden()} />
      </Show>
    </>
  );
}

/** Whether a symbol is marked system critical (the edge stripe of its row). */
export function isCritical(target: Target | null): boolean {
  return countersFor(target)?.critical ?? false;
}
