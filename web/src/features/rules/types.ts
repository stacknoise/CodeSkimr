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

/** What one layer that limits its use may use. */
export interface AllowedOut {
  layer: string;
  may_use: string[];
}

/** One rule with its violations. */
export interface RuleOut {
  id: string;
  kind: "layers" | "cycles" | "forbid" | "public_api";
  from: string;
  to: string;
  internal: string[];
  reason: string | null;
  allowed: AllowedOut[];
  violations: ViolationOut[];
  new: number;
}

/** The answer of `rules.show`. */
export interface RulesOut {
  scanned: boolean;
  holdable: boolean;
  cycles: boolean;
  held_on: string | null;
  rules: RuleOut[];
  resolved: number;
  old: number;
  new: number;
}

/** The answer of `rules.hold`: the day written into the file and how many violations it holds. */
export interface HeldOut {
  held_on: string;
  entries: number;
}

/** The answer of `project_file.open`: the file that was opened and whether it was created. */
export interface OpenedOut {
  file: string;
  created: boolean;
}
