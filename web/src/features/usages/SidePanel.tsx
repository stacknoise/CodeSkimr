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
import { Breakable } from "../../shared/ui/Breakable";
import { accuracyNote } from "./accuracyNote";
import { shortName } from "./names";
import type { UsagesAnswer } from "./types";
import { panelStyle } from "../../shared/layout/panelStyle";
import { panelWidth } from "./layout";
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

/** The documentation comment of the symbol, left out when the code has none. */
function Description(props: { doc: string | null }) {
  return (
    <Show when={props.doc}>
      {(doc) => (
        <section class={styles.block}>
          <h3>{t(I18N_KEYS.USAGES_DESCRIPTION_TITLE)}</h3>
          <p class={styles.description}>{doc()}</p>
        </section>
      )}
    </Show>
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

/** How sure the usages are, and the reason when level 2 did not check them. This is what keeps the
 * screen honest: unchecked results are counted apart from verified ones (ADR 0033). */
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
      <Show when={accuracyNote(props.answer)}>{(note) => <p>{note()}</p>}</Show>
      <Show when={props.answer.level2 === LEVEL2_UNAVAILABLE}>
        <p class={styles.warning}>{t(I18N_KEYS.USAGES_LEVEL2_UNAVAILABLE)}</p>
      </Show>
      <Show when={props.answer.level2 !== LEVEL2_UNAVAILABLE && summary().unchecked > 0}>
        <p class={styles.warning}>{t(I18N_KEYS.USAGES_LEVEL2_PARTIAL)}</p>
      </Show>
    </section>
  );
}

/** The right column of screen 6: the symbol, the counts, the impact and the accuracy. */
export function SidePanel(props: { answer: UsagesAnswer; onOpen: () => void }) {
  const symbol = () => props.answer.symbol;
  return (
    <aside
      class={styles.panel}
      style={panelStyle(panelWidth)}
      aria-label={t(I18N_KEYS.USAGES_TITLE)}
    >
      <div>
        <div class={styles.kind}>
          {[symbol().kind, symbol().layer ?? MISSING].join(KIND_SEPARATOR)}
        </div>
        <h2 aria-label={shortName(symbol().qualified_name)}>
          <Breakable text={shortName(symbol().qualified_name)} />
        </h2>
        <Show when={props.answer.signature}>
          {(signature) => <div class={styles.signature}>{signature()}</div>}
        </Show>
      </div>
      <div class={styles.tiles}>
        <Tile value={props.answer.summary.places} label={t(I18N_KEYS.USAGES_PLACES)} accent />
        <Tile value={props.answer.summary.files} label={t(I18N_KEYS.USAGES_FILES)} />
        <Tile value={props.answer.summary.layers} label={t(I18N_KEYS.USAGES_LAYERS)} />
      </div>
      <Description doc={props.answer.doc} />
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
