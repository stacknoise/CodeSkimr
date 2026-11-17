import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { Adder } from "./Adder";
import { addNote } from "./api";
import { NoteCard } from "./Cards";
import { KIND } from "./consts";
import { NoteForm } from "./Forms";
import { report, useRequestedForm } from "./panelState";
import { change } from "./store";
import type { Annotation, Target } from "./types";
import styles from "./Tabs.module.css";

/** The tab "Notes" of the side panel (mock 14): the notes of the symbol and the button "Add note". */
export function NotesTab(props: { project: number; target: Target; notes: Annotation[] }) {
  const [adding, setAdding] = createSignal(false);
  useRequestedForm(KIND.note, () => setAdding(true));
  return (
    <div class={styles.content}>
      <For each={props.notes}>
        {(note) => (
          <NoteCard
            project={props.project}
            note={note}
            onFailed={(failure) => report(failure, null)}
          />
        )}
      </For>
      <Show when={props.notes.length === 0 && !adding()}>
        <p class={styles.empty}>{t(I18N_KEYS.ANNOTATIONS_EMPTY_NOTES)}</p>
      </Show>
      <Adder
        adding={adding()}
        label={t(I18N_KEYS.ANNOTATIONS_ADD_NOTE)}
        onAdd={() => setAdding(true)}
      >
        <NoteForm
          save={(text) => change(props.project, () => addNote(props.project, props.target, text))}
          onClose={() => setAdding(false)}
        />
      </Adder>
    </div>
  );
}
