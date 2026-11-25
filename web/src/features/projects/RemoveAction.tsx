import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { REMOVE_ICON } from "./consts";
import { markRemoving } from "./removing";
import { Icon, type IconName } from "../../shared/ui/Icon";
import { RemoveDialog } from "./RemoveDialog";
import type { ProjectRow } from "./types";
import styles from "./ProjectTable.module.css";

/** A button of the row that does not select the row. */
function RowButton(props: {
  label: string;
  text: string;
  onClick: () => void;
  danger?: boolean;
  disabled?: boolean;
  /** An icon instead of the text; the text stays as the tooltip. */
  icon?: IconName;
}) {
  return (
    <button
      type="button"
      class={cx(
        props.icon === undefined ? styles.action : styles.rescan,
        props.danger === true && styles.danger,
      )}
      aria-label={props.label}
      title={props.text}
      disabled={props.disabled}
      onClick={(event) => {
        event.stopPropagation();
        props.onClick();
      }}
    >
      {props.icon === undefined ? props.text : <Icon name={props.icon} />}
    </button>
  );
}

/** The project whose question is open. It lives outside the row: while a scan runs, the list is
 * read again and the row is built anew, which must not close the question. */
const [askingFor, setAskingFor] = createSignal<number | null>(null);

/** Whether a question before a removal is open. */
export const removeQuestionOpen = () => askingFor() !== null;

/** Closes the open question, if there is one. */
export function closeRemoveQuestion(): void {
  setAskingFor(null);
}

/** Removes a project and leaves the state of the row as it found it: the answers are locked
 * meanwhile, and afterwards only the own question is closed. */
async function removeProject(id: number, onRemove: () => Promise<boolean>): Promise<void> {
  markRemoving(id, true);
  // The row shows "removing" now; the question is not needed any more.
  setAskingFor((open) => (open === id ? null : open));
  try {
    await onRemove();
  } finally {
    markRemoving(id, false);
    // The user may have opened the question of another project meanwhile.
    setAskingFor((open) => (open === id ? null : open));
  }
}

/** "Remove" of a project: the button opens the dialog (mock 24, D1) that names the consequences;
 * its red button removes. Removing deletes what the app stored about the project, never its files. */
export function RemoveAction(props: { row: ProjectRow; onRemove: () => Promise<boolean> }) {
  const asking = () => askingFor() === props.row.id;
  const remove = () => removeProject(props.row.id, props.onRemove);
  return (
    <>
      <RowButton
        label={t(I18N_KEYS.PROJECTS_REMOVE, { name: props.row.name })}
        text={t(I18N_KEYS.PROJECTS_REMOVE_BUTTON)}
        icon={REMOVE_ICON}
        onClick={() => setAskingFor(props.row.id)}
      />
      <Show when={asking()}>
        <RemoveDialog row={props.row} onYes={() => void remove()} onNo={closeRemoveQuestion} />
      </Show>
    </>
  );
}
