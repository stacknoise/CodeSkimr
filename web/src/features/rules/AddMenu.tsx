import { createSignal, For, Show } from "solid-js";
import { type MessageKey, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { CLOSE_KEY, DIALOG, KIND } from "./consts";
import { inputOf } from "./edit";
import { openDialog, saveRules } from "./editor";
import type { RulesOut } from "./types";
import styles from "./RulesScreen.module.css";

/** One entry of the menu: its text, its hint and what it does. */
interface Entry {
  title: MessageKey;
  hint: MessageKey;
  off: boolean;
  run: () => void;
}

/** The entries of the menu "Add rule" (mock 41): the three kinds with a dialog and the switch
 * for the cycles, which is off once it is on. */
function entriesOf(project: number, data: RulesOut): Entry[] {
  const dialog = (kind: (typeof DIALOG)[keyof typeof DIALOG]) => () =>
    void openDialog(project, { kind, index: null, rule: null });
  const cycles = data.rules.some((rule) => rule.kind === KIND.cycles);
  return [
    {
      title: I18N_KEYS.RULES_ADD_FORBID,
      hint: I18N_KEYS.RULES_ADD_FORBID_HINT,
      off: false,
      run: dialog(DIALOG.forbid),
    },
    {
      title: I18N_KEYS.RULES_ADD_PUBLIC_API,
      hint: I18N_KEYS.RULES_ADD_PUBLIC_API_HINT,
      off: false,
      run: dialog(DIALOG.publicApi),
    },
    {
      title: I18N_KEYS.RULES_ADD_CYCLES,
      hint: I18N_KEYS.RULES_ADD_CYCLES_HINT,
      off: cycles,
      run: () => void saveRules({ ...inputOf(project, data), cycles: true }),
    },
    {
      title: I18N_KEYS.RULES_ADD_LAYERS,
      hint: I18N_KEYS.RULES_ADD_LAYERS_HINT,
      off: false,
      run: dialog(DIALOG.layers),
    },
  ];
}

/** The entries of the open menu. */
function MenuList(props: { entries: Entry[]; onPick: (entry: Entry) => void }) {
  return (
    <div class={styles.menu} role="menu">
      <For each={props.entries}>
        {(entry) => (
          <button
            type="button"
            role="menuitem"
            class={styles.menuItem}
            disabled={entry.off}
            onClick={() => props.onPick(entry)}
          >
            <strong>{t(entry.title)}</strong>
            <small>{t(entry.hint)}</small>
          </button>
        )}
      </For>
    </div>
  );
}

/** The button "Add rule" and its menu. It is off for a Git project, whose project file the rules
 * cannot be written into, and before the rules are read. */
export function AddMenu(props: { project: number; data: RulesOut | null }) {
  const [open, setOpen] = createSignal(false);
  useKeydown((event) => event.key === CLOSE_KEY && setOpen(false));
  const pick = (entry: Entry) => {
    setOpen(false);
    entry.run();
  };
  return (
    <span class={styles.menuHost}>
      <button
        type="button"
        class={styles.primary}
        aria-haspopup="menu"
        aria-expanded={open()}
        disabled={props.data?.holdable !== true}
        title={props.data?.holdable === false ? t(I18N_KEYS.RULES_ADD_GIT) : undefined}
        onClick={() => setOpen(!open())}
      >
        {t(I18N_KEYS.RULES_ADD)}
      </button>
      <Show when={open() && props.data}>
        {(data) => <MenuList entries={entriesOf(props.project, data())} onPick={pick} />}
      </Show>
    </span>
  );
}
