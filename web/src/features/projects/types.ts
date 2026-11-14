import type { SCAN_MODE, SOURCE_KIND, TOOL_STATE } from "./consts";

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
  /** Microseconds the last full scan spent parsing the language; `null` if not recorded. */
  parse_micros: number | null;
  /** Tier of the language pack; `null` if no pack of that id is installed. */
  tier: "full" | "basic" | null;
}

/** One tool that level 2 needs, and the languages of the project that need it. */
export interface ToolInfo {
  tool: string;
  role: "indexer" | "toolchain";
  state: (typeof TOOL_STATE)[keyof typeof TOOL_STATE];
  languages: string[];
  found: string | null;
  required: string | null;
  /** Message keys in the Fluent files of `locales/`, named by the language pack. */
  purpose_key: string;
  install_key: string;
}

/** What the check of the tools found: the output of `projects.tools`. */
export interface ToolsInfo {
  /** Seconds since the Unix epoch; `null` if nothing was checked. */
  checked_at: number | null;
  languages: { language: string; level2_ready: boolean }[];
  tools: ToolInfo[];
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
  /** The last check of the tools, `null` if the project was never checked (absent in an export,
   * which does not run on the machine of the reader). */
  tools?: ToolsInfo | null;
}

/** The output of `projects.add` and `projects.rescan`. */
export interface Scanned {
  project: number;
  snapshot: number;
  files: number;
  symbols: number;
  references: number;
  seconds: number;
  tools: ToolsInfo;
}
