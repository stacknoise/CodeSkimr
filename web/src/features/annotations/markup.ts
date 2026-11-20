import { LINK_SCHEMES, MARKUP_RULES, PIECE } from "./consts";
import type { Piece } from "./types";

/** Whether an address may be a link: `http` or `https`, nothing else (no `javascript:`). */
export function isSafeLink(address: string): boolean {
  try {
    return LINK_SCHEMES.includes(new URL(address).protocol);
  } catch {
    return false;
  }
}

/** The part of the subset that starts at `at`, with the number of characters it takes. A link
 * whose address is not safe is plain text. */
function partAt(text: string, at: number): { piece: Piece; length: number } | null {
  for (const rule of MARKUP_RULES) {
    rule.pattern.lastIndex = at;
    const found = rule.pattern.exec(text);
    if (found === null) {
      continue;
    }
    const [, first = "", second] = found;
    const address = second ?? first;
    if (rule.kind === PIECE.link && !isSafeLink(address)) {
      continue;
    }
    const piece: Piece =
      rule.kind === PIECE.link
        ? { kind: rule.kind, text: second === undefined ? address : first, href: address }
        : { kind: rule.kind, text: first };
    return { piece, length: found[0].length };
  }
  return null;
}

/** Splits a text into pieces of the small Markdown subset: `**bold**`, `*italic*`, `` `code` ``,
 * `[text](https://…)` and bare `http`/`https` addresses. Everything else, `<` and `&` included,
 * stays plain text; the pieces are drawn as text nodes, never as HTML (ADR 0068, decision 8). */
export function parseMarkup(text: string): Piece[] {
  const pieces: Piece[] = [];
  let plain = "";
  let at = 0;
  while (at < text.length) {
    const part = partAt(text, at);
    if (part === null) {
      plain += text.charAt(at);
      at += 1;
      continue;
    }
    if (plain !== "") {
      pieces.push({ kind: PIECE.text, text: plain });
      plain = "";
    }
    pieces.push(part.piece);
    at += part.length;
  }
  return plain === "" ? pieces : [...pieces, { kind: PIECE.text, text: plain }];
}
