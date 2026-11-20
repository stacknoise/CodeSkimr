import {
  CONFIDENCE,
  DIRECT_DEPTH,
  IDENTIFIER_CHAR,
  LOCATION_JOIN,
  NAME_SEPARATOR,
  NOT_FOUND,
  REF,
  STATE,
  WORD_SEPARATOR,
  type State,
} from "./consts";
import type { CodeLine, Hit, Place, UsagesAnswer } from "./types";

/** The hits the list shows: the direct ones of the callers if the answer has them, else of the
 * callees. A symbol picked in the graph narrows them to the hits between it and the focus. */
export function listedHits(answer: UsagesAnswer, picked: number | null = null): Hit[] {
  const direct = ((answer.incoming ?? answer.outgoing)?.hits ?? []).filter(
    (hit) => hit.depth === DIRECT_DEPTH,
  );
  return picked === null ? direct : direct.filter((hit) => listedPlace(answer, hit).id === picked);
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

/** Where `word` stands as a whole word in `text`, `NOT_FOUND` if it does not. `Accept` is not
 * found in `AcceptedOperation`, but in `args.Accept`. */
function wholeWordAt(text: string, word: string): number {
  let at = text.indexOf(word);
  while (at !== NOT_FOUND) {
    const before = text.charAt(at - 1);
    const after = text.charAt(at + word.length);
    if (!IDENTIFIER_CHAR.test(before) && !IDENTIFIER_CHAR.test(after)) {
      return at;
    }
    at = text.indexOf(word, at + 1);
  }
  return NOT_FOUND;
}

/** A line of code cut around the first whole-word occurrence of `word`. A line without the word
 * comes back whole in `before`. */
export function splitAt(
  text: string,
  word: string,
): { before: string; word: string; after: string } {
  const at = word === "" ? NOT_FOUND : wholeWordAt(text, word);
  if (at === NOT_FOUND) {
    return { before: text, word: "", after: "" };
  }
  return { before: text.slice(0, at), word, after: text.slice(at + word.length) };
}

/** Whether every word of `query` is in `text`, ignoring case; an empty query matches all. */
export function matchesQuery(text: string, query: string): boolean {
  const haystack = text.toLowerCase();
  return query
    .toLowerCase()
    .split(WORD_SEPARATOR)
    .filter((word) => word !== "")
    .every((word) => haystack.includes(word));
}
