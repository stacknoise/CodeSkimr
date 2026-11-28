import { FIRST, LINE_END } from "./consts";
import type { CodeToken, MarkRange, Part } from "./types";

/** The parts of one line: plain stretches and coloured runs, in order, without overlaps. A run
 * that overlaps the one before it or does not fit the line is dropped. */
function partsOfLine(line: string, tokens: readonly CodeToken[]): Part[] {
  const parts: Part[] = [];
  let at = FIRST;
  for (const token of tokens) {
    const end = Math.min(token.end, line.length);
    if (token.start < at || end <= token.start) {
      continue;
    }
    if (token.start > at) {
      parts.push({ text: line.slice(at, token.start), syntax: null });
    }
    parts.push({ text: line.slice(token.start, end), syntax: token.class });
    at = end;
  }
  if (at < line.length) {
    parts.push({ text: line.slice(at), syntax: null });
  }
  return parts;
}

/** Cuts the parts at the borders of `mark` and flags the ones inside it. */
function marked(parts: readonly Part[], mark: MarkRange): Part[] {
  let at = FIRST;
  return parts.flatMap((part) => {
    const from = at;
    at += part.text.length;
    const cuts = [from, mark.start, mark.end, at]
      .filter((cut) => cut >= from && cut <= at)
      .sort((a, b) => a - b);
    return cuts.slice(0, cuts.length - 1).flatMap((cut, index) => {
      const end = cuts[index + 1] ?? at;
      if (end <= cut) {
        return [];
      }
      const inside = cut >= mark.start && end <= mark.end;
      return [{ ...part, text: part.text.slice(cut - from, end - from), marked: inside }];
    });
  });
}

/** The lines of `text` as lists of parts; put together, the parts give the line back. The first
 * line can have a stretch `mark`ed. */
export function linesOf(text: string, tokens: readonly CodeToken[], mark?: MarkRange): Part[][] {
  return text.split(LINE_END).map((line, index) => {
    const parts = partsOfLine(
      line,
      tokens.filter((token) => token.line === index),
    );
    return index === FIRST && mark !== undefined && mark.end > mark.start
      ? marked(parts, mark)
      : parts;
  });
}
