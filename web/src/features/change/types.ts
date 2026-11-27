import type { ScanEntry } from "../scan/types";

/** A list of the answer, cut to its limit (`ListedOut` in crates/application/src/compare/wire.rs). */
export interface Listed<T> {
  total: number;
  is_cut: boolean;
  items: T[];
}

/** A symbol of one of the two scans. */
export interface SymbolOut {
  id: number;
  kind: string;
  qualified_name: string;
  signature: string | null;
  visibility: string | null;
  path: string;
}

/** A symbol of both scans: as the base and as the target have it. */
export interface MatchedOut {
  base: SymbolOut;
  target: SymbolOut;
  reasons: string[];
  narrowed: boolean;
}

/** The symbol that holds a usage. */
export interface HolderOut {
  id: number;
  kind: string;
  qualified_name: string;
}

/** One direct usage of a symbol in the base scan. */
export interface UsageOut {
  from: HolderOut;
  path: string;
  line: number;
  column: number;
  kind: string;
  confidence: string;
  status: string;
}

/** A symbol whose usages the change can break, with those usages. */
export interface SymbolImpactOut {
  symbol: SymbolOut;
  cause: string;
  reasons: string[];
  narrowed: boolean;
  usages: Listed<UsageOut>;
  not_touched: number;
}

/** A critical symbol the change touches. */
export interface CriticalOut {
  annotation: number;
  kind: string;
  qualified_name: string;
  outcome: string;
}

/** A dependency between two layers that is new in the target scan. */
export interface LayerEdgeOut {
  from: string;
  to: string;
  count: number;
}

/** The layer rules the target breaks and the base did not. */
export interface LayersOut {
  new_violations: LayerEdgeOut[];
  new_cycles: LayerEdgeOut[];
}

/** The numbers at the head of the answer; the usage counts are `null` without the impact. */
export interface SummaryOut {
  changed_symbols: number;
  public_symbols: number;
  usages_certain: number | null;
  usages_probable: number | null;
  usages_not_touched: number | null;
}

/** The answer of `scans.compare` and `scans.impact` (docs/compare.md, `"schema": 1`). */
export interface ChangeAnswer {
  schema: number;
  project: number;
  base: ScanEntry;
  target: ScanEntry;
  summary: SummaryOut;
  added: Listed<SymbolOut>;
  removed: Listed<SymbolOut>;
  changed: Listed<MatchedOut>;
  moved: Listed<MatchedOut>;
  critical: CriticalOut[];
  layers: LayersOut;
  impact: Listed<SymbolImpactOut> | null;
  is_cut: boolean;
  direct_usages_only: boolean;
}

/** One row of the table of the impact: a symbol the difference removes, changes or adds. */
export interface ImpactRow {
  /** Unique within the answer; names the row for the selection. */
  key: string;
  /** What happened: `removed`, `changed` or `added`. */
  cause: string;
  /** The symbol as the target has it, or as the base has it if it is removed. */
  symbol: SymbolOut;
  /** Why it changed; empty for a removed or new symbol. */
  reasons: string[];
  narrowed: boolean;
  /** The symbol as the base has it, for the old signature. */
  before: SymbolOut | null;
  /** Its usages; `null` if the change cannot break any. */
  impact: SymbolImpactOut | null;
}

/** The two ends of a comparison as the screen asks for them, by the `scan` id of the entry; `null` leaves the choice to the
 * registry (the newest scan against the one before). */
export interface Selection {
  base: number | null;
  target: number | null;
}

/** The answer of `scans.impact_markdown`: the text `codeskimr change --markdown` prints. */
export interface MarkdownAnswer {
  markdown: string;
}
