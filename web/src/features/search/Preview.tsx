import { For, Show } from "solid-js";
import type { MessageKey } from "../../shared/i18n/i18n";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { KindBadge } from "../explorer/KindBadge";
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

/** The right column: the chosen hit and what the keys do with it (mock 4). */
export function Preview(props: { hit: Hit | null }) {
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
