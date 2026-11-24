import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import styles from "./ProjectTable.module.css";

/** A button of the row that does not select the row. */
function RowButton(props: {
  label: string;
  text: string;
  onClick: () => void;
  danger?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      class={cx(styles.action, props.danger === true && styles.danger)}
      aria-label={props.label}
      disabled={props.disabled}
      onClick={(event) => {
        event.stopPropagation();
        props.onClick();
      }}
    >
      {props.text}
    </button>
  );
}

/** The second step: the question and the two answers. */
function Confirm(props: { name: string; busy: boolean; onYes: () => void; onNo: () => void }) {
  return (
    <span class={styles.ask} role="group" aria-label={t(I18N_KEYS.PROJECTS_REMOVE_ASK)}>
      <span class={styles.question}>
        {t(I18N_KEYS.PROJECTS_REMOVE_QUESTION, { name: props.name })}
      </span>
      <RowButton
        label={t(I18N_KEYS.PROJECTS_REMOVE_CONFIRM)}
        text={t(I18N_KEYS.PROJECTS_REMOVE_CONFIRM)}
        danger
        disabled={props.busy}
        onClick={props.onYes}
      />
      <RowButton
        label={t(I18N_KEYS.PROJECTS_REMOVE_CANCEL)}
        text={t(I18N_KEYS.PROJECTS_REMOVE_CANCEL)}
        disabled={props.busy}
        onClick={props.onNo}
      />
    </span>
  );
}

/** The project whose question is open. It lives outside the row: while a scan runs, the list is
 * read again and the row is built anew, which must not close the question. */
const [askingFor, setAskingFor] = createSignal<number | null>(null);
/** The projects that are being removed; also outside the row, for the same reason. */
const [removing, setRemoving] = createSignal<ReadonlySet<number>>(new Set());

/** Closes the open question, if there is one. */
export function closeRemoveQuestion(): void {
  setAskingFor(null);
}

/** "Remove" of a project row, with a second step: the first click asks, the second one removes.
 * Removing deletes what the app stored about the project, never its files. */
export function RemoveAction(props: {
  id: number;
  name: string;
  onRemove: () => Promise<boolean>;
}) {
  const asking = () => askingFor() === props.id;
  const busy = () => removing().has(props.id);
  const remove = async () => {
    const id = props.id;
    setRemoving((previous) => new Set(previous).add(id));
    try {
      await props.onRemove();
    } finally {
      setRemoving((previous) => {
        const next = new Set(previous);
        next.delete(id);
        return next;
      });
      // Only the own question: the user may have opened the one of another project meanwhile.
      setAskingFor((open) => (open === id ? null : open));
    }
  };
  return (
    <Show
      when={asking()}
      fallback={
        <RowButton
          label={t(I18N_KEYS.PROJECTS_REMOVE, { name: props.name })}
          text={t(I18N_KEYS.PROJECTS_REMOVE_BUTTON)}
          onClick={() => setAskingFor(props.id)}
        />
      }
    >
      <Confirm
        name={props.name}
        busy={busy()}
        onYes={() => void remove()}
        onNo={() => closeRemoveQuestion()}
      />
    </Show>
  );
}
