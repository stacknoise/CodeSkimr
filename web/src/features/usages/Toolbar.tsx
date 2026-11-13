import { For } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEPTH_ALL, DEPTHS, ROLE, WAYS } from "./consts";
import {
  direction,
  setDirection,
  setTestsShown,
  setWalkDepth,
  testsShown,
  walkDepth,
} from "./usages";
import styles from "./Toolbar.module.css";

/** The level of the mock: only the symbol level exists here; the other two belong to the graph
 * screen and are shown off, with the reason as a tooltip. */
function Levels() {
  const soon = t(I18N_KEYS.USAGES_LEVEL_SOON);
  return (
    <div class={styles.group} role={ROLE.group} aria-label={t(I18N_KEYS.USAGES_LEVEL_LABEL)}>
      <button type="button" aria-disabled={true} title={soon}>
        {t(I18N_KEYS.USAGES_LEVEL_NAMESPACES)}
      </button>
      <button type="button" aria-disabled={true} title={soon}>
        {t(I18N_KEYS.USAGES_LEVEL_TYPES)}
      </button>
      <button type="button" class={styles.current} aria-pressed={true}>
        {t(I18N_KEYS.USAGES_LEVEL_SYMBOL)}
      </button>
    </div>
  );
}

function Directions() {
  return (
    <div class={styles.group} role={ROLE.group} aria-label={t(I18N_KEYS.USAGES_DIRECTION_LABEL)}>
      <For each={WAYS}>
        {(way) => (
          <button
            type="button"
            class={cx(direction() === way.id && styles.on)}
            aria-pressed={direction() === way.id}
            onClick={() => setDirection(way.id)}
          >
            {t(way.label)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The toolbar of screen 6: level, direction, depth, tests and the key hint (mock 6). */
export function Toolbar() {
  return (
    <div class={styles.bar}>
      <Levels />
      <Directions />
      <label class={styles.field}>
        {t(I18N_KEYS.USAGES_DEPTH)}
        <select
          value={walkDepth()}
          onChange={(event) => setWalkDepth(Number(event.currentTarget.value))}
        >
          <For each={DEPTHS}>
            {(depth) => (
              <option value={depth} selected={depth === walkDepth()}>
                {depth === DEPTH_ALL ? t(I18N_KEYS.USAGES_DEPTH_ALL) : depth}
              </option>
            )}
          </For>
        </select>
      </label>
      <label class={styles.field}>
        <input
          type="checkbox"
          checked={testsShown()}
          onChange={(event) => setTestsShown(event.currentTarget.checked)}
        />
        {t(I18N_KEYS.USAGES_SHOW_TESTS)}
      </label>
      <span class={styles.hint}>{t(I18N_KEYS.USAGES_HINT_OPEN)}</span>
    </div>
  );
}
