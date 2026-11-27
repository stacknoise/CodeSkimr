import { For, Show } from "solid-js";
import { currentLanguage, errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { NoticeBox } from "../../shared/ui/NoticeBox";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import {
  changeMetrics,
  changeMetricsFailure,
  changeMetricsLevel,
  changeMetricsMetric,
  changeMetricsReading,
  chooseMetricsLevel,
  showMetricsMetric,
} from "./change";
import { Selectors } from "./Selectors";
import { Side } from "./MetricsTable";
import {
  METRIC_CHOICES,
  METRICS_LIST,
  METRICS_LEVEL_FIELD_ID,
  METRICS_LEVELS,
  NO_TAG,
  UNKNOWN_FIGURE,
} from "./consts";
import { Figure } from "./Figure";
import {
  crossingCount,
  metricAvailable,
  narrowedTo,
  notComparable,
  unchangedCount,
} from "./metrics";
import type { ChangeAnswer, MetricsAnswer } from "./types";
import parts from "./Parts.module.css";
import styles from "./MetricsTab.module.css";

/** The figure of the objects that cannot be compared: the new and the removed ones. */
function NotComparable(props: { compared: ChangeAnswer | null }) {
  const numbers = () => (props.compared === null ? null : notComparable(props.compared));
  return (
    <Figure
      label={t(I18N_KEYS.CHANGE_METRICS_CARD_NEW)}
      value={String(numbers() === null ? UNKNOWN_FIGURE : numbers()!.added + numbers()!.removed)}
      hint={
        numbers() === null
          ? t(I18N_KEYS.CHANGE_METRICS_CARD_NEW_UNKNOWN)
          : t(I18N_KEYS.CHANGE_METRICS_CARD_NEW_HINT, numbers()!)
      }
    />
  );
}

/** The four figures: what got worse, what got better, what stayed and what cannot be compared. */
function Figures(props: { answer: MetricsAnswer; compared: ChangeAnswer | null }) {
  const same = () => unchangedCount(props.answer);
  const sameText = () => {
    const count = same();
    return count === null ? UNKNOWN_FIGURE : count.toLocaleString(currentLanguage());
  };
  return (
    <div class={parts.figures}>
      <Figure
        label={t(I18N_KEYS.CHANGE_METRICS_CARD_WORSE)}
        value={String(props.answer.worse.total)}
        hint={t(I18N_KEYS.CHANGE_METRICS_CARD_WORSE_HINT, {
          crossing: crossingCount(props.answer.worse.items),
        })}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_METRICS_CARD_BETTER)}
        value={String(props.answer.better.total)}
        hint={t(I18N_KEYS.CHANGE_METRICS_CARD_BETTER_HINT, {
          crossing: crossingCount(props.answer.better.items),
        })}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_METRICS_CARD_SAME)}
        value={sameText()}
        hint={t(
          same() === null
            ? I18N_KEYS.CHANGE_METRICS_CARD_SAME_UNKNOWN
            : I18N_KEYS.CHANGE_METRICS_CARD_SAME_HINT,
        )}
      />
      <NotComparable compared={props.compared} />
    </div>
  );
}

/** The chip of one metric. A metric that the level does not have is greyed out; the tooltip says
 * why. */
function Chip(props: { choice: (typeof METRIC_CHOICES)[number] }) {
  const unavailable = () => !metricAvailable(props.choice.id, changeMetricsLevel());
  return (
    <button
      type="button"
      class={cx(styles.chip, changeMetricsMetric() === props.choice.id && styles.on)}
      aria-pressed={changeMetricsMetric() === props.choice.id}
      disabled={unavailable()}
      title={unavailable() ? t(I18N_KEYS.CHANGE_METRICS_UNAVAILABLE) : undefined}
      onClick={() => showMetricsMetric(props.choice.id)}
    >
      {t(props.choice.label)}
    </button>
  );
}

/** The choice of the metric (chips) and of the level. */
function Controls(props: { project: number }) {
  return (
    <div class={styles.controls}>
      <span class={parts.figureLabel}>{t(I18N_KEYS.CHANGE_METRICS_FILTER_LABEL)}</span>
      <For each={METRIC_CHOICES}>{(choice) => <Chip choice={choice} />}</For>
      <span class={styles.spacer} />
      <label class={styles.levelLabel} for={METRICS_LEVEL_FIELD_ID}>
        {t(I18N_KEYS.CHANGE_METRICS_LEVEL_LABEL)}
      </label>
      <select
        id={METRICS_LEVEL_FIELD_ID}
        class={styles.level}
        disabled={changeMetricsReading()}
        title={changeMetricsReading() ? t(I18N_KEYS.CHANGE_METRICS_READING) : undefined}
        onChange={(event) => chooseMetricsLevel(props.project, event.currentTarget.value)}
      >
        <For each={METRICS_LEVELS}>
          {(level) => (
            <option value={level.id} selected={changeMetricsLevel() === level.id}>
              {t(level.label)}
            </option>
          )}
        </For>
      </select>
    </div>
  );
}

/** The two lists side by side. */
function Lists(props: { answer: MetricsAnswer }) {
  const tag = () => METRICS_LEVELS.find((level) => level.id === props.answer.level)?.tag ?? NO_TAG;
  const moved = () =>
    narrowedTo(props.answer.worse, changeMetricsMetric()).length +
    narrowedTo(props.answer.better, changeMetricsMetric()).length;
  return (
    <Show
      when={moved() > 0}
      fallback={<p class={parts.plain}>{t(I18N_KEYS.CHANGE_METRICS_NONE)}</p>}
    >
      <div class={styles.sides}>
        <Side
          side={METRICS_LIST.worse}
          title={t(I18N_KEYS.CHANGE_METRICS_LIST_WORSE)}
          list={props.answer.worse}
          metric={changeMetricsMetric()}
          tag={tag()}
          tone={TONE.bad}
        />
        <Side
          side={METRICS_LIST.better}
          title={t(I18N_KEYS.CHANGE_METRICS_LIST_BETTER)}
          list={props.answer.better}
          metric={changeMetricsMetric()}
          tag={tag()}
          tone={TONE.good}
        />
      </div>
    </Show>
  );
}

/** The view "Metrics" (mock 43 a): the two scans, the figures, the choice of metric and level and
 * the two lists of what got worse and what got better. */
export function MetricsTab(props: { project: number; answer: ChangeAnswer | null }) {
  return (
    <>
      <Selectors project={props.project} />
      <Show when={changeMetricsFailure()}>
        {(failure) => (
          <NoticeBox
            alert
            text={
              failure().code === null
                ? t(I18N_KEYS.CHANGE_METRICS_LOAD_FAILED)
                : errorText(failure().code as ErrorCode)
            }
            code={failure().code}
          />
        )}
      </Show>
      <Show when={changeMetricsReading() && changeMetrics() === null}>
        <p class={parts.plain} role="status">
          {t(I18N_KEYS.CHANGE_METRICS_READING)}
        </p>
      </Show>
      <Show when={changeMetrics()}>
        {(answer) => (
          <>
            <Figures answer={answer()} compared={props.answer} />
            <Controls project={props.project} />
            <Lists answer={answer()} />
            <p class={parts.plain}>{t(I18N_KEYS.CHANGE_METRICS_NOTE)}</p>
          </>
        )}
      </Show>
    </>
  );
}
