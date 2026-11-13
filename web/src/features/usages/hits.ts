import { CONFIDENCE, REF, STATE, type State } from "./consts";
import type { Hit, Place, UsagesAnswer } from "./types";

/** The hits the list shows: the callers if the answer has them, else the callees. */
export function listedHits(answer: UsagesAnswer): Hit[] {
  return (answer.incoming ?? answer.outgoing)?.hits ?? [];
}

/** How sure a hit is: verified by level 2, presumed by level 1 itself, or just unchecked. */
export function stateOf(hit: Hit): State {
  if (hit.verified) {
    return STATE.verified;
  }
  return hit.confidence === CONFIDENCE.probable ? STATE.presumed : STATE.unchecked;
}

/** How many of the hits are tests and how many are presumed. */
export function countKinds(hits: Hit[]): { tests: number; presumed: number } {
  return {
    tests: hits.filter((hit) => hit.kind === REF.test).length,
    presumed: hits.filter((hit) => stateOf(hit) === STATE.presumed).length,
  };
}

/** The symbol a hit of the list names: the caller for callers, the callee for the callees that the
 * list shows when the answer has no callers (there `from` is always the symbol in focus). */
export function listedPlace(answer: UsagesAnswer, hit: Hit): Place {
  return answer.incoming === null ? hit.to : hit.from;
}
