import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  KEY,
  LAYER_CLASS,
  LAYER_TONES,
  LEVEL2_UNAVAILABLE,
  KIND_SEPARATOR,
  MISSING,
} from "./consts";
import { shortName } from "./names";
import type { UsagesAnswer } from "./types";
import styles from "./SidePanel.module.css";

function Tile(props: { value: number; label: string; accent?: boolean }) {
  return (
    <div class={styles.tile}>
      <div class={props.accent ? styles.accent : undefined}>
        {new Intl.NumberFormat(currentLanguage()).format(props.value)}
      </div>
      <span>{props.label}</span>
    </div>
  );
}

function Impact(props: { answer: UsagesAnswer }) {
  const impact = () => props.answer.impact;
  return (
    <section class={styles.block}>
      <h3>{t(I18N_KEYS.USAGES_IMPACT_TITLE)}</h3>
      <p>
        {impact().total === 0
          ? t(I18N_KEYS.USAGES_IMPACT_NONE)
          : t(I18N_KEYS.USAGES_IMPACT_TEXT, {
              count: impact().total,
              layers: impact().layers.length,
              public: impact().public_interfaces,
            })}
      </p>
      <Show when={impact().total > 0}>
        <p>
          {t(I18N_KEYS.USAGES_IMPACT_ACCURACY, {
            verified: impact().verified,
            unchecked: impact().unchecked,
          })}
        </p>
      </Show>
      <div class={styles.chips}>
        <For each={impact().layers}>
          {(share, index) => (
            <span class={styles[`${LAYER_CLASS}${(index() % LAYER_TONES) + 1}`]}>
              {share.layer} {share.count}
            </span>
          )}
        </For>
      </div>
    </section>
  );
}

/** How sure the usages are, and the reason when no indexer checked them. This is what keeps the
 * screen honest: unchecked results are counted apart from verified ones (ADR 0032). */
function Accuracy(props: { answer: UsagesAnswer }) {
  const summary = () => props.answer.summary;
  return (
    <section class={styles.accuracy}>
      <h3>{t(I18N_KEYS.USAGES_ACCURACY_TITLE)}</h3>
      <p>
        {t(I18N_KEYS.USAGES_ACCURACY, {
          verified: summary().verified,
          unchecked: summary().unchecked,
          presumed: summary().probable,
        })}
      </p>
      <Show when={props.answer.level2 === LEVEL2_UNAVAILABLE || summary().unchecked > 0}>
        <p class={styles.warning}>{t(I18N_KEYS.USAGES_LEVEL2_UNAVAILABLE)}</p>
      </Show>
    </section>
  );
}

/** The right column of screen 6: the symbol, the counts, the impact and the accuracy. */
export function SidePanel(props: { answer: UsagesAnswer; onOpen: () => void }) {
  const symbol = () => props.answer.symbol;
  return (
    <aside class={styles.panel} aria-label={t(I18N_KEYS.USAGES_TITLE)}>
      <div>
        <div class={styles.kind}>
          {[symbol().kind, symbol().layer ?? MISSING].join(KIND_SEPARATOR)}
        </div>
        <h2>{shortName(symbol().qualified_name)}</h2>
        <Show when={props.answer.signature}>
          {(signature) => <div class={styles.signature}>{signature()}</div>}
        </Show>
      </div>
      <div class={styles.tiles}>
        <Tile value={props.answer.summary.places} label={t(I18N_KEYS.USAGES_PLACES)} accent />
        <Tile value={props.answer.summary.files} label={t(I18N_KEYS.USAGES_FILES)} />
        <Tile value={props.answer.summary.layers} label={t(I18N_KEYS.USAGES_LAYERS)} />
      </div>
      <Impact answer={props.answer} />
      <section class={styles.block}>
        <h3>{t(I18N_KEYS.USAGES_WORKS_TITLE)}</h3>
        <p>{t(I18N_KEYS.USAGES_WORKS_TEXT)}</p>
      </section>
      <Accuracy answer={props.answer} />
      <button type="button" class={styles.open} onClick={() => props.onOpen()}>
        {t(I18N_KEYS.USAGES_OPEN_EXPLORER)} <kbd>{KEY.open}</kbd>
      </button>
    </aside>
  );
}
