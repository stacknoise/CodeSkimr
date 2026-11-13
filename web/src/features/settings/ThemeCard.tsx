import { For } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { THEME_ATTRIBUTE } from "../../shared/theme/consts";
import type { ThemeName } from "../../shared/theme/tokens";
import { SWATCH_TOKENS, THEME_TEXTS } from "./consts";
import styles from "./ThemeCard.module.css";
import { ThemePreview } from "./ThemePreview";

/** One theme with its live preview. Choosing the card applies the theme at once. */
export function ThemeCard(props: {
  theme: ThemeName;
  selected: boolean;
  onSelect: (theme: ThemeName) => void;
}) {
  const texts = () => THEME_TEXTS[props.theme];
  return (
    <div class={cx(styles.card, props.selected && styles.on)}>
      <ThemePreview theme={props.theme} />
      <div class={styles.row}>
        <div>
          <div class={styles.name}>{t(texts().name)}</div>
          <div class={styles.note}>{t(texts().note)}</div>
        </div>
        {props.selected ? (
          <Pill tone={TONE.accent}>{t(I18N_KEYS.SETTINGS_THEME_SELECTED)}</Pill>
        ) : (
          <button type="button" class={styles.select} onClick={() => props.onSelect(props.theme)}>
            {t(I18N_KEYS.SETTINGS_THEME_SELECT)}
          </button>
        )}
      </div>
      <div class={styles.swatches} {...{ [THEME_ATTRIBUTE]: props.theme }} aria-hidden={true}>
        <For each={SWATCH_TOKENS}>{(token) => <i class={styles[token]} />}</For>
      </div>
    </div>
  );
}
