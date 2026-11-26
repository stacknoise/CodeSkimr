/** One place of a violation; line and column start at 1 (crates/app/src/ui_commands/rules.rs). */
export interface PlaceOut {
  path: string;
  line: number;
  column: number;
}

/** The ring of modules behind a cycle violation. */
export interface RingOut {
  modules: string[];
  chain: string[];
  weakest_count: number;
  resolution_edges: number;
}

/** One violation: the symbol that refers and the one it must not reach. For a cycle, `from` and
 * `to` are the ends of the weakest edge of the ring and `places` are its references. */
export interface ViolationOut {
  from: string;
  to: string;
  old: boolean;
  places: PlaceOut[];
  ring: RingOut | null;
}

/** One rule with its violations. */
export interface RuleOut {
  id: string;
  kind: "cycles" | "forbid" | "public_api";
  from: string;
  to: string;
  internal: string[];
  reason: string | null;
  violations: ViolationOut[];
  new: number;
}

/** The answer of `rules.show`. */
export interface RulesOut {
  scanned: boolean;
  cycles: boolean;
  held_on: string | null;
  rules: RuleOut[];
  resolved: number;
  old: number;
  new: number;
}
