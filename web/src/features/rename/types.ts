import type { SITE_KIND, TEXT_KIND } from "./consts";

/** A symbol in an answer (`PlaceRow` in crates/application/src/rename_rows.rs). */
export interface Place {
  id: number;
  qualified_name: string;
  kind: string;
}

/** One place of the code that would change (`SiteRow` in crates/application/src/rename_rows.rs). */
export interface Site {
  kind: (typeof SITE_KIND)[keyof typeof SITE_KIND];
  path: string;
  line: number;
  column: number;
  confidence: string | null;
  verified: boolean;
  from: Place | null;
  layer: string | null;
  /** The line of code the place stands on; `null` if the folder was not read. */
  code: string | null;
}

/** A symbol that already has the new name. */
export interface Conflict extends Place {
  path: string;
  signature: string | null;
  same_signature: boolean;
}

/** One line that holds the old name as text. */
export interface TextHit {
  path: string;
  line: number;
  text: string;
  kind: (typeof TEXT_KIND)[keyof typeof TEXT_KIND];
}

/** The answer of `rename.preview` (`PreviewOut`). */
export interface Preview {
  snapshot: number;
  symbol: Place;
  old_name: string;
  new_name: string;
  public_interface: boolean;
  preliminary: boolean;
  sites_total: number;
  verified: number;
  files: number;
  layers: { layer: string; count: number }[];
  sites: Site[];
  conflicts: Conflict[];
  text_searched: boolean;
  text_total: number;
  text_skipped: number;
  text_hits: TextHit[];
}
