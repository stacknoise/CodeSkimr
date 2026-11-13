import { Show, type JSX } from "solid-js";
import { SearchField } from "../features/search/SearchField";
import { t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { cx } from "../shared/ui/cx";
import { Icon } from "../shared/ui/Icon";
import { BREADCRUMB_SLASH, CURRENT_PAGE, LOGO_LETTER, type Route } from "./consts";
import styles from "./App.module.css";
import { currentPath, hrefOf, navigate } from "./router";

/** One button of the rail: it shows its screen without loading the page again. */
export function RailLink(props: { route: Route }) {
  return (
    <a
      href={hrefOf(props.route.path)}
      class={cx(styles.railButton, currentPath() === props.route.path && styles.on)}
      aria-label={t(props.route.label)}
      aria-current={currentPath() === props.route.path ? CURRENT_PAGE : undefined}
      onClick={(event) => {
        event.preventDefault();
        navigate(props.route.path);
      }}
    >
      <Icon name={props.route.icon} />
    </a>
  );
}

/** The top bar of every screen: name, what the screen belongs to (`children`, see `Crumb`) and
 * the quick search (blueprint 15.2). */
export function TopBar(props: { children?: JSX.Element }) {
  return (
    <header class={styles.top}>
      <div class={styles.brand}>
        <span class={styles.logo}>{LOGO_LETTER}</span>
        <span class={styles.name}>{t(I18N_KEYS.APP_NAME)}</span>
      </div>
      {props.children}
      <SearchField />
    </header>
  );
}

/** The project in the top bar: a slash and the name, a link to the projects when `href` is
 * given. */
export function Crumb(props: { name: string; href?: string }) {
  return (
    <>
      <span class={styles.slash} aria-hidden={true}>
        {BREADCRUMB_SLASH}
      </span>
      <Show when={props.href} fallback={<b class={styles.crumb}>{props.name}</b>}>
        {(href) => (
          <a
            class={styles.crumb}
            href={hrefOf(href())}
            onClick={(event) => {
              event.preventDefault();
              navigate(href());
            }}
          >
            {props.name}
          </a>
        )}
      </Show>
    </>
  );
}
