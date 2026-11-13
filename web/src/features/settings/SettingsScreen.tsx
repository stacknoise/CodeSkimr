import { For, Show } from "solid-js";
import { currentLanguage, isLanguage, setLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { uiConfig } from "../../shared/settings/ui-config";
import { activeTheme, chosenTheme, followSystem, selectTheme } from "../../shared/theme/theme";
import { THEMES } from "../../shared/theme/tokens";
import { FOLLOW_SYSTEM_ID, LANGUAGE_FIELD_ID, LANGUAGE_NAME_KEYS } from "./consts";
import styles from "./SettingsScreen.module.css";
import { ThemeCard } from "./ThemeCard";
import { LANGUAGES } from "../../shared/i18n/consts";

/** Screen 13: theme with live preview, language, and the editor link (blueprint 15.3). */
export function SettingsScreen() {
  const following = () => chosenTheme() === null;
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.SETTINGS_TITLE)}</h1>
        <p class={styles.sub}>{t(I18N_KEYS.SETTINGS_SUBTITLE)}</p>
      </header>
      <section class={styles.panel}>
        <h2>{t(I18N_KEYS.SETTINGS_THEME_TITLE)}</h2>
        <p class={styles.note}>{t(I18N_KEYS.SETTINGS_THEME_NOTE)}</p>
        <div class={styles.grid}>
          <For each={THEMES}>
            {(theme) => (
              <ThemeCard theme={theme} selected={activeTheme() === theme} onSelect={selectTheme} />
            )}
          </For>
        </div>
        <label class={styles.check} for={FOLLOW_SYSTEM_ID}>
          <input
            id={FOLLOW_SYSTEM_ID}
            type="checkbox"
            checked={following()}
            onChange={(event) =>
              event.currentTarget.checked ? followSystem() : selectTheme(activeTheme())
            }
          />
          <span>
            {t(I18N_KEYS.SETTINGS_THEME_FOLLOW_SYSTEM)}{" "}
            <span class={styles.muted}>{t(I18N_KEYS.SETTINGS_THEME_FOLLOW_SYSTEM_NOTE)}</span>
          </span>
        </label>
      </section>
      <section class={styles.panel}>
        <h2>{t(I18N_KEYS.SETTINGS_LANGUAGE_TITLE)}</h2>
        <select
          id={LANGUAGE_FIELD_ID}
          class={styles.field}
          aria-label={t(I18N_KEYS.SETTINGS_LANGUAGE_TITLE)}
          value={currentLanguage()}
          onChange={(event) =>
            isLanguage(event.currentTarget.value) && setLanguage(event.currentTarget.value)
          }
        >
          <For each={LANGUAGES}>
            {(language) => (
              <option value={language} selected={currentLanguage() === language}>
                {t(LANGUAGE_NAME_KEYS[language])}
              </option>
            )}
          </For>
        </select>
        <p class={styles.note}>{t(I18N_KEYS.SETTINGS_STORAGE_NOTE)}</p>
      </section>
      <Show when={uiConfig()}>
        {(config) => (
          <section class={styles.panel}>
            <h2>{t(I18N_KEYS.SETTINGS_EDITOR_TITLE)}</h2>
            <p class={styles.note}>{t(I18N_KEYS.SETTINGS_EDITOR_NOTE)}</p>
            <dl class={styles.facts}>
              <dt>{t(I18N_KEYS.SETTINGS_EDITOR_TEMPLATE)}</dt>
              <dd class={styles.mono}>{config().editor.url_template}</dd>
              <dt>{t(I18N_KEYS.SETTINGS_EDITOR_SCHEMES)}</dt>
              <dd class={styles.mono}>{config().editor.allowed_schemes.join(", ")}</dd>
            </dl>
          </section>
        )}
      </Show>
    </>
  );
}
