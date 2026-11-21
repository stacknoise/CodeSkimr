import type { Target } from "../annotations/types";
import type { AlertKind, AlertState, ChangeKind } from "./consts";

/** One alert as `alerts.list` answers it (`AlertOut` in crates/app/src/ui_commands). */
export interface Alert {
  id: number;
  /** Id of the critical annotation the alert is about. */
  annotation: number;
  target: Target;
  /** The reason of the critical mark. */
  reason: string | null;
  kind: AlertKind;
  state: AlertState;
  /** How often the symbol was seen changed while the alert was open. */
  counter: number;
  /** Seconds since 1970. */
  first_seen: number;
  last_seen: number;
  dismissed: number | null;
  snapshot: number;
  commit: string | null;
  trigger: string;
  changes: ChangeKind[];
  /** For a removed symbol: how many places still refer to it (provisional). */
  references: number | null;
  /** For an orphaned target: the closest symbol of the same name. */
  closest: string | null;
}

/** A page of alerts and the number of open alerts of the project. */
export interface AlertPage {
  items: Alert[];
  total: number;
  offset: number;
  limit: number;
  next_offset: number | null;
  open: number;
}

/** What a dismissal answers. */
export interface Dismissed {
  dismissed: number;
  open: number;
}
