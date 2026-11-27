import { Show } from "solid-js";
import { Icon } from "../../shared/ui/Icon";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { DIALOG, ICON, ICON_SIZE, KIND } from "./consts";
import { indexOfRule } from "./edit";
import { openDialog } from "./editor";
import type { RuleOut, RulesOut } from "./types";
import styles from "./RulesScreen.module.css";

/** The dialog that changes a rule of this kind; the cycle rule is a switch and has none. */
function changeDialog(rule: RuleOut) {
  if (rule.kind === KIND.layers) {
    return DIALOG.layers;
  }
  return rule.kind === KIND.forbid ? DIALOG.forbid : DIALOG.publicApi;
}

/** The pen and the bin at the right of a rule card (mock 41, ADR 0112): the pen opens the dialog
 * that changes the rule, the bin the one that asks before the rule is removed. */
export function RuleActions(props: { project: number; data: RulesOut; rule: RuleOut }) {
  const index = () => indexOfRule(props.data, props.rule);
  return (
    <span class={styles.ruleActions}>
      <Show when={props.rule.kind !== KIND.cycles}>
        <button
          type="button"
          class={styles.iconButton}
          title={t(I18N_KEYS.RULES_EDIT)}
          aria-label={`${t(I18N_KEYS.RULES_EDIT)} ${props.rule.id}`}
          onClick={() =>
            void openDialog(props.project, {
              kind: changeDialog(props.rule),
              index: index(),
              rule: props.rule,
            })
          }
        >
          <Icon name={ICON.pen} size={ICON_SIZE} />
        </button>
      </Show>
      <button
        type="button"
        class={styles.iconButton}
        title={t(I18N_KEYS.RULES_DELETE)}
        aria-label={`${t(I18N_KEYS.RULES_DELETE)} ${props.rule.id}`}
        onClick={() =>
          void openDialog(props.project, { kind: DIALOG.remove, index: index(), rule: props.rule })
        }
      >
        <Icon name={ICON.bin} size={ICON_SIZE} />
      </button>
    </span>
  );
}
