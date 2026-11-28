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
  /** The symbol `from` names in the newest scan, `null` if the scan has none of that name. */
  from_symbol: SymbolRef | null;
  to_symbol: SymbolRef | null;
}

/** A symbol of the newest scan, as the explorer opens it. */
export interface SymbolRef {
  id: number;
  kind: string;
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
  /** The version of the project file as it was read; sent back with a change (CFG-106). */
  file_version: string;
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

/** One entry of `[[rules.forbid]]` as `rules.set` carries it. */
export interface ForbidInput {
  from: string;
  to: string;
  reason?: string;
}

/** One entry of `[[rules.public_api]]` as `rules.set` carries it. */
export interface PublicApiInput {
  module: string;
  internal: string[];
}

/** What one layer may use; `null` is no limit. */
export interface LimitInput {
  layer: string;
  may_use: string[] | null;
}

/** The input of `rules.set` and `rules.preview`: the rules the project file gets as a whole. */
export interface RulesInput {
  project: number;
  cycles: boolean;
  forbid: ForbidInput[];
  public_api: PublicApiInput[];
  limits: LimitInput[];
  /** The version `rules.show` answered; a file that changed since is refused (CFG-106). */
  file_version: string;
}

/** The answer of `rules.set`. */
export interface SavedOut {
  rules: number;
}

/** What the layer editor knows that the rules dialogs need: the layers and the namespaces. */
export interface LayersOut {
  layers: { name: string }[];
  namespaces: { name: string }[];
}
