import type { SourceSpan } from "./types";

/** A piece of the code: plain text, or the name of a symbol. */
export interface Segment {
  text: string;
  /** Id of the symbol the text names, `null` for plain text. */
  symbol: number | null;
}

/** Joins a line break to the piece before it. */
const LINE_END = "\n";

/** Cuts the lines into plain pieces and names of symbols, in the order of the text. A span that
 * overlaps the one before it, or that does not fit its line, is dropped, so the pieces always
 * give the lines back when put together. */
export function segments(lines: readonly string[], spans: readonly SourceSpan[]): Segment[] {
  const pieces: Segment[] = [];
  let next = 0;
  lines.forEach((line, index) => {
    let at = 0;
    while (next < spans.length && (spans[next]?.line ?? index) <= index) {
      const span = spans[next++];
      if (span && span.line === index && span.start >= at && span.end > span.start) {
        const end = Math.min(span.end, line.length);
        pieces.push({ text: line.slice(at, span.start), symbol: null });
        pieces.push({ text: line.slice(span.start, end), symbol: span.symbol });
        at = end;
      }
    }
    pieces.push({
      text: line.slice(at) + (index < lines.length - 1 ? LINE_END : ""),
      symbol: null,
    });
  });
  return pieces.filter((piece) => piece.text !== "");
}
