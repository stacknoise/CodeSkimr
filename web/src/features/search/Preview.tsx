import { For, Show } from "solid-js";
import type { MessageKey } from "../../shared/i18n/i18n";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { KindBadge } from "../explorer/KindBadge";
import type { Hit } from "./types";
import styles from "./QuickSearch.module.css";

/** An action of the preview: its text, its key and whether it works yet. */
interface Action {
  label: MessageKey;
  key: MessageKey;
  ready: boolean;
}

const ACTIONS: readonly Action[] = [
  { label: I18N_KEYS.PALETTE_ACTION_EXPLORER, key: I18N_KEYS.PALETTE_KEY_EXPLORER, ready: true },
  { label: I18N_KEYS.PALETTE_ACTION_GRAPH, key: I18N_KEYS.PALETTE_KEY_GRAPH, ready: true },
  { label: I18N_KEYS.PALETTE_ACTION_EDITOR, key: I18N_KEYS.PALETTE_KEY_EDITOR, ready: true },
  { label: I18N_KEYS.PALETTE_ACTION_COPY, key: I18N_KEYS.PALETTE_KEY_COPY, ready: true },
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
                  <li
                    class={cx(styles.action, !action.ready && styles.off)}
                    aria-disabled={action.ready ? undefined : true}
                    title={action.ready ? undefined : t(I18N_KEYS.PALETTE_ACTION_SOON)}
                  >
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
