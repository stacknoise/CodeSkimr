import { createEffect, createSignal, For, on, onCleanup, Show } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { ErrorDetails } from "../../shared/ui/ErrorDetails";
import {
  heldNow,
  holdFailed,
  holdState,
  loadRules,
  openedNow,
  openFailed,
  openProjectFile,
  resetRules,
  rulesData,
  rulesFailure,
} from "./rules";
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

/** The button that holds the state, with what it did. A Git project has no project file to put
 * the file next to, so the button is off there. */
function HoldButton(props: { project: number; answer: RulesOut | null }) {
  const [busy, setBusy] = createSignal(false);
  const hold = async () => {
    setBusy(true);
    await holdState(props.project);
    setBusy(false);
  };
  return (
    <button
      type="button"
      class={styles.hold}
      disabled={busy() || props.answer?.holdable !== true || props.answer.scanned !== true}
      title={
        props.answer?.holdable === false
          ? t(I18N_KEYS.RULES_HOLD_GIT)
          : t(I18N_KEYS.RULES_HOLD_HINT)
      }
      onClick={hold}
    >
      {t(I18N_KEYS.RULES_HOLD)}
    </button>
  );
}

/** The button that opens the project file with the program of the system. Like holding the state
 * it needs a folder project. */
function OpenButton(props: { project: number; answer: RulesOut | null }) {
  const [busy, setBusy] = createSignal(false);
  const open = async () => {
    setBusy(true);
    await openProjectFile(props.project);
    setBusy(false);
  };
  return (
    <button
      type="button"
      class={styles.hold}
      disabled={busy() || props.answer?.holdable !== true}
      title={
        props.answer?.holdable === false
          ? t(I18N_KEYS.RULES_OPEN_GIT)
          : t(I18N_KEYS.RULES_OPEN_HINT)
      }
      onClick={open}
    >
      {t(I18N_KEYS.RULES_OPEN)}
    </button>
  );
}

/** What opening the project file did: that it was created, or why it failed. */
function OpenResult() {
  return (
    <>
      <Show when={openedNow()?.created}>
        <p class={styles.note} role="status">
          {t(I18N_KEYS.RULES_OPEN_CREATED, { file: openedNow()?.file ?? "" })}
        </p>
      </Show>
      <Show when={openFailed() !== undefined}>
        <div class={styles.alert} role="alert">
          {openFailed() === null
            ? t(I18N_KEYS.RULES_OPEN_FAILED)
            : errorText(openFailed() as ErrorCode)}
        </div>
      </Show>
    </>
  );
}

/** What holding the state did: the day and how many old burdens the file has now, or why it
 * failed. */
function HoldResult() {
  return (
    <>
      <Show when={heldNow()}>
        {(done) => (
          <p class={styles.note} role="status">
            {t(I18N_KEYS.RULES_HOLD_DONE, { date: done().held_on, count: done().entries })}
          </p>
        )}
      </Show>
      <Show when={holdFailed() !== undefined}>
        <div class={styles.alert} role="alert">
          {holdFailed() === null
            ? t(I18N_KEYS.RULES_HOLD_FAILED)
            : errorText(holdFailed() as ErrorCode)}
        </div>
      </Show>
    </>
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
      <header class={styles.header}>
        <div>
          <h1>{t(I18N_KEYS.RULES_TITLE)}</h1>
          <p class={styles.sub}>{t(I18N_KEYS.RULES_SUBTITLE)}</p>
        </div>
        <Show when={activeProject()}>
          {(project) => (
            <div class={styles.actions}>
              <OpenButton project={project()} answer={rulesData()} />
              <HoldButton project={project()} answer={rulesData()} />
            </div>
          )}
        </Show>
      </header>
      <Show
        when={activeProject()}
        fallback={<p class={styles.note}>{t(I18N_KEYS.RULES_NO_PROJECT)}</p>}
      >
        <Failure />
        <HoldResult />
        <OpenResult />
        <Show when={rulesData()}>{(data) => <Answer data={data()} />}</Show>
      </Show>
    </>
  );
}
