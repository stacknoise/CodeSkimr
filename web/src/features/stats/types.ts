import type { ASSESSMENTS, KINDS, ORDERS, TAB } from "./consts";

/** A view of the screen. */
export type Tab = (typeof TAB)[keyof typeof TAB];
/** A ranking: every tab but the overview. */
export type RankingName = Exclude<Tab, "overview">;
/** The kinds of symbols a ranking holds. */
export type Kinds = (typeof KINDS)[number]["id"];
/** Which count the most used symbols are sorted by. */
export type Order = (typeof ORDERS)[number]["id"];
/** How much "no usage found" says about a symbol. */
export type Assessment = (typeof ASSESSMENTS)[number]["id"];

/** One symbol of a ranking (`RowOut` of crates/app/src/ui_commands/stats.rs). */
export interface Row {
  id: number;
  name: string;
  qualified_name: string;
  kind: string;
  path: string;
  language: string;
  parent: number | null;
  visibility: string | null;
  test: boolean;
  certain: number;
  total: number;
  complexity: number | null;
  complexity_rating: string | null;
  loc: number | null;
  efferent: number | null;
  afferent: number | null;
  assessment: Assessment | null;
}

/** The symbols without usages per assessment. */
export interface UnusedCounts {
  likely: number;
  public: number;
  approximate: number;
  tests_hidden: number;
}

/** The answer of `stats.ranking`. */
export interface RankingAnswer {
  snapshot: number | null;
  rows: Row[];
  total: number;
  unused: UnusedCounts | null;
}

/** One bar of the complexity histogram. */
export interface Bar {
  complexity: number;
  count: number;
}

/** The answer of `stats.overview`. */
export interface Overview {
  snapshot: number | null;
  files: number;
  symbols: number;
  ranked: number;
  references: { certain: number; probable: number };
  unused: UnusedCounts;
  complexity: { limit: number; over_limit: number; histogram: Bar[] };
  top: { most_used: Row[]; unused: Row[]; complexity: Row[]; coupling: Row[] };
}
