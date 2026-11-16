import type { SourceSpan, SourceToken, TokenClass } from "./types";

/** A piece of the code: plain text, the name of a symbol, a coloured run, or both. */
export interface Segment {
  text: string;
  /** Id of the symbol the text names, `null` for plain text. */
  symbol: number | null;
  /** What the text is for the colours, `null` for plain code. */
  tokenClass: TokenClass | null;
}

/** Joins a line break to the piece before it. */
const LINE_END = "\n";

/** A stretch of one line that carries a value (a symbol or a class). */
interface Run<T> {
  start: number;
  end: number;
  value: T;
}

/** The runs of line `index` that fit the line, in order, without overlaps: a run that overlaps the
 * one before it, has no width or does not fit its line is dropped. */
function runsOf<T>(
  items: readonly { line: number; start: number; end: number; value: T }[],
  index: number,
  length: number,
): Run<T>[] {
  const runs: Run<T>[] = [];
  let at = 0;
  for (const item of items) {
    if (item.line === index && item.start >= at && item.end > item.start) {
      const end = Math.min(item.end, length);
      if (end > item.start) {
        runs.push({ start: item.start, end, value: item.value });
        at = end;
      }
    }
  }
  return runs;
}

/** The value of the run that covers position `at`, `null` if none does. */
function valueAt<T>(runs: readonly Run<T>[], at: number): T | null {
  return runs.find((run) => run.start <= at && at < run.end)?.value ?? null;
}

/** Cuts one line at every place a symbol or a coloured run starts or ends. */
function cutLine(line: string, symbols: Run<number>[], tokens: Run<TokenClass>[]): Segment[] {
  const cuts = new Set<number>([0, line.length]);
  for (const run of [...symbols, ...tokens]) {
    cuts.add(run.start);
    cuts.add(run.end);
  }
  const sorted = [...cuts].sort((a, b) => a - b);
  return sorted.slice(0, sorted.length - 1).map((from, position) => ({
    text: line.slice(from, sorted[position + 1]),
    symbol: valueAt(symbols, from),
    tokenClass: valueAt(tokens, from),
  }));
}

/** Cuts the lines into pieces that are plain, the name of a symbol or coloured, in the order of the
 * text. A span or run that overlaps the one before it, or that does not fit its line, is dropped,
 * so the pieces always give the lines back when put together. */
export function segments(
  lines: readonly string[],
  spans: readonly SourceSpan[],
  tokens: readonly SourceToken[] = [],
): Segment[] {
  const named = spans.map((span) => ({ ...span, value: span.symbol }));
  const coloured = tokens.map((token) => ({ ...token, value: token.class }));
  const pieces = lines.flatMap((line, index) => {
    const own = cutLine(
      line,
      runsOf(named, index, line.length),
      runsOf(coloured, index, line.length),
    );
    return index < lines.length - 1 ? withLineEnd(own) : own;
  });
  return pieces.filter((piece) => piece.text !== "");
}

/** Puts the line break behind the pieces of a line: onto the last piece if that is plain, else as a
 * piece of its own, so a name or a colour never covers it. */
function withLineEnd(pieces: Segment[]): Segment[] {
  const last = pieces[pieces.length - 1];
  if (last !== undefined && last.symbol === null && last.tokenClass === null) {
    return [...pieces.slice(0, pieces.length - 1), { ...last, text: last.text + LINE_END }];
  }
  return [...pieces, { text: LINE_END, symbol: null, tokenClass: null }];
}
