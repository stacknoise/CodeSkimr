import { NONE, PREFIXES, type Filter } from "./consts";

/** What the input asks for: a filter (from a prefix, else the chip) and the text to look for. */
export function parseQuery(raw: string, chip: Filter): { filter: Filter; text: string } {
  const found = PREFIXES.find(({ prefix }) => raw.startsWith(prefix));
  if (found) {
    return { filter: found.filter, text: raw.slice(found.prefix.length).trim() };
  }
  return { filter: chip, text: raw.trim() };
}

/** A name split around the first place where the text occurs in it (any case). */
export function splitMatch(name: string, text: string): { pre: string; hit: string; post: string } {
  const at = text === "" ? NONE : name.toLowerCase().indexOf(text.toLowerCase());
  if (at === NONE) {
    return { pre: name, hit: "", post: "" };
  }
  return {
    pre: name.slice(0, at),
    hit: name.slice(at, at + text.length),
    post: name.slice(at + text.length),
  };
}
