import { Show, type JSX } from "solid-js";
import { SearchField } from "../features/search/SearchField";
import { t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { activeProject } from "../shared/project/active";
import { cx } from "../shared/ui/cx";
import { Icon } from "../shared/ui/Icon";
import { BREADCRUMB_SLASH, CURRENT_PAGE, LOGO_LETTER, RAIL_OWNER, type Route } from "./consts";
import styles from "./App.module.css";
import { currentPath, hrefOf, navigate } from "./router";

/** A button of the rail that cannot be used because no project is open. */
function ShutLink(props: { route: Route }) {
  return (
    <span
      class={cx(styles.railButton, styles.shut)}
      role="link"
      aria-disabled={true}
      aria-label={t(props.route.label)}
      title={t(I18N_KEYS.NAV_NEEDS_PROJECT, { name: t(props.route.label) })}
    >
      <Icon name={props.route.icon} />
    </span>
  );
}

/** One button of the rail: it shows its screen without loading the page again. */
export function RailLink(props: { route: Route }) {
  const active = () => (RAIL_OWNER[currentPath()] ?? currentPath()) === props.route.path;
  const shut = () => props.route.needsProject === true && activeProject() === null;
  return (
    <Show when={!shut()} fallback={<ShutLink route={props.route} />}>
      <a
        href={hrefOf(props.route.path)}
        class={cx(styles.railButton, active() && styles.on)}
        aria-label={t(props.route.label)}
        title={t(props.route.label)}
        aria-current={active() ? CURRENT_PAGE : undefined}
        onClick={(event) => {
          event.preventDefault();
          navigate(props.route.path);
        }}
      >
        <Icon name={props.route.icon} />
      </a>
    </Show>
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
