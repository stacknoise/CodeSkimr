import {
  BADGE,
  CONFIDENCE_PROBABLE,
  JSON_INDENT,
  LIST_LINE_END,
  LIST_SEPARATOR,
  PLACE_SEPARATOR,
  SITE_KIND,
} from "./consts";
import type { Preview, Site } from "./types";

/** The badge of a place. */
export type SiteBadge = (typeof BADGE)[keyof typeof BADGE];

/** How sure a place is, as the key of the badge: declarations are facts of the syntax, a verified
 * usage is sure, a usage the resolver guessed is probable, any other one is preliminary. */
export function siteBadge(site: Site): SiteBadge {
  if (site.kind === SITE_KIND.declaration) {
    return BADGE.declaration;
  }
  if (site.verified) {
    return BADGE.sure;
  }
  return site.confidence === CONFIDENCE_PROBABLE ? BADGE.probable : BADGE.unchecked;
}

/** Path and line of a place, as an editor takes them. */
export function placeText(path: string, line: number): string {
  return `${path}${PLACE_SEPARATOR}${line}`;
}

/** The list of places and text hits as plain text, one place per line, for the clipboard. */
export function listText(preview: Preview): string {
  const sites = preview.sites.map((site) =>
    [placeText(site.path, site.line), siteBadge(site)].join(LIST_SEPARATOR),
  );
  const texts = preview.text_hits.map((hit) =>
    [placeText(hit.path, hit.line), hit.kind, hit.text].join(LIST_SEPARATOR),
  );
  return [...sites, ...texts].join(LIST_LINE_END);
}

/** The whole answer as JSON, as the console prints it with `--json`. */
export function jsonText(preview: Preview): string {
  return JSON.stringify(preview, null, JSON_INDENT);
}
