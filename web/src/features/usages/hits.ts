import {
  CONFIDENCE,
  LOCATION_JOIN,
  NAME_SEPARATOR,
  NOT_FOUND,
  REF,
  STATE,
  type State,
} from "./consts";
import type { CodeLine, Hit, Place, UsagesAnswer } from "./types";

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

/** The key a code line is found by: the place of the hit. */
export function placeKey(path: string, line: number): string {
  return `${path}${LOCATION_JOIN}${line}`;
}

/** The code lines by the place of their hit. */
export function codeByPlace(lines: CodeLine[]): Map<string, string> {
  return new Map(lines.map((found) => [placeKey(found.path, found.line), found.text]));
}

/** The last part of a qualified name: the word to mark in the code of a hit. */
export function shortName(place: Place): string {
  return place.qualified_name.split(NAME_SEPARATOR).pop() ?? place.qualified_name;
}

/** A line of code cut around the first occurrence of `word`. A line without the word comes back
 * whole in `before`. */
export function splitAt(
  text: string,
  word: string,
): { before: string; word: string; after: string } {
  const at = word === "" ? NOT_FOUND : text.indexOf(word);
  if (at === NOT_FOUND) {
    return { before: text, word: "", after: "" };
  }
  return { before: text.slice(0, at), word, after: text.slice(at + word.length) };
}
