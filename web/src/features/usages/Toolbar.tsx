import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEPTH_ALL, ROLE, WAYS } from "./consts";
import {
  direction,
  setDirection,
  setTestsShown,
  setWalkDepth,
  testsShown,
  depthChoices,
  walkDepth,
} from "./usages";
import styles from "./Toolbar.module.css";

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

/** The toolbar of screen 6: the way back, direction, depth, tests and the key hint (mock 6). */
export function Toolbar(props: { onBack: () => void }) {
  return (
    <div class={styles.bar}>
      <div class={styles.group}>
        <button type="button" onClick={props.onBack}>
          {t(I18N_KEYS.USAGES_BACK)}
        </button>
      </div>
      <Directions />
      <Show when={depthChoices().length > 1}>
        <label class={styles.field}>
          {t(I18N_KEYS.USAGES_DEPTH)}
          <select
            value={walkDepth()}
            onChange={(event) => setWalkDepth(Number(event.currentTarget.value))}
          >
            <For each={depthChoices()}>
              {(depth) => (
                <option value={depth} selected={depth === walkDepth()}>
                  {depth === DEPTH_ALL ? t(I18N_KEYS.USAGES_DEPTH_ALL) : depth}
                </option>
              )}
            </For>
          </select>
        </label>
      </Show>
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
