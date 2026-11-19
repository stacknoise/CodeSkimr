import { t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { LOCATION_JOIN, REF_LABEL, STATE, type State } from "./consts";
import { listedPlace, matchesQuery, placeKey, stateOf } from "./hits";
import type { Hit, UsagesAnswer } from "./types";

/** The text of the state of a hit. */
export const STATE_LABEL: Record<State, MessageKey> = {
  [STATE.verified]: I18N_KEYS.USAGES_VERIFIED,
  [STATE.unchecked]: I18N_KEYS.USAGES_UNCHECKED,
  [STATE.presumed]: I18N_KEYS.USAGES_PRESUMED,
};

/** The text of the kind of a hit; a kind without a text shows its own name. */
export function kindText(hit: Hit): string {
  const label = REF_LABEL[hit.kind];
  return label ? t(label) : hit.kind;
}

/** The place of a hit as `path:line`. */
export function location(hit: Hit): string {
  return `${hit.path}${LOCATION_JOIN}${hit.line}`;
}

/** Everything a row of the list shows, as one text for the filter. */
export function rowText(answer: UsagesAnswer, hit: Hit, code: Map<string, string>): string {
  return [
    kindText(hit),
    listedPlace(answer, hit).qualified_name,
    code.get(placeKey(hit.path, hit.line)) ?? "",
    location(hit),
    t(STATE_LABEL[stateOf(hit)]),
  ].join(LOCATION_JOIN);
}

/** The hits whose row contains every word of `query`. */
export function filterHits(
  answer: UsagesAnswer,
  hits: Hit[],
  query: string,
  code: Map<string, string>,
): Hit[] {
  return query.trim() === ""
    ? hits
    : hits.filter((hit) => matchesQuery(rowText(answer, hit, code), query));
}
