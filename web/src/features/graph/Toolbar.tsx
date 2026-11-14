import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEPTHS, LEVEL } from "./consts";
import type { Level } from "./types";
import styles from "./Toolbar.module.css";

/** The switches above the map (mock 3): the level, "all connections weak", "violations only" and
 * the hint on the mouse; `F` fits the picture to the view. */
export function Toolbar(props: {
  level: Level;
  onLevel: (level: Level) => void;
  depth: number;
  onDepth: (depth: number) => void;
  weak: boolean;
  onWeak: (on: boolean) => void;
  violationsOnly: boolean;
  onViolationsOnly: (on: boolean) => void;
  onFit: () => void;
}) {
  return (
    <div class={styles.bar}>
      <div class={styles.levels} role="group" aria-label={t(I18N_KEYS.GRAPH_LEVEL_LABEL)}>
        <button
          type="button"
          class={cx(styles.level, props.level === LEVEL.layers && styles.on)}
          aria-pressed={props.level === LEVEL.layers}
          onClick={() => props.onLevel(LEVEL.layers)}
        >
          {t(I18N_KEYS.GRAPH_MODE_LAYERS)}
        </button>
        <button
          type="button"
          class={cx(styles.level, props.level === LEVEL.namespaces && styles.on)}
          aria-pressed={props.level === LEVEL.namespaces}
          onClick={() => props.onLevel(LEVEL.namespaces)}
        >
          {t(I18N_KEYS.GRAPH_MODE_NAMESPACES)}
        </button>
      </div>
      <Show when={props.level === LEVEL.namespaces}>
        <div class={styles.levels} role="group" aria-label={t(I18N_KEYS.GRAPH_DEPTH_LABEL)}>
          <For each={DEPTHS}>
            {(item) => (
              <button
                type="button"
                class={cx(styles.level, props.depth === item.value && styles.on)}
                aria-pressed={props.depth === item.value}
                onClick={() => props.onDepth(item.value)}
              >
                {t(item.label)}
              </button>
            )}
          </For>
        </div>
      </Show>
      <label class={styles.toggle}>
        <input
          type="checkbox"
          checked={props.weak}
          onChange={(event) => props.onWeak(event.currentTarget.checked)}
        />
        {t(I18N_KEYS.GRAPH_TOGGLE_WEAK)}
      </label>
      <label class={styles.toggle}>
        <input
          type="checkbox"
          checked={props.violationsOnly}
          disabled={props.level === LEVEL.namespaces}
          onChange={(event) => props.onViolationsOnly(event.currentTarget.checked)}
        />
        {t(I18N_KEYS.GRAPH_TOGGLE_VIOLATIONS_ONLY)}
      </label>
      <button type="button" class={styles.fit} onClick={props.onFit}>
        {t(I18N_KEYS.GRAPH_ACTION_FIT)} <kbd class={styles.kbd}>{t(I18N_KEYS.GRAPH_FIT_KEY)}</kbd>
      </button>
      <span class={styles.hint}>{t(I18N_KEYS.GRAPH_HINT_NAVIGATION)}</span>
    </div>
  );
}
