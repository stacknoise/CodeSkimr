import { For, Match, Show, Switch } from "solid-js";
import { currentLanguage, isLanguage, setLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { uiConfig, type UiConfig } from "../../shared/settings/ui-config";
import { activeTheme, chosenTheme, followSystem, selectTheme } from "../../shared/theme/theme";
import { THEMES } from "../../shared/theme/tokens";
import { moveAlongTabs } from "../../shared/keys/tabs";
import { cx } from "../../shared/ui/cx";
import {
  FOLLOW_SYSTEM_ID,
  LANGUAGE_FIELD_ID,
  LANGUAGE_NAME_KEYS,
  SETTINGS_PANEL_ID,
  SETTINGS_SEARCH_ID,
  SETTINGS_TAB,
  SETTINGS_TAB_ID_PREFIX,
  SETTINGS_TAB_STOP,
  SETTINGS_TABS,
} from "./consts";
import styles from "./SettingsScreen.module.css";
import { CredentialsPanel } from "./CredentialsPanel";
import { ConfigSourcesPanel, EffectivePanel } from "./EffectivePanel";
import { effectiveSearch } from "./effective";
import { searchSettings, settingsTab, showSettingsTab } from "./tab";
import { IndexerMemoryPanel } from "./IndexerMemoryPanel";
import { IndexersPanel } from "./IndexersPanel";
import { ThemeCard } from "./ThemeCard";
import { LANGUAGES } from "../../shared/i18n/consts";
import { CODES_ROUTE, LAYERS_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { statusInfo } from "../status/status";

/** The theme cards and the follow-system switch. */
function ThemePanel() {
  const following = () => chosenTheme() === null;
  return (
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
  );
}

/** The interface language choice. */
function LanguagePanel() {
  return (
    <section class={`${styles.panel} ${styles.narrow}`}>
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
  );
}

/** The way to the layer editor. */
function LayersPanel() {
  return (
    <section class={`${styles.panel} ${styles.narrow}`}>
      <h2>{t(I18N_KEYS.SETTINGS_LAYERS_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_LAYERS_NOTE)}</p>
      <button type="button" class={styles.field} onClick={() => navigate(LAYERS_ROUTE.path)}>
        {t(I18N_KEYS.SETTINGS_LAYERS_ACTION)}
      </button>
    </section>
  );
}

/** The way to the list of the error codes. */
function CodesPanel() {
  return (
    <section class={`${styles.panel} ${styles.narrow}`}>
      <h2>{t(I18N_KEYS.SETTINGS_CODES_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_CODES_NOTE)}</p>
      <button type="button" class={styles.field} onClick={() => navigate(CODES_ROUTE.path)}>
        {t(I18N_KEYS.SETTINGS_CODES_ACTION)}
      </button>
    </section>
  );
}

/** The read-only editor link settings. */
function EditorPanel(props: { config: UiConfig }) {
  return (
    <section class={`${styles.panel} ${styles.narrow}`}>
      <h2>{t(I18N_KEYS.SETTINGS_EDITOR_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_EDITOR_NOTE)}</p>
      <dl class={styles.facts}>
        <dt>{t(I18N_KEYS.SETTINGS_EDITOR_TEMPLATE)}</dt>
        <dd class={styles.mono}>{props.config.editor.url_template}</dd>
        <dt>{t(I18N_KEYS.SETTINGS_EDITOR_SCHEMES)}</dt>
        <dd class={styles.mono}>{props.config.editor.allowed_schemes.join(", ")}</dd>
      </dl>
    </section>
  );
}

const TAB_IDS = SETTINGS_TABS.map((tab) => tab.id);

/** The tab bar with the search field at its end. */
function TabBar() {
  return (
    <div class={styles.bar}>
      <div
        class={styles.tabs}
        role="tablist"
        aria-label={t(I18N_KEYS.SETTINGS_TABS_LABEL)}
        onKeyDown={(event) =>
          moveAlongTabs(event, TAB_IDS, settingsTab(), showSettingsTab, SETTINGS_TAB_ID_PREFIX)
        }
      >
        <For each={SETTINGS_TABS}>
          {(tab) => (
            <button
              type="button"
              role="tab"
              id={`${SETTINGS_TAB_ID_PREFIX}${tab.id}`}
              aria-controls={SETTINGS_PANEL_ID}
              aria-selected={settingsTab() === tab.id}
              tabindex={settingsTab() === tab.id ? SETTINGS_TAB_STOP.on : SETTINGS_TAB_STOP.off}
              class={cx(styles.tab, settingsTab() === tab.id && styles.on)}
              onClick={() => showSettingsTab(tab.id)}
            >
              {t(tab.label)}
            </button>
          )}
        </For>
      </div>
      <input
        id={SETTINGS_SEARCH_ID}
        type="search"
        class={styles.search}
        placeholder={t(I18N_KEYS.SETTINGS_SEARCH)}
        aria-label={t(I18N_KEYS.SETTINGS_SEARCH)}
        value={effectiveSearch()}
        onInput={(event) => searchSettings(event.currentTarget.value)}
      />
    </div>
  );
}

/** The content of the tab that is shown. */
function TabPage() {
  return (
    <Switch>
      <Match when={settingsTab() === SETTINGS_TAB.appearance}>
        <ThemePanel />
        <LanguagePanel />
      </Match>
      <Match when={settingsTab() === SETTINGS_TAB.analysis}>
        <LayersPanel />
        <IndexersPanel />
        <IndexerMemoryPanel />
      </Match>
      <Match when={settingsTab() === SETTINGS_TAB.access}>
        <CredentialsPanel />
        <Show when={uiConfig()}>{(config) => <EditorPanel config={config()} />}</Show>
      </Match>
      <Match when={settingsTab() === SETTINGS_TAB.effective}>
        <EffectivePanel />
      </Match>
      <Match when={settingsTab() === SETTINGS_TAB.info}>
        <ConfigSourcesPanel />
        <CodesPanel />
      </Match>
    </Switch>
  );
}

/** Screen 13: the settings in five tabs (ADR 0098): appearance (theme with live preview,
 * language), analysis (layers, tools for stage 2), access (Git logins, editor link), the effective
 * settings and information (configuration file, error codes; the version is in the header) (blueprint 15.3). */
export function SettingsScreen() {
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.SETTINGS_TITLE)}</h1>
        <Show when={statusInfo()?.version}>
          {(version) => (
            <p class={styles.sub}>{t(I18N_KEYS.SETTINGS_VERSION, { version: version() })}</p>
          )}
        </Show>
      </header>
      <TabBar />
      <div
        role="tabpanel"
        id={SETTINGS_PANEL_ID}
        class={styles.page}
        aria-labelledby={`${SETTINGS_TAB_ID_PREFIX}${settingsTab()}`}
      >
        <TabPage />
      </div>
    </>
  );
}
