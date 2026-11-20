import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  folderDialogAvailable,
  folderDialogWaiting,
  giveUpWaiting,
  pickFolder,
} from "./folderDialog";
import styles from "./NewProjectForm.module.css";

/** The button "Browse" next to the folder field (ADR 0075): it opens the folder dialog of the
 * operating system on the machine of the app and puts the chosen path into the field. It is only
 * there if the app has such a dialog; the path can always be typed. */
export function BrowseButton(props: { onPicked: (path: string) => void }) {
  const browse = async () => {
    const path = await pickFolder();
    if (path !== null) {
      props.onPicked(path);
    }
  };
  return (
    <Show when={folderDialogAvailable()}>
      <button type="button" class={styles.browse} disabled={folderDialogWaiting()} onClick={browse}>
        {t(I18N_KEYS.PROJECTS_BROWSE)}
      </button>
    </Show>
  );
}

/** What the form says while the dialog of the system is open. */
export function BrowseWaiting() {
  return (
    <Show when={folderDialogWaiting()}>
      <div class={styles.waiting} role="status">
        <strong>{t(I18N_KEYS.PROJECTS_BROWSE_WAITING_TITLE)}</strong>
        <p>{t(I18N_KEYS.PROJECTS_BROWSE_WAITING_TEXT)}</p>
        <button type="button" class={styles.browse} onClick={giveUpWaiting}>
          {t(I18N_KEYS.PROJECTS_BROWSE_GIVE_UP)}
        </button>
      </div>
    </Show>
  );
}

/** The path the dialog chose goes into the field, with a note to check it; typing into the field
 * takes the note away again. */
export function useBrowse(props: { source: string; onSource: (value: string) => void }) {
  const [taken, setTaken] = createSignal(false);
  return {
    taken: () => taken() && props.source.trim() !== "",
    picked: (path: string) => {
      props.onSource(path);
      setTaken(true);
    },
    typed: (value: string) => {
      setTaken(false);
      props.onSource(value);
    },
  };
}
