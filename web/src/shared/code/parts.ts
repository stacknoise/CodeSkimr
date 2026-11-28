import { FIRST, LINE_END } from "./consts";
import type { CodeToken, Part } from "./types";

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

/** The lines of `text` as lists of parts; put together, the parts give the line back. */
export function linesOf(text: string, tokens: readonly CodeToken[]): Part[][] {
  return text.split(LINE_END).map((line, index) =>
    partsOfLine(
      line,
      tokens.filter((token) => token.line === index),
    ),
  );
}
