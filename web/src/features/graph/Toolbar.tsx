import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEPTHS, LEVEL } from "./consts";
import type { Level } from "./types";
import styles from "./Toolbar.module.css";

type ToolbarProps = {
  level: Level;
  onLevel: (level: Level) => void;
  /** Whether a type is known to put in the middle: the one chosen in the explorer. */
  focusable: boolean;
  onTypeFocus: () => void;
  depth: number;
  onDepth: (depth: number) => void;
  weak: boolean;
  onWeak: (on: boolean) => void;
  violationsOnly: boolean;
  onViolationsOnly: (on: boolean) => void;
  onFit: () => void;
};

/** One button of a group of exclusive choices. */
function Choice(props: {
  on: boolean;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  title?: string | undefined;
}) {
  return (
    <button
      type="button"
      class={cx(styles.level, props.on && styles.on)}
      aria-pressed={props.on}
      disabled={props.disabled}
      title={props.title}
      onClick={props.onClick}
    >
      {props.label}
    </button>
  );
}

/** The three levels of the map. */
function Levels(props: ToolbarProps) {
  return (
    <div class={styles.levels} role="group" aria-label={t(I18N_KEYS.GRAPH_LEVEL_LABEL)}>
      <Choice
        on={props.level === LEVEL.layers}
        label={t(I18N_KEYS.GRAPH_MODE_LAYERS)}
        onClick={() => props.onLevel(LEVEL.layers)}
      />
      <Choice
        on={props.level === LEVEL.namespaces}
        label={t(I18N_KEYS.GRAPH_MODE_NAMESPACES)}
        onClick={() => props.onLevel(LEVEL.namespaces)}
      />
      <Choice
        on={props.level === LEVEL.types}
        label={t(I18N_KEYS.GRAPH_MODE_TYPE_FOCUS)}
        disabled={!props.focusable}
        title={props.focusable ? undefined : t(I18N_KEYS.GRAPH_TYPES_PICK)}
        onClick={props.onTypeFocus}
      />
    </div>
  );
}

/** The depth of the namespace level. */
function Depths(props: ToolbarProps) {
  return (
    <div class={styles.levels} role="group" aria-label={t(I18N_KEYS.GRAPH_DEPTH_LABEL)}>
      <For each={DEPTHS}>
        {(item) => (
          <Choice
            on={props.depth === item.value}
            label={t(item.label)}
            onClick={() => props.onDepth(item.value)}
          />
        )}
      </For>
    </div>
  );
}

/** A checkbox with its label. */
function Toggle(props: {
  checked: boolean;
  label: string;
  onChange: (on: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label class={styles.toggle}>
      <input
        type="checkbox"
        checked={props.checked}
        disabled={props.disabled}
        onChange={(event) => props.onChange(event.currentTarget.checked)}
      />
      {props.label}
    </label>
  );
}

/** The switches above the map (mock 3): the level, "all connections weak", "violations only" and
 * the hint on the mouse; `F` fits the picture to the view. */
export function Toolbar(props: ToolbarProps) {
  return (
    <div class={styles.bar}>
      <Levels {...props} />
      <Show when={props.level === LEVEL.namespaces}>
        <Depths {...props} />
      </Show>
      <Toggle checked={props.weak} label={t(I18N_KEYS.GRAPH_TOGGLE_WEAK)} onChange={props.onWeak} />
      <Toggle
        checked={props.violationsOnly}
        disabled={props.level !== LEVEL.layers}
        label={t(I18N_KEYS.GRAPH_TOGGLE_VIOLATIONS_ONLY)}
        onChange={props.onViolationsOnly}
      />
      <button type="button" class={styles.fit} onClick={props.onFit}>
        {t(I18N_KEYS.GRAPH_ACTION_FIT)} <kbd class={styles.kbd}>{t(I18N_KEYS.GRAPH_FIT_KEY)}</kbd>
      </button>
      <span class={styles.hint}>{t(I18N_KEYS.GRAPH_HINT_NAVIGATION)}</span>
    </div>
  );
}
