import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { LayersHint } from "../layers/LayersHint";
import type { Outline } from "./types";
import styles from "./Overview.module.css";

/** The counts of an outline: layers that hold types, namespaces and types. */
export function counts(outline: Outline) {
  const layers = outline.layers.filter((layer) => layer.namespaces.length > 0);
  const spaces = layers.flatMap((layer) => layer.namespaces);
  return {
    layers: layers.length,
    namespaces: spaces.length,
    types: spaces.reduce((sum, space) => sum + space.types.length, 0),
  };
}

/** Whether any type sits in a layer of the project; the group without a name does not count. */
export function hasLayers(outline: Outline): boolean {
  return outline.layers.some((layer) => layer.name !== "");
}

/** What the middle column shows while no type is selected (blueprint 15.4, initial state). */
export function Overview(props: { outline: Outline | null }) {
  return (
    <Show when={props.outline}>
      {(outline) => (
        <div class={styles.overview}>
          <h2>{t(I18N_KEYS.EXPLORER_OVERVIEW_TITLE)}</h2>
          <p>
            {t(I18N_KEYS.EXPLORER_OVERVIEW_COUNTS, {
              ...counts(outline()),
            })}
          </p>
          <p class={styles.hint}>{t(I18N_KEYS.EXPLORER_OVERVIEW_HINT)}</p>
          <Show when={!hasLayers(outline())}>
            <LayersHint />
          </Show>
        </div>
      )}
    </Show>
  );
}
