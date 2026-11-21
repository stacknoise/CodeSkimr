import { createMemo, createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { DEPTH_PROPERTY, OPENER } from "./consts";
import { ownerOf } from "./draft";
import { buildTree, initiallyOpen, visibleRows } from "./prefixes";
import type { LayerEntry, NamespaceEntry, PrefixNode } from "./types";
import styles from "./LayersScreen.module.css";

/** The button that opens or closes a row; a row without children has an empty place instead. */
function Opener(props: { node: PrefixNode; open: boolean; onToggle: (prefix: string) => void }) {
  return (
    <Show when={props.node.children.length > 0} fallback={<span class={styles.opener} />}>
      <button
        type="button"
        class={styles.opener}
        aria-expanded={props.open}
        aria-label={t(props.open ? I18N_KEYS.LAYERS_COLLAPSE : I18N_KEYS.LAYERS_EXPAND, {
          name: props.node.prefix,
        })}
        onClick={() => props.onToggle(props.node.prefix)}
      >
        {props.open ? OPENER.open : OPENER.closed}
      </button>
    </Show>
  );
}

/** One row of the tree: opener, tick, name, size and the layer that owns it now. */
function Row(props: {
  node: PrefixNode;
  open: boolean;
  chosen: boolean;
  owner: string | null;
  onToggleOpen: (prefix: string) => void;
  onToggleChosen: (prefix: string) => void;
}) {
  return (
    <li class={styles.row} style={{ [DEPTH_PROPERTY]: props.node.depth }}>
      <Opener node={props.node} open={props.open} onToggle={props.onToggleOpen} />
      <label class={styles.pick}>
        <input
          type="checkbox"
          checked={props.chosen}
          aria-label={t(I18N_KEYS.LAYERS_SELECT, { name: props.node.prefix })}
          onChange={() => props.onToggleChosen(props.node.prefix)}
        />
        <span class={styles.label}>{props.node.label}</span>
      </label>
      <span class={styles.count}>{t(I18N_KEYS.LAYERS_TYPES, { count: props.node.types })}</span>
      <Show when={props.owner}>
        {(layer) => (
          <span class={styles.owner}>{t(I18N_KEYS.LAYERS_OWNED_BY, { layer: layer() })}</span>
        )}
      </Show>
    </li>
  );
}

/** The namespaces of the scan as a tree of prefixes; the ticked ones go to `onChosen`. */
export function NamespacePicker(props: {
  namespaces: readonly NamespaceEntry[];
  layers: readonly LayerEntry[];
  chosen: ReadonlySet<string>;
  onChosen: (next: ReadonlySet<string>) => void;
}) {
  const roots = createMemo(() => buildTree(props.namespaces));
  const [open, setOpen] = createSignal<ReadonlySet<string>>(initiallyOpen(roots()));
  const rows = () => visibleRows(roots(), open());
  const flip = (set: ReadonlySet<string>, prefix: string) => {
    const next = new Set(set);
    if (!next.delete(prefix)) {
      next.add(prefix);
    }
    return next;
  };
  return (
    <ul class={styles.tree}>
      <For each={rows()}>
        {(node) => (
          <Row
            node={node}
            open={open().has(node.prefix)}
            chosen={props.chosen.has(node.prefix)}
            owner={ownerOf(props.layers, node.prefix)}
            onToggleOpen={(prefix) => setOpen(flip(open(), prefix))}
            onToggleChosen={(prefix) => props.onChosen(flip(props.chosen, prefix))}
          />
        )}
      </For>
    </ul>
  );
}
