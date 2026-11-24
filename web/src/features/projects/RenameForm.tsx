import { Show } from "solid-js";
import { Icon } from "../../shared/ui/Icon";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { RENAME_ICON, RENAME_ICON_SIZE } from "./consts";
import { cancelRename, renamingProject, saveRename, startRename, typeRename } from "./projects";
import styles from "./ProjectTable.module.css";

/** The name of a project row with its rename button. While the name is edited the input, "Save" and
 * "Cancel" take its place; the typed text lives in `projects.ts`, so a new read of the list does
 * not lose it. */
export function RenameField(props: { id: number; name: string }) {
  const editing = () => renamingProject()?.id === props.id;
  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    void saveRename();
  };
  return (
    <Show when={editing()}>
      <form class={styles.rename} onSubmit={submit} onClick={(event) => event.stopPropagation()}>
        <input
          class={styles.renameInput}
          aria-label={t(I18N_KEYS.PROJECTS_RENAME_FIELD, { name: props.name })}
          value={renamingProject()?.text ?? ""}
          onInput={(event) => typeRename(event.currentTarget.value)}
          onKeyDown={(event) => event.key === "Escape" && cancelRename()}
          ref={(input) => queueMicrotask(() => input.focus())}
          spellcheck={false}
          autocomplete="off"
        />
        <button type="submit" class={styles.action}>
          {t(I18N_KEYS.PROJECTS_RENAME_SAVE)}
        </button>
        <button type="button" class={styles.action} onClick={cancelRename}>
          {t(I18N_KEYS.PROJECTS_RENAME_CANCEL)}
        </button>
      </form>
    </Show>
  );
}

/** The button that opens the name edit. */
export function RenameButton(props: { id: number; name: string }) {
  return (
    <Show when={renamingProject()?.id !== props.id}>
      <button
        type="button"
        class={styles.pencil}
        aria-label={t(I18N_KEYS.PROJECTS_RENAME, { name: props.name })}
        title={t(I18N_KEYS.PROJECTS_RENAME_BUTTON)}
        onClick={(event) => {
          event.stopPropagation();
          startRename(props.id, props.name);
        }}
      >
        <Icon name={RENAME_ICON} size={RENAME_ICON_SIZE} />
      </button>
    </Show>
  );
}
