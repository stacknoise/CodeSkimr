import { For } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { GRAPH_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { currentType } from "../explorer/explorer";
import { LEVEL } from "../graph/consts";
import { setGraphFocus, setGraphLevel } from "../graph/graph";
import type { Level } from "../graph/types";
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

/** The levels of the mock: the symbol level is this screen; the other two open the dependency
 * map on that level. The type level needs a type chosen in the explorer. */
function Levels() {
  const pick = t(I18N_KEYS.GRAPH_TYPES_PICK);
  const typed = () => currentType() !== null;
  return (
    <div class={styles.group} role={ROLE.group} aria-label={t(I18N_KEYS.USAGES_LEVEL_LABEL)}>
      <button type="button" onClick={() => openMap(LEVEL.namespaces)}>
        {t(I18N_KEYS.USAGES_LEVEL_NAMESPACES)}
      </button>
      <button
        type="button"
        aria-disabled={!typed()}
        title={typed() ? undefined : pick}
        onClick={() => typed() && openMap(LEVEL.types)}
      >
        {t(I18N_KEYS.USAGES_LEVEL_TYPES)}
      </button>
      <button type="button" class={styles.current} aria-pressed={true}>
        {t(I18N_KEYS.USAGES_LEVEL_SYMBOL)}
      </button>
    </div>
  );
}

/** Opens the dependency map on `level`; the type level puts the explorer's type in the middle. */
function openMap(level: Level) {
  const type = currentType();
  if (level === LEVEL.types && type !== null) {
    setGraphFocus(type);
  }
  setGraphLevel(level);
  navigate(GRAPH_ROUTE.path);
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
