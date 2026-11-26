import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { showModuleInGraph } from "../graph/show";
import {
  ARROW,
  INTERNAL_JOIN,
  INTERNAL_SUFFIX,
  KIND,
  LAYERS_JOIN,
  MARK,
  MORE_JOIN,
  PLACE_SEPARATOR,
  PLACES_JOIN,
  PLACES_MORE,
  USES_JOIN,
  VISIBLE_PLACES,
  VISIBLE_VIOLATIONS,
} from "./consts";
import type { PlaceOut, RingOut, RuleOut, ViolationOut } from "./types";
import styles from "./RulesScreen.module.css";

/** `path:line` of a place, as an editor takes it. */
function placeText(place: PlaceOut): string {
  return `${place.path}${PLACE_SEPARATOR}${place.line}`;
}

/** The first places of a violation; the rest is counted in its own text. */
function placesText(violation: ViolationOut): string {
  const named = violation.places.slice(0, VISIBLE_PLACES).map(placeText).join(PLACES_JOIN);
  return violation.places.length > VISIBLE_PLACES ? `${named}${PLACES_MORE}` : named;
}

/** The badge that says whether a violation is an old burden or new. */
function Age(props: { old: boolean }) {
  return (
    <span class={props.old ? styles.badgeOld : styles.badgeNew}>
      {props.old ? t(I18N_KEYS.RULES_VIOLATION_OLD) : t(I18N_KEYS.RULES_VIOLATION_NEW)}
    </span>
  );
}

/** One cycle: the way round the ring, its size, the weakest edge and where to cut it. */
function CycleRow(props: { violation: ViolationOut; ring: RingOut }) {
  return (
    <li class={styles.violation}>
      <span class={styles.symbol}>{props.ring.chain.join(` ${ARROW} `)}</span>
      <span class={styles.advice}>
        {t(I18N_KEYS.RULES_RING_ADVICE, {
          edges: props.ring.resolution_edges,
          places: props.violation.places.length,
        })}
      </span>
      <button
        type="button"
        class={styles.show}
        onClick={() => showModuleInGraph(props.violation.from)}
      >
        {t(I18N_KEYS.RULES_RING_SHOW)}
      </button>
      <Age old={props.violation.old} />
    </li>
  );
}

/** One violation: who reaches whom, how often, where, and whether it is an old burden. */
function ViolationRow(props: { violation: ViolationOut }) {
  return (
    <Show when={props.violation.ring} fallback={<ReferenceRow violation={props.violation} />}>
      {(ring) => <CycleRow violation={props.violation} ring={ring()} />}
    </Show>
  );
}

/** A reference that a rule forbids, from the symbol that refers to the one it reaches. */
function ReferenceRow(props: { violation: ViolationOut }) {
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
      <Age old={props.violation.old} />
    </li>
  );
}

/** What each layer may use, as the layer rule states it: `Api → Domain; Data → nothing`. */
function layersSpec(rule: RuleOut): string {
  return rule.allowed
    .map((entry) =>
      t(I18N_KEYS.RULES_LAYERS_ALLOWED, {
        layer: entry.layer,
        uses:
          entry.may_use.length > 0
            ? entry.may_use.join(USES_JOIN)
            : t(I18N_KEYS.RULES_LAYERS_NOTHING),
      }),
    )
    .join(LAYERS_JOIN);
}

/** The title of the rule and the line under the badge that says what it demands. */
function titleAndSpec(rule: RuleOut): { title: string; spec: string } {
  if (rule.kind === KIND.layers) {
    return { title: t(I18N_KEYS.RULES_LAYERS_TITLE), spec: layersSpec(rule) };
  }
  if (rule.kind === KIND.cycles) {
    return { title: t(I18N_KEYS.RULES_CYCLES_TITLE), spec: t(I18N_KEYS.RULES_CYCLES_SPEC) };
  }
  if (rule.kind === KIND.forbid) {
    return {
      title: t(I18N_KEYS.RULES_FORBID_TITLE, { from: rule.from, to: rule.to }),
      spec: rule.reason ?? t(I18N_KEYS.RULES_FORBID_SPEC, { from: rule.from, to: rule.to }),
    };
  }
  return {
    title: t(I18N_KEYS.RULES_PUBLIC_API_TITLE),
    spec: t(I18N_KEYS.RULES_PUBLIC_API_SPEC, {
      module: rule.from,
      internal: rule.internal.map((prefix) => `${prefix}${INTERNAL_SUFFIX}`).join(INTERNAL_JOIN),
    }),
  };
}

/** The title and the line of the rule as the project file says it. The layers are no new kind
 * of rule, so they carry no badge. */
function Heading(props: { rule: RuleOut }) {
  const text = () => titleAndSpec(props.rule);
  return (
    <>
      <span class={styles.title}>{text().title}</span>
      <Show when={props.rule.kind !== KIND.layers}>
        <span class={styles.kindBadge}>{t(I18N_KEYS.RULES_KIND_NEW)}</span>
      </Show>
      <span class={styles.spec}>{text().spec}</span>
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
            : `${t(I18N_KEYS.RULES_MORE, { count: hidden() })}${MORE_JOIN}${t(I18N_KEYS.RULES_SHOW_ALL)}`}
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
