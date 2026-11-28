import type { SymbolRef } from "../rules/types";

/** A place of a function: file below the project, name, first and last line (counted from 1). */
export interface Place {
  path: string;
  symbol: string;
  line: number;
  end_line: number;
}

/** One finding of the check (the answer of the MCP tool `check_conventions`). Only the measure
 * that belongs to the rule of the finding is set. */
export interface Finding extends Place {
  code: string;
  description: string;
  lines: number | null;
  max_lines: number | null;
  value: number | null;
  max_value: number | null;
  literals: number | null;
  duplicate_of: Place | null;
  similarity_percent: number | null;
  /** The symbol of the newest scan with this qualified name, `null` if the scan has none (a file
   * written after it). */
  symbol_ref: SymbolRef | null;
}

/** A file that could not be analysed, with the code of the reason. */
export interface Skipped {
  path: string;
  code: string;
}

/** What `conventions.check` answers. */
export interface ConventionsOut {
  files_checked: number;
  total: number;
  findings: Finding[];
  unchecked: Skipped[];
  unmatched: string[];
  not_checked: string[];
}
