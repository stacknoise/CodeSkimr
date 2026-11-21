import type { Outline, SymbolInfo } from "../features/explorer/types";
import type { GraphData } from "../features/graph/types";
import type { Hit, Place, Summary, UsagesAnswer } from "../features/usages/types";

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

/** A hit as the export holds it: the places are named by id (`ExportData.places`). */
export type StoredHit = Omit<Hit, "from" | "to"> & { from: number; to: number };

/** The usages of one symbol as the export holds them. */
export interface StoredUsages {
  /** Both directions, the direct usages only, tests included. */
  answer: Omit<UsagesAnswer, "incoming" | "outgoing"> & {
    incoming: { total: number; hits: StoredHit[] } | null;
    outgoing: { total: number; hits: StoredHit[] } | null;
  };
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
  /** Every place that a hit names, once. */
  places: Record<string, Place>;
  usages: Record<string, StoredUsages>;
  graph: GraphData;
}
