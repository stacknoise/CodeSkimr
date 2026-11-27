import { For, Show } from "solid-js";
import { currentLanguage, errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { NoticeBox } from "../../shared/ui/NoticeBox";
import { Pill, type PillTone } from "../../shared/ui/Pill";
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
import { Selectors } from "./CompareTab";
import {
  CROSSING_TEXT,
  DIRECTION_TONE,
  METRIC_CHOICES,
  METRIC_TEXT,
  METRICS_LEVEL_FIELD_ID,
  METRICS_LEVELS,
  NO_TAG,
  UNKNOWN_FIGURE,
} from "./consts";
import { Figure } from "./Figure";
import {
  crossingCount,
  deltaText,
  metricValue,
  narrowedTo,
  notComparable,
  unchangedCount,
} from "./metrics";
import type { ChangeAnswer, Listed, MetricChangeOut, MetricMoveOut, MetricsAnswer } from "./types";
import parts from "./Parts.module.css";
import styles from "./MetricsTab.module.css";

/** One metric of an object: its name, from and to, the size of the move and, where it crossed the
 * bound of the rating, a note. */
function Move(props: { move: MetricMoveOut }) {
  return (
    <div class={styles.move}>
      <span class={styles.metric}>
        {t(METRIC_TEXT[props.move.metric] ?? I18N_KEYS.CHANGE_METRICS_FILTER_ALL)}
      </span>
      <span class={parts.name}>
        {metricValue(props.move.from)} → {metricValue(props.move.to)}
      </span>
      <Pill tone={DIRECTION_TONE[props.move.direction] ?? TONE.neutral}>
        {deltaText(props.move)}
      </Pill>
      <Show when={props.move.crossing}>
        {(crossing) => (
          <span class={styles.cross}>
            {t(CROSSING_TEXT[crossing()] ?? I18N_KEYS.CHANGE_METRICS_CROSSING_INTO)}
          </span>
        )}
      </Show>
    </div>
  );
}

/** One object: the tag of its level, its name, its file and the moves of its metrics. */
function Item(props: { item: MetricChangeOut; tag: string }) {
  return (
    <li class={styles.item}>
      <span class={styles.tag} aria-hidden={true}>
        {props.tag}
      </span>
      <div class={styles.body}>
        <b class={styles.name} title={props.item.name}>
          {props.item.name}
        </b>
        <span class={styles.where} title={props.item.path}>
          {props.item.path}
        </span>
        <div class={styles.moves}>
          <For each={props.item.moves}>{(move) => <Move move={move} />}</For>
        </div>
      </div>
    </li>
  );
}

/** One of the two lists: its heading with the count, the order and the objects. */
function Side(props: {
  title: string;
  list: Listed<MetricChangeOut>;
  items: MetricChangeOut[];
  tag: string;
  tone: PillTone;
}) {
  return (
    <section class={cx(parts.card, styles.side)} aria-label={props.title}>
      <header class={styles.sideHead}>
        <h2 class={cx(styles.sideTitle, styles[props.tone])}>{props.title}</h2>
        <Pill tone={props.tone}>{props.list.total}</Pill>
        <span class={styles.order}>{t(I18N_KEYS.CHANGE_METRICS_ORDER)}</span>
      </header>
      <ul class={styles.items} aria-label={t(I18N_KEYS.CHANGE_METRICS_LIST_LABEL)}>
        <For each={props.items}>{(item) => <Item item={item} tag={props.tag} />}</For>
      </ul>
    </section>
  );
}

/** The figure of the objects that cannot be compared: the new and the removed ones. */
function NotComparable(props: { compared: ChangeAnswer }) {
  const numbers = () => notComparable(props.compared);
  return (
    <Figure
      label={t(I18N_KEYS.CHANGE_METRICS_CARD_NEW)}
      value={String(numbers().added + numbers().removed)}
      hint={t(I18N_KEYS.CHANGE_METRICS_CARD_NEW_HINT, numbers())}
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
      <Show when={props.compared}>{(compared) => <NotComparable compared={compared()} />}</Show>
    </div>
  );
}

/** The choice of the metric (chips) and of the level. */
function Controls(props: { project: number }) {
  return (
    <div class={styles.controls}>
      <span class={parts.figureLabel}>{t(I18N_KEYS.CHANGE_METRICS_FILTER_LABEL)}</span>
      <For each={METRIC_CHOICES}>
        {(choice) => (
          <button
            type="button"
            class={cx(styles.chip, changeMetricsMetric() === choice.id && styles.on)}
            aria-pressed={changeMetricsMetric() === choice.id}
            onClick={() => showMetricsMetric(choice.id)}
          >
            {t(choice.label)}
          </button>
        )}
      </For>
      <span class={styles.spacer} />
      <label class={styles.levelLabel} for={METRICS_LEVEL_FIELD_ID}>
        {t(I18N_KEYS.CHANGE_METRICS_LEVEL_LABEL)}
      </label>
      <select
        id={METRICS_LEVEL_FIELD_ID}
        class={styles.level}
        disabled={changeMetricsReading()}
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
  const worse = () => narrowedTo(props.answer.worse, changeMetricsMetric());
  const better = () => narrowedTo(props.answer.better, changeMetricsMetric());
  return (
    <Show
      when={worse().length + better().length > 0}
      fallback={<p class={parts.plain}>{t(I18N_KEYS.CHANGE_METRICS_NONE)}</p>}
    >
      <div class={styles.sides}>
        <Side
          title={t(I18N_KEYS.CHANGE_METRICS_LIST_WORSE)}
          list={props.answer.worse}
          items={worse()}
          tag={tag()}
          tone={TONE.bad}
        />
        <Side
          title={t(I18N_KEYS.CHANGE_METRICS_LIST_BETTER)}
          list={props.answer.better}
          items={better()}
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
