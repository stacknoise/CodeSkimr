import { createEffect, For, Show } from "solid-js";
import { currentLanguage, t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { treeOutline } from "../explorer/explorer";
import { KindBadge } from "../explorer/KindBadge";
import { tilesOf } from "../explorer/metrics";
import { TileGrid } from "../explorer/TileGrid";
import { PREVIEW_TILES } from "./consts";
import { loadPreview, previewInfo } from "./preview";
import type { Hit } from "./types";
import styles from "./QuickSearch.module.css";

/** An action of the preview: its text and its key. */
interface Action {
  label: MessageKey;
  key: MessageKey;
}

const ACTIONS: readonly Action[] = [
  { label: I18N_KEYS.PALETTE_ACTION_EXPLORER, key: I18N_KEYS.PALETTE_KEY_EXPLORER },
  { label: I18N_KEYS.PALETTE_ACTION_GRAPH, key: I18N_KEYS.PALETTE_KEY_GRAPH },
  { label: I18N_KEYS.PALETTE_ACTION_EDITOR, key: I18N_KEYS.PALETTE_KEY_EDITOR },
  { label: I18N_KEYS.PALETTE_ACTION_COPY, key: I18N_KEYS.PALETTE_KEY_COPY },
];

const tileClasses = {
  grid: styles.previewTiles,
  tile: styles.previewTile,
  value: styles.previewValue,
  label: styles.previewLabel,
  good: styles.good,
  medium: styles.medium,
  poor: styles.poor,
};

/** The documentation and the three numbers of the chosen hit (mock 4), once they are read. */
function Details(props: { id: number }) {
  const info = () => {
    const shown = previewInfo();
    return shown?.id === props.id ? shown : null;
  };
  const tiles = () => {
    const shown = info();
    const all = shown ? tilesOf(shown, treeOutline()?.thresholds ?? null, currentLanguage()) : [];
    return all.filter((tile) => PREVIEW_TILES.includes(tile.id));
  };
  return (
    <Show when={info()}>
      {(shown) => (
        <>
          <Show when={shown().doc}>
            <p class={styles.previewDoc}>{shown().doc}</p>
          </Show>
          <TileGrid tiles={tiles()} classes={tileClasses} />
        </>
      )}
    </Show>
  );
}

/** The right column: the chosen hit and what the keys do with it (mock 4). */
export function Preview(props: { hit: Hit | null }) {
  createEffect(() => {
    const project = activeProject();
    const hit = props.hit;
    if (project !== null && hit !== null) {
      void loadPreview(project, hit.id);
    }
  });
  return (
    <aside class={styles.preview} aria-label={t(I18N_KEYS.PALETTE_PREVIEW)}>
      <h3 class={styles.previewTitle}>{t(I18N_KEYS.PALETTE_PREVIEW)}</h3>
      <Show when={props.hit}>
        {(hit) => (
          <>
            <div class={styles.previewName}>
              <KindBadge kind={hit().kind} />
              <b>{hit().name}</b>
            </div>
            <p class={styles.previewPath}>{hit().qualified_name}</p>
            <p class={styles.previewPath}>{hit().path}</p>
            <Details id={hit().id} />
            <ul class={styles.actions}>
              <For each={ACTIONS}>
                {(action) => (
                  <li class={styles.action}>
                    <span>{t(action.label)}</span>
                    <kbd>{t(action.key)}</kbd>
                  </li>
                )}
              </For>
            </ul>
          </>
        )}
      </Show>
    </aside>
  );
}
