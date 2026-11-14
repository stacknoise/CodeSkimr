import type { SCAN_MODE, SOURCE_KIND } from "./consts";

/** The newest scan of a project. */
export interface ScanInfo {
  snapshot: number;
  /** Seconds since the Unix epoch. */
  created: number;
  mode: (typeof SCAN_MODE)[keyof typeof SCAN_MODE];
  files: number;
  symbols: number;
  references: number;
}

/** A language of a project. */
export interface LanguageInfo {
  language: string;
  files: number;
  loc: number;
  symbols: number;
  /** Tier of the language pack; `null` if no pack of that id is installed. */
  tier: "full" | "basic" | null;
}

/** One row of `projects.list`. */
export interface ProjectRow {
  id: number;
  name: string;
  kind: (typeof SOURCE_KIND)[keyof typeof SOURCE_KIND];
  location: string;
  branch: string | null;
  commit: string | null;
  scan: ScanInfo | null;
  types: number;
  loc: number;
  languages: LanguageInfo[];
  running: boolean;
}

/** The output of `projects.add` and `projects.rescan`. */
export interface Scanned {
  project: number;
  snapshot: number;
  files: number;
  symbols: number;
  references: number;
  seconds: number;
}
