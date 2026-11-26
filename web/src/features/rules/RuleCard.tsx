import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  ARROW,
  INTERNAL_JOIN,
  INTERNAL_SUFFIX,
  KIND,
  MARK,
  PLACE_SEPARATOR,
  PLACES_JOIN,
  VISIBLE_PLACES,
  VISIBLE_VIOLATIONS,
} from "./consts";
import type { PlaceOut, RuleOut, ViolationOut } from "./types";
import styles from "./RulesScreen.module.css";

/** `path:line` of a place, as an editor takes it. */
function placeText(place: PlaceOut): string {
  return `${place.path}${PLACE_SEPARATOR}${place.line}`;
}

/** The first places of a violation; the rest is counted in its own text. */
function placesText(violation: ViolationOut): string {
  return violation.places.slice(0, VISIBLE_PLACES).map(placeText).join(PLACES_JOIN);
}

/** One violation: who reaches whom, how often, where, and whether it is an old burden. */
function ViolationRow(props: { violation: ViolationOut }) {
  return (
    <li class={styles.violation}>
      <span class={styles.symbol}>{props.violation.from}</span>
      <span class={styles.arrow} aria-hidden="true">
        {ARROW}
      </span>
      <span class={styles.symbol}>{props.violation.to}</span>
      <span class={styles.count}>
        {t(I18N_KEYS.RULES_PLACES, { count: props.violation.places.length })}
      </span>
      <span class={styles.places}>{placesText(props.violation)}</span>
      <span class={props.violation.old ? styles.badgeOld : styles.badgeNew}>
        {props.violation.old ? t(I18N_KEYS.RULES_VIOLATION_OLD) : t(I18N_KEYS.RULES_VIOLATION_NEW)}
      </span>
    </li>
  );
}

/** The title and the line of the rule as the project file says it. */
function Heading(props: { rule: RuleOut }) {
  const forbid = () => props.rule.kind === KIND.forbid;
  return (
    <>
      <span class={styles.title}>
        {forbid()
          ? t(I18N_KEYS.RULES_FORBID_TITLE, { from: props.rule.from, to: props.rule.to })
          : t(I18N_KEYS.RULES_PUBLIC_API_TITLE)}
      </span>
      <span class={styles.kindBadge}>{t(I18N_KEYS.RULES_KIND_NEW)}</span>
      <span class={styles.spec}>
        {forbid()
          ? (props.rule.reason ??
            t(I18N_KEYS.RULES_FORBID_SPEC, { from: props.rule.from, to: props.rule.to }))
          : t(I18N_KEYS.RULES_PUBLIC_API_SPEC, {
              module: props.rule.from,
              internal: props.rule.internal
                .map((prefix) => `${prefix}${INTERNAL_SUFFIX}`)
                .join(INTERNAL_JOIN),
            })}
      </span>
    </>
  );
}

/** The badges at the right of the heading: new violations, old burdens, or "kept". */
function Badges(props: { rule: RuleOut }) {
  const old = () => props.rule.violations.length - props.rule.new;
  return (
    <span class={styles.badges}>
      <Show when={props.rule.violations.length === 0}>
        <span class={styles.badgeKept}>{t(I18N_KEYS.RULES_STATE_KEPT)}</span>
      </Show>
      <Show when={props.rule.new > 0}>
        <span class={styles.badgeNew}>
          {t(I18N_KEYS.RULES_BADGE_NEW, { count: props.rule.new })}
        </span>
      </Show>
      <Show when={old() > 0}>
        <span class={styles.badgeOld}>{t(I18N_KEYS.RULES_BADGE_OLD, { count: old() })}</span>
      </Show>
    </span>
  );
}

/** The violations of a rule, the first few at first and a button for the rest. */
function Violations(props: { rule: RuleOut }) {
  const [all, setAll] = createSignal(false);
  const shown = () =>
    all() ? props.rule.violations : props.rule.violations.slice(0, VISIBLE_VIOLATIONS);
  const hidden = () => props.rule.violations.length - VISIBLE_VIOLATIONS;
  return (
    <>
      <ul class={styles.violations}>
        <For each={shown()}>{(violation) => <ViolationRow violation={violation} />}</For>
      </ul>
      <Show when={hidden() > 0}>
        <button type="button" class={styles.more} onClick={() => setAll(!all())}>
          {all()
            ? t(I18N_KEYS.RULES_SHOW_LESS)
            : `${t(I18N_KEYS.RULES_MORE, { count: hidden() })} · ${t(I18N_KEYS.RULES_SHOW_ALL)}`}
        </button>
      </Show>
    </>
  );
}

/** One rule: its state mark, heading, badges and violations. */
export function RuleCard(props: { rule: RuleOut }) {
  const broken = () => props.rule.violations.length > 0;
  return (
    <section class={styles.card} aria-label={props.rule.id}>
      <div class={styles.head}>
        <span class={broken() ? styles.markBroken : styles.markKept} aria-hidden="true">
          {broken() ? MARK.broken : MARK.kept}
        </span>
        <Heading rule={props.rule} />
        <Badges rule={props.rule} />
      </div>
      <Show when={broken()}>
        <Violations rule={props.rule} />
      </Show>
    </section>
  );
}
