import { createEffect, For, on, onCleanup, Show } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { ErrorDetails } from "../../shared/ui/ErrorDetails";
import { loadRules, resetRules, rulesData, rulesFailure } from "./rules";
import { RuleCard } from "./RuleCard";
import type { RulesOut } from "./types";
import styles from "./RulesScreen.module.css";

/** One number of the strip above the rules, with what it counts. */
function Figure(props: { value: string; label: string }) {
  return (
    <div class={styles.figure}>
      <span class={styles.figureValue}>{props.value}</span>
      <span class={styles.figureLabel}>{props.label}</span>
    </div>
  );
}

/** The four numbers: kept rules, new violations, old burdens, resolved ones. */
function Summary(props: { data: RulesOut }) {
  const kept = () => props.data.rules.filter((rule) => rule.violations.length === 0).length;
  const held = () => props.data.held_on;
  return (
    <div class={styles.summary}>
      <Figure
        value={t(I18N_KEYS.RULES_KEPT_COUNT, { count: kept() })}
        label={t(I18N_KEYS.RULES_KEPT_LABEL)}
      />
      <Figure
        value={t(I18N_KEYS.RULES_FAILING_COUNT, { count: props.data.new })}
        label={t(I18N_KEYS.RULES_FAILING_LABEL)}
      />
      <Show
        when={held()}
        fallback={
          <Figure value={t(I18N_KEYS.RULES_HELD_NONE)} label={t(I18N_KEYS.RULES_HELD_NONE_LABEL)} />
        }
      >
        {(date) => (
          <Figure
            value={t(I18N_KEYS.RULES_HELD_COUNT, { count: props.data.old, date: date() })}
            label={t(I18N_KEYS.RULES_HELD_LABEL)}
          />
        )}
      </Show>
      <Show when={held()}>
        <Figure
          value={t(I18N_KEYS.RULES_RESOLVED_COUNT, { count: props.data.resolved })}
          label={t(I18N_KEYS.RULES_RESOLVED_LABEL)}
        />
      </Show>
    </div>
  );
}

/** The rules of an answer, or the reason why there are none to show. */
function Answer(props: { data: RulesOut }) {
  return (
    <Show
      when={props.data.scanned}
      fallback={<p class={styles.note}>{t(I18N_KEYS.RULES_NOT_SCANNED)}</p>}
    >
      <Show
        when={props.data.rules.length > 0}
        fallback={<p class={styles.note}>{t(I18N_KEYS.RULES_NONE)}</p>}
      >
        <Summary data={props.data} />
        <For each={props.data.rules}>{(rule) => <RuleCard rule={rule} />}</For>
      </Show>
    </Show>
  );
}

/** The failure of the read, with the code and its details. */
function Failure() {
  return (
    <Show when={rulesFailure()}>
      {(failure) => (
        <div class={styles.alert} role="alert">
          {failure().code === null
            ? t(I18N_KEYS.RULES_LOAD_FAILED)
            : errorText(failure().code as ErrorCode)}
          <Show when={failure().code}>
            {(code) => <ErrorDetails code={code()} message={errorText(code())} />}
          </Show>
        </div>
      )}
    </Show>
  );
}

/** The screen "Rules": every architecture rule of the project with its violations (ADR 0104). */
export function RulesScreen() {
  createEffect(
    on(activeProject, (id) => {
      resetRules();
      if (id !== null) {
        void loadRules(id);
      }
    }),
  );
  onCleanup(resetRules);
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.RULES_TITLE)}</h1>
        <p class={styles.sub}>{t(I18N_KEYS.RULES_SUBTITLE)}</p>
      </header>
      <Show
        when={activeProject()}
        fallback={<p class={styles.note}>{t(I18N_KEYS.RULES_NO_PROJECT)}</p>}
      >
        <Failure />
        <Show when={rulesData()}>{(data) => <Answer data={data()} />}</Show>
      </Show>
    </>
  );
}
