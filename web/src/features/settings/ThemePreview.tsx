import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ThemeName } from "../../shared/theme/tokens";
import { THEME_ATTRIBUTE } from "../../shared/theme/consts";
import { PREVIEW_NEW_FILES, PREVIEW_OTHER_PROJECT } from "./consts";
import styles from "./ThemePreview.module.css";

/** A miniature of the real UI in the colours of one theme: top bar, rail, list, badges and a
 * button. The container carries that theme, so the same variables and components as everywhere
 * else draw it (blueprint 15.3). */
export function ThemePreview(props: { theme: ThemeName }) {
  return (
    <div class={styles.preview} {...{ [THEME_ATTRIBUTE]: props.theme }} aria-hidden={true}>
      <div class={styles.bar}>
        <span class={styles.logo} />
        <b>{t(I18N_KEYS.APP_NAME)}</b>
        <span class={styles.shortcut}>{t(I18N_KEYS.APP_SEARCH_SHORTCUT)}</span>
      </div>
      <div class={styles.middle}>
        <div class={styles.rail}>
          <i class={styles.active} />
          <i />
          <i />
        </div>
        <div class={styles.content}>
          <div class={styles.heading}>{t(I18N_KEYS.PREVIEW_PROJECTS)}</div>
          <div class={cx(styles.line, styles.selected)}>
            <b>{t(I18N_KEYS.PREVIEW_SAMPLE_PROJECT)}</b>
            <Pill tone={TONE.good} compact>
              {t(I18N_KEYS.PREVIEW_BADGE_CURRENT)}
            </Pill>
          </div>
          <div class={styles.line}>
            <span>{PREVIEW_OTHER_PROJECT}</span>
            <Pill tone={TONE.warn} compact>
              {t(I18N_KEYS.PREVIEW_BADGE_NEW, { count: PREVIEW_NEW_FILES })}
            </Pill>
          </div>
          <div>
            <span class={styles.button}>{t(I18N_KEYS.PREVIEW_ADD)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
