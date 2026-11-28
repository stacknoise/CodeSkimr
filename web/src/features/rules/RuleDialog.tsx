import { createEffect, createSignal, type JSX, on, onCleanup, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { useDialogFocus } from "../../shared/ui/dialogFocus";
import { CLOSE_KEY, DIALOG, DIALOG_TITLE_ID, PREVIEW_DELAY_MS } from "./consts";
import { closeDialog, type DialogTarget, previewRules, saveFailed, saveRules } from "./editor";
import { ForbidForm, LayersForm, PublicApiForm, RemoveBody, type Draft } from "./RuleForms";
import type { RuleOut, RulesInput, RulesOut } from "./types";
import styles from "./RuleDialog.module.css";

/** The text under the form: what the last scan would say about the rule as it is now typed. */
function previewText(answer: RulesOut | null, rule: RuleOut | undefined): string {
  if (answer === null) {
    return t(I18N_KEYS.RULES_PREVIEW_INVALID);
  }
  if (!answer.scanned) {
    return t(I18N_KEYS.RULES_PREVIEW_UNSCANNED);
  }
  const found = rule?.violations.length ?? 0;
  if (found === 0) {
    return t(I18N_KEYS.RULES_PREVIEW_NONE);
  }
  return t(I18N_KEYS.RULES_PREVIEW_SOME, { count: found, old: found - (rule?.new ?? 0) });
}

/** Asks for the preview of the typed rule a moment after the last change. */
function usePreview(draft: Draft) {
  const [text, setText] = createSignal(t(I18N_KEYS.RULES_PREVIEW_CHECKING));
  createEffect(
    on(draft.build, (input) => {
      if (input === null || draft.match === undefined) {
        setText(t(I18N_KEYS.RULES_PREVIEW_INVALID));
        return;
      }
      setText(t(I18N_KEYS.RULES_PREVIEW_CHECKING));
      const match = draft.match;
      const timer = setTimeout(
        () =>
          void previewRules(input).then((out) => setText(previewText(out, out?.rules.find(match)))),
        PREVIEW_DELAY_MS,
      );
      onCleanup(() => clearTimeout(timer));
    }),
  );
  return text;
}

/** The title of the dialog for what it changes. */
function titleOf(target: DialogTarget): string {
  const fresh = target.index === null;
  const keys = {
    [DIALOG.forbid]: fresh ? I18N_KEYS.RULES_DIALOG_FORBID_NEW : I18N_KEYS.RULES_DIALOG_FORBID_EDIT,
    [DIALOG.publicApi]: fresh ? I18N_KEYS.RULES_DIALOG_API_NEW : I18N_KEYS.RULES_DIALOG_API_EDIT,
    [DIALOG.layers]: I18N_KEYS.RULES_DIALOG_LAYERS,
    [DIALOG.remove]: I18N_KEYS.RULES_DIALOG_DELETE,
  };
  return t(keys[target.kind]);
}

/** The draft of the form the dialog shows. */
function draftOf(project: number, data: RulesOut, target: DialogTarget): Draft {
  if (target.kind === DIALOG.forbid) {
    return ForbidForm(project, data, target.index);
  }
  if (target.kind === DIALOG.publicApi) {
    return PublicApiForm(project, data, target.index);
  }
  if (target.kind === DIALOG.layers) {
    return LayersForm(project, data);
  }
  return RemoveBody(project, data, target.rule, target.index);
}

/** The failure of the last save, with its code. */
function SaveFailure() {
  return (
    <Show when={saveFailed() !== undefined}>
      <p class={styles.error} role="alert">
        {saveFailed() === null
          ? t(I18N_KEYS.RULES_SAVE_FAILED)
          : errorText(saveFailed() as NonNullable<ReturnType<typeof saveFailed>>)}
      </p>
    </Show>
  );
}

/** The buttons: cancel and save (or delete). */
function Foot(props: { target: DialogTarget; build: () => RulesInput | null }) {
  const [busy, setBusy] = createSignal(false);
  const remove = () => props.target.kind === DIALOG.remove;
  const save = async () => {
    const input = props.build();
    if (input !== null) {
      setBusy(true);
      await saveRules(input);
      setBusy(false);
    }
  };
  return (
    <div class={styles.foot}>
      <span class={styles.promise}>{t(I18N_KEYS.RULES_PROMISE)}</span>
      <button type="button" class={styles.secondary} onClick={closeDialog}>
        {t(I18N_KEYS.RULES_CANCEL)}
      </button>
      <button
        type="button"
        class={styles.primary}
        disabled={busy() || props.build() === null}
        onClick={save}
      >
        {busy()
          ? t(I18N_KEYS.RULES_SAVING)
          : t(remove() ? I18N_KEYS.RULES_DELETE_CONFIRM : I18N_KEYS.RULES_SAVE)}
      </button>
    </div>
  );
}

/** The modal dialog that changes one rule of the project file (mock 41, ADR 0112): the form, what
 * the last scan would say about the rule as typed, and the buttons. Esc closes it. */
export function RuleDialog(props: { project: number; data: RulesOut; target: DialogTarget }) {
  let dialog: HTMLDivElement | undefined;
  const draft = draftOf(props.project, props.data, props.target);
  const preview = usePreview(draft);
  useLayer(() => true);
  useKeydown((event) => event.key === CLOSE_KEY && closeDialog());
  useDialogFocus(() => dialog);
  const shown: () => JSX.Element = () => draft.body;
  return (
    <Portal>
      <div class={styles.scrim} onClick={closeDialog} />
      <div
        ref={dialog}
        class={styles.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby={DIALOG_TITLE_ID}
      >
        <h2 id={DIALOG_TITLE_ID} class={styles.title}>
          {titleOf(props.target)}
        </h2>
        <div class={styles.body}>
          {shown()}
          <Show when={props.target.kind !== DIALOG.remove}>
            <p class={styles.preview} role="status">
              {preview()}
            </p>
          </Show>
          <SaveFailure />
        </div>
        <Foot target={props.target} build={draft.build} />
      </div>
    </Portal>
  );
}
