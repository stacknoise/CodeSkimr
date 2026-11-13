/** A symbol on the screen. */
export interface Place {
  id: number;
  qualified_name: string;
  kind: string;
  layer: string | null;
}

/** One place where the symbol is used. */
export interface Hit {
  depth: number;
  kind: string;
  confidence: string;
  /** Whether level 2 produced the reference. */
  verified: boolean;
  path: string;
  line: number;
  from: Place;
  to: Place;
}

/** The references of one direction. */
export interface Walk {
  total: number;
  hits: Hit[];
}

export interface Summary {
  places: number;
  files: number;
  layers: number;
  verified: number;
  unchecked: number;
  probable: number;
}

export interface Impact {
  total: number;
  public_interfaces: number;
  layers: { layer: string; count: number }[];
}

/** The answer of `usages.query`. */
export interface UsagesAnswer {
  snapshot: number;
  symbol: Place;
  signature: string | null;
  path: string;
  depth: number;
  incoming: Walk | null;
  outgoing: Walk | null;
  summary: Summary;
  impact: Impact;
  /** `unavailable` while no indexer exists for the project. */
  level2: string;
}
