import type { Outline, SymbolInfo } from "../features/explorer/types";
import type { GraphData } from "../features/graph/types";
import type { Summary, UsagesAnswer } from "../features/usages/types";

/** What the export says about the project; it names no folder or address. */
export interface ExportProject {
  id: number;
  name: string;
  commit: string | null;
  /** Seconds since the Unix epoch. */
  created: number;
  mode: "full" | "incremental";
  files: number;
  symbols: number;
  references: number;
}

/** The usages of one symbol as the export holds them. */
export interface StoredUsages {
  /** Both directions, as deep as the configuration allows, tests included. */
  answer: UsagesAnswer;
  /** The direct usages when tests are left out. */
  summary_without_tests: Summary;
}

/** The data block of the page (`Data` in crates/app/src/ui_commands/export.rs). */
export interface ExportData {
  format: number;
  snapshot: number;
  project: ExportProject;
  outline: Outline;
  symbols: Record<string, SymbolInfo>;
  usages: Record<string, StoredUsages>;
  graph: GraphData;
}
