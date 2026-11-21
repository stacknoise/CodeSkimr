import { Show } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { ICON_DOWNLOAD, ICON_SIZE_MEDIUM } from "../../shared/ui/consts";
import { Icon } from "../../shared/ui/Icon";
import type { ProjectRow } from "../projects/types";
import styles from "./ExportButton.module.css";
import { exportProject, isExporting, lastExportFailure } from "./exportHtml";

/** "Export as HTML" of the top bar: writes the newest scan of the open project into one page and
 * downloads it. It needs a scan; without one the button is shut and says why. */
export function ExportButton(props: { project: ProjectRow }) {
  const scanned = () => props.project.scan !== null;
  const title = () =>
    scanned() ? t(I18N_KEYS.EXPORT_BUTTON_TITLE) : t(I18N_KEYS.EXPORT_BUTTON_NEEDS_SCAN);
  return (
    <>
      <button
        type="button"
        class={styles.button}
        title={title()}
        aria-label={t(I18N_KEYS.EXPORT_BUTTON_LABEL)}
        disabled={!scanned() || isExporting()}
        onClick={() => void exportProject(props.project.id, props.project.name)}
      >
        <Icon name={ICON_DOWNLOAD} size={ICON_SIZE_MEDIUM} />
        {isExporting() ? t(I18N_KEYS.EXPORT_BUTTON_BUSY) : t(I18N_KEYS.EXPORT_BUTTON_LABEL)}
      </button>
      <Show when={lastExportFailure()}>
        {(failure) => (
          <span class={styles.failure} role="alert">
            {failure().code === null
              ? t(I18N_KEYS.EXPORT_BUTTON_FAILED)
              : errorText(failure().code!)}
          </span>
        )}
      </Show>
    </>
  );
}
