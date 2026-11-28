/** What a run of code is for the colours (`TokenClass` of the server). */
export type SyntaxClass =
  "comment" | "string" | "number" | "keyword" | "type" | "function" | "attribute";

/** A coloured run of a piece of code; `line` counts the lines of the piece from 0. */
export interface CodeToken {
  line: number;
  start: number;
  end: number;
  class: SyntaxClass;
}

/** The answer of `code.highlight`. */
export interface HighlightAnswer {
  pieces: { tokens: CodeToken[] }[];
}

/** A stretch of text with the class of its colour, `null` for plain code. */
export interface Part {
  text: string;
  syntax: SyntaxClass | null;
  /** The part lies inside the stretch the caller asked to mark. */
  marked?: boolean;
}

/** A stretch of the first line that the caller marks, in UTF-16 units. */
export interface MarkRange {
  start: number;
  end: number;
}
