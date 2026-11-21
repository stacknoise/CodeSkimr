import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { ICON_BELL, ICON_SIZE_MEDIUM } from "../../shared/ui/consts";
import { Icon } from "../../shared/ui/Icon";
import { BELL_ID, BELL_MAX, BELL_OVER, DRAWER_ID } from "./consts";
import { drawerShown, hideDrawer, openAlertCount, showDrawer } from "./store";
import styles from "./Bell.module.css";

/** Opens the drawer of the project, or closes it. */
function toggle(): void {
  const project = activeProject();
  if (drawerShown()) {
    hideDrawer();
  } else if (project !== null) {
    showDrawer(project);
  }
}

/** The bell of the top bar (mock 15): the number of open alerts, and the button that opens the
 * drawer. Without the commands on the server, or without a project, it is not drawn. */
export function Bell() {
  const count = () => openAlertCount() ?? 0;
  return (
    <Show when={openAlertCount() !== null}>
      <button
        type="button"
        id={BELL_ID}
        class={styles.bell}
        aria-label={
          count() === 0
            ? t(I18N_KEYS.ALERTS_BELL_NONE)
            : t(I18N_KEYS.ALERTS_BELL_LABEL, { count: count() })
        }
        aria-haspopup="dialog"
        aria-expanded={drawerShown()}
        aria-controls={DRAWER_ID}
        onClick={toggle}
      >
        <Icon name={ICON_BELL} size={ICON_SIZE_MEDIUM} />
        <Show when={count() > 0}>
          <b class={styles.count}>{count() > BELL_MAX ? BELL_OVER : count()}</b>
        </Show>
      </button>
    </Show>
  );
}
