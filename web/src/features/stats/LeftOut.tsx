import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import { KindBadge } from "../explorer/KindBadge";
import { LIST_JOIN } from "./consts";
import { liftRule } from "./keep";
import styles from "./LeftOut.module.css";
import type { IgnoreRule, LeftOut } from "./types";

/** The entry of the exception list as a small list of its parts. */
function RuleView(props: { rule: IgnoreRule }) {
  const part = (label: string, values: string[] | undefined) => (
    <Show when={values !== undefined && values.length > 0}>
      <dt>{label}</dt>
      <dd>{values?.join(LIST_JOIN)}</dd>
    </Show>
  );
  return (
    <dl class={styles.rule}>
      {part(t(I18N_KEYS.STATS_LEFT_OUT_PATHS), props.rule.paths)}
      {part(t(I18N_KEYS.STATS_LEFT_OUT_SYMBOLS), props.rule.symbols)}
      <dt>{t(I18N_KEYS.STATS_LEFT_OUT_REASON)}</dt>
      <dd>{props.rule.reason}</dd>
    </dl>
  );
}

function Line(props: { item: LeftOut; project: number; editable: boolean }) {
  const [open, setOpen] = createSignal(false);
  return (
    <li class={styles.line}>
      <div class={styles.head}>
        <KindBadge kind={props.item.row.kind} />
        <span class={styles.name} title={props.item.row.qualified_name}>
          {props.item.row.name}
        </span>
        <span class={styles.path}>{props.item.row.path}</span>
        <Pill tone={TONE.good}>{t(I18N_KEYS.STATS_LEFT_OUT_PILL)}</Pill>
        <span class={styles.reason}>
          {t(I18N_KEYS.STATS_LEFT_OUT_RULE, { reason: props.item.rule.reason })}
        </span>
        <button type="button" aria-expanded={open()} onClick={() => setOpen(!open())}>
          {t(open() ? I18N_KEYS.STATS_LEFT_OUT_HIDE : I18N_KEYS.STATS_LEFT_OUT_SHOW)}
        </button>
        <button
          type="button"
          disabled={!props.editable}
          title={props.editable ? undefined : t(I18N_KEYS.STATS_KEEP_GIT)}
          aria-label={t(I18N_KEYS.STATS_LIFT_ROW, { name: props.item.row.name })}
          onClick={() => void liftRule(props.project, props.item.rule)}
        >
          {t(I18N_KEYS.STATS_LIFT_BUTTON)}
        </button>
      </div>
      <Show when={open()}>
        <RuleView rule={props.item.rule} />
      </Show>
    </li>
  );
}

/** The symbols the exception list of the project leaves out of the ranking, with the entry that
 * does (ADR 0101); `total` counts all of them, `items` holds the first ones. */
export function LeftOutList(props: {
  items: LeftOut[];
  total: number;
  project: number;
  editable: boolean;
}) {
  return (
    <section class={styles.section} aria-label={t(I18N_KEYS.STATS_LEFT_OUT_TITLE)}>
      <h3>{t(I18N_KEYS.STATS_LEFT_OUT_TITLE)}</h3>
      <p class={styles.note}>{t(I18N_KEYS.STATS_LEFT_OUT_NOTE, { count: props.total })}</p>
      <Show when={props.total > props.items.length}>
        <p class={styles.note}>
          {t(I18N_KEYS.STATS_ROWS_SHOWN, { shown: props.items.length, total: props.total })}
        </p>
      </Show>
      <ul class={styles.list}>
        <For each={props.items}>
          {(item) => <Line item={item} project={props.project} editable={props.editable} />}
        </For>
      </ul>
    </section>
  );
}
