import { createEffect, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { Pill } from "../../shared/ui/Pill";
import { TONE as PILL_TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import {
  KEY,
  LEVEL2_UNAVAILABLE,
  LOCATION_JOIN,
  SCROLL_NEAREST,
  REF,
  REF_LABEL,
  STATE,
  TRUE,
  type State,
} from "./consts";
import { countKinds, listedPlace, stateOf } from "./hits";
import { selectedHit, setSelectedHit } from "./usages";
import type { Hit, Place, UsagesAnswer } from "./types";
import styles from "./HitList.module.css";

const STATE_LABEL: Record<State, MessageKey> = {
  [STATE.verified]: I18N_KEYS.USAGES_VERIFIED,
  [STATE.unchecked]: I18N_KEYS.USAGES_UNCHECKED,
  [STATE.presumed]: I18N_KEYS.USAGES_PRESUMED,
};

function kindText(hit: Hit): string {
  const label = REF_LABEL[hit.kind];
  return label ? t(label) : hit.kind;
}

function location(hit: Hit): string {
  return `${hit.path}${LOCATION_JOIN}${hit.line}`;
}

function Row(props: { hit: Hit; place: Place; index: number; onOpen: (hit: Hit) => void }) {
  const state = () => stateOf(props.hit);
  let row: HTMLDivElement | undefined;
  createEffect(() => {
    if (selectedHit() === props.index) {
      row?.scrollIntoView?.(SCROLL_NEAREST);
    }
  });
  return (
    <div
      ref={row}
      class={cx(styles.hit, selectedHit() === props.index && styles.selected)}
      aria-current={selectedHit() === props.index ? TRUE : undefined}
      onClick={() => setSelectedHit(props.index)}
      onDblClick={() => props.onOpen(props.hit)}
    >
      <span class={cx(styles.kind, props.hit.kind === REF.test && styles.test)}>
        {kindText(props.hit)}
      </span>
      <span class={styles.where} title={props.place.qualified_name}>
        {props.place.qualified_name}
      </span>
      <span class={styles.place}>{location(props.hit)}</span>
      <span
        class={cx(styles.state, styles[state()])}
        title={state() === STATE.unchecked ? t(I18N_KEYS.USAGES_UNCHECKED_TIP) : undefined}
      >
        {t(STATE_LABEL[state()])}
      </span>
    </div>
  );
}

/** The hit list of screen 6 with the counts by kind and by certainty (mock 6). */
export function HitList(props: { answer: UsagesAnswer; hits: Hit[]; onOpen: (hit: Hit) => void }) {
  const kinds = () => countKinds(props.hits);
  const walk = () => props.answer.incoming ?? props.answer.outgoing;
  return (
    <div class={styles.list}>
      <div class={styles.head}>
        <b>{t(I18N_KEYS.USAGES_HITS, { count: props.hits.length })}</b>
        <Show when={kinds().tests > 0}>
          <Pill tone={PILL_TONE.accent}>
            {t(I18N_KEYS.USAGES_REF_TEST)} {kinds().tests}
          </Pill>
        </Show>
        <Show when={kinds().presumed > 0}>
          <Pill>
            {t(I18N_KEYS.USAGES_PRESUMED)} {kinds().presumed}
          </Pill>
        </Show>
        <Show when={(walk()?.total ?? 0) > props.hits.length}>
          <span class={styles.cut}>
            {t(I18N_KEYS.USAGES_HITS_CUT, {
              shown: props.hits.length,
              total: walk()?.total ?? 0,
            })}
          </span>
        </Show>
        <span class={styles.keys}>
          <kbd>{KEY.next}</kbd>
          <span>{t(I18N_KEYS.USAGES_HITS_NEXT)}</span>
          <kbd>{KEY.open}</kbd>
          <span>{t(I18N_KEYS.USAGES_HITS_OPEN)}</span>
        </span>
      </div>
      <div class={styles.rows}>
        <Show
          when={props.hits.length > 0}
          fallback={
            <p class={styles.empty}>
              {t(I18N_KEYS.USAGES_HITS_EMPTY)}
              <Show when={props.answer.level2 === LEVEL2_UNAVAILABLE}>
                {t(I18N_KEYS.USAGES_HITS_EMPTY_UNCHECKED)}
              </Show>
            </p>
          }
        >
          <For each={props.hits}>
            {(hit, index) => (
              <Row
                hit={hit}
                place={listedPlace(props.answer, hit)}
                index={index()}
                onOpen={props.onOpen}
              />
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
