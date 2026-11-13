import { Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { metricValue } from "./format";
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

/** What the middle column shows while no type is selected (blueprint 15.4, initial state). */
export function Overview(props: { outline: Outline | null }) {
  return (
    <Show when={props.outline}>
      {(outline) => (
        <div class={styles.overview}>
          <h2>{t(I18N_KEYS.EXPLORER_OVERVIEW_TITLE)}</h2>
          <p>
            {t(I18N_KEYS.EXPLORER_OVERVIEW_COUNTS, {
              layers: metricValue(counts(outline()).layers, currentLanguage()),
              namespaces: metricValue(counts(outline()).namespaces, currentLanguage()),
              types: metricValue(counts(outline()).types, currentLanguage()),
            })}
          </p>
          <p class={styles.hint}>{t(I18N_KEYS.EXPLORER_OVERVIEW_HINT)}</p>
        </div>
      )}
    </Show>
  );
}
