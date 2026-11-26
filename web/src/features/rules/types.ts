/** One place of a violation; line and column start at 1 (crates/app/src/ui_commands/rules.rs). */
export interface PlaceOut {
  path: string;
  line: number;
  column: number;
}

/** One violation: the symbol that refers and the one it must not reach. */
export interface ViolationOut {
  from: string;
  to: string;
  old: boolean;
  places: PlaceOut[];
}

/** One rule with its violations. */
export interface RuleOut {
  id: string;
  kind: "forbid" | "public_api";
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
