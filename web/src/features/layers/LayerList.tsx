import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { MOVE, MOVE_MARK, REMOVE_MARK } from "./consts";
import type { LayerEntry } from "./types";
import styles from "./LayersScreen.module.css";

/** What the list tells the screen to change. */
export interface LayerActions {
  onRename: (index: number, name: string) => void;
  onMove: (index: number, step: number) => void;
  onRemove: (index: number) => void;
  onRemoveNamespace: (index: number, prefix: string) => void;
  onAddSelection: (index: number) => void;
}

/** A small square button with a mark and a spoken name. */
function Mark(props: { mark: string; label: string; disabled?: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      class={styles.small}
      disabled={props.disabled}
      aria-label={props.label}
      onClick={() => props.onClick()}
    >
      {props.mark}
    </button>
  );
}

/** The name of a layer and the buttons that move or remove it. */
function LayerHead(props: {
  layer: LayerEntry;
  index: number;
  last: boolean;
  actions: LayerActions;
}) {
  const { actions, index, layer } = props;
  const name = () => ({ name: layer.name });
  return (
    <div class={styles.cardHead}>
      <input
        class={styles.field}
        value={layer.name}
        aria-label={t(I18N_KEYS.LAYERS_NAME_LABEL)}
        onInput={(event) => actions.onRename(index, event.currentTarget.value)}
      />
      <Mark
        mark={MOVE_MARK.up}
        label={t(I18N_KEYS.LAYERS_MOVE_UP, name())}
        disabled={index === 0}
        onClick={() => actions.onMove(index, MOVE.up)}
      />
      <Mark
        mark={MOVE_MARK.down}
        label={t(I18N_KEYS.LAYERS_MOVE_DOWN, name())}
        disabled={props.last}
        onClick={() => actions.onMove(index, MOVE.down)}
      />
      <Mark
        mark={REMOVE_MARK}
        label={t(I18N_KEYS.LAYERS_REMOVE, name())}
        onClick={() => actions.onRemove(index)}
      />
    </div>
  );
}

/** The namespaces a layer owns, each with the button that takes it out. */
function Chips(props: { layer: LayerEntry; index: number; actions: LayerActions }) {
  return (
    <ul class={styles.chips}>
      <For each={props.layer.namespaces}>
        {(prefix) => (
          <li class={styles.chip}>
            <span class={styles.mono}>{prefix}</span>
            <button
              type="button"
              aria-label={t(I18N_KEYS.LAYERS_REMOVE_NAMESPACE, { name: prefix })}
              onClick={() => props.actions.onRemoveNamespace(props.index, prefix)}
            >
              {REMOVE_MARK}
            </button>
          </li>
        )}
      </For>
    </ul>
  );
}

/** One layer: its name, the namespaces it owns and the buttons that change it. */
function LayerCard(props: {
  layer: LayerEntry;
  index: number;
  last: boolean;
  canAdd: boolean;
  actions: LayerActions;
}) {
  return (
    <li class={styles.card}>
      <LayerHead
        layer={props.layer}
        index={props.index}
        last={props.last}
        actions={props.actions}
      />
      <Chips layer={props.layer} index={props.index} actions={props.actions} />
      <button
        type="button"
        class={styles.secondary}
        disabled={!props.canAdd}
        onClick={() => props.actions.onAddSelection(props.index)}
      >
        {t(I18N_KEYS.LAYERS_ADD_SELECTION)}
      </button>
    </li>
  );
}

/** The layers of the draft, outermost first. */
export function LayerList(props: {
  layers: readonly LayerEntry[];
  canAdd: boolean;
  actions: LayerActions;
}) {
  return (
    <Show
      when={props.layers.length > 0}
      fallback={<p class={styles.note}>{t(I18N_KEYS.LAYERS_EMPTY)}</p>}
    >
      <ul class={styles.cards}>
        <For each={props.layers}>
          {(layer, index) => (
            <LayerCard
              layer={layer}
              index={index()}
              last={index() === props.layers.length - 1}
              canAdd={props.canAdd}
              actions={props.actions}
            />
          )}
        </For>
      </ul>
    </Show>
  );
}
