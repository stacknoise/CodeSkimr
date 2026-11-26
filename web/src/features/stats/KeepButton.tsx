import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { keepSymbol } from "./keep";
import styles from "./KeepButton.module.css";
import type { Row } from "./types";

/** The form that asks for the reason and writes the symbol into the exception list. */
function ReasonForm(props: { row: Row; project: number; onDone: () => void }) {
  const [reason, setReason] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const save = async (event: SubmitEvent) => {
    event.preventDefault();
    setBusy(true);
    const done = await keepSymbol(props.project, props.row, reason());
    setBusy(false);
    if (done) {
      props.onDone();
    }
  };
  return (
    <form class={styles.form} onSubmit={save}>
      <input
        type="text"
        required
        autofocus
        value={reason()}
        aria-label={t(I18N_KEYS.STATS_KEEP_REASON_LABEL)}
        placeholder={t(I18N_KEYS.STATS_KEEP_REASON_PLACEHOLDER)}
        onInput={(event) => setReason(event.currentTarget.value)}
      />
      <button type="submit" disabled={busy() || reason().trim() === ""}>
        {t(I18N_KEYS.STATS_KEEP_SAVE)}
      </button>
      <button type="button" onClick={props.onDone}>
        {t(I18N_KEYS.STATS_KEEP_CANCEL)}
      </button>
    </form>
  );
}

/** The cell of a row without usages with the button "Keep on purpose": it asks for a reason and
 * writes the symbol into the exception list of the project file (ADR 0101). A Git project has no
 * such file, so the button is off there. */
export function KeepButton(props: { row: Row; project: number; editable: boolean }) {
  const [open, setOpen] = createSignal(false);
  return (
    <td class={styles.cell} onClick={(event) => event.stopPropagation()}>
      <Show
        when={open()}
        fallback={
          <button
            type="button"
            disabled={!props.editable}
            title={props.editable ? undefined : t(I18N_KEYS.STATS_KEEP_GIT)}
            aria-label={t(I18N_KEYS.STATS_KEEP_ROW, { name: props.row.name })}
            onClick={() => setOpen(true)}
          >
            {t(I18N_KEYS.STATS_KEEP_BUTTON)}
          </button>
        }
      >
        <ReasonForm row={props.row} project={props.project} onDone={() => setOpen(false)} />
      </Show>
    </td>
  );
}
