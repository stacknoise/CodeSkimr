import type { ScanReport } from "../../shared/project/report";
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
  /** Where the tool was found; only the listing of the settings knows it. */
  path?: string | null;
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

/** A file the newest scan left out because it alone could not be analysed (ADR 0085). */
export interface LeftOutFile {
  path: string;
  /** The registered code, for example `PRS-101`. */
  code: string;
  /** Whether the file is shown as the previous scan saw it; otherwise it is missing. */
  kept: boolean;
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
  /** The project is being removed: its data goes in many short steps. */
  removing?: boolean;
  /** The removal stopped before it was done; asking for it again goes on. */
  interrupted?: boolean;
  /** The files the newest scan left out, as far as it names them; absent in an export. */
  left_out?: LeftOutFile[];
  /** How many more files it left out than `left_out` names. */
  left_out_more?: number;
  /** The last check of the tools, `null` if the project was never checked (absent in an export,
   * which does not run on the machine of the reader). */
  tools?: ToolsInfo | null;
  /** What the last scan was and what level 2 did (ADR 0072); `null` while no scan stored a report,
   * absent in an export. */
  report?: ScanReport | null;
}

/** Whether a scan from the browser started stage 2 (ADR 0072, decision 4). */
export interface Level2Start {
  state: "started" | "not_started" | "not_requested";
  /** Why it did not start: `not_permitted`, `not_confirmed`, `busy` or `paused`. */
  reason?: string | null;
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
  level2?: Level2Start;
}

/** The state of the level 2 switch of a project (`level2.status`, ADR 0071, decision 3). */
export interface Level2Status {
  state: "on" | "off" | "not_enabled" | "locked";
  /** Whether a confirmation exists; only informative if the state is `not_enabled`. */
  confirmed: boolean;
  /** `window` if the server can open the console window, `copy` if the screen shows the command. */
  launch: "window" | "copy";
}

/** The answer of `POST /api/v1/level2/confirm-window`. */
export interface ConfirmWindow {
  launch: "window" | "copy";
  command?: string;
}
