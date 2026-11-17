import type { AnnotationKind, PieceKind, TaskStatus } from "./consts";

/** What an annotation is about: the kind and the qualified name of a symbol and, for a member
 * with overloads, the parameter types of its signature (ADR 0068, decision 3). */
export interface Target {
  kind: string;
  name: string;
  parameters?: string[];
}

/** One annotation as the server answers it (`annotations.list`). */
export interface Annotation {
  id: number;
  project: number;
  target: Target;
  kind: AnnotationKind;
  title?: string | null;
  text?: string | null;
  status?: TaskStatus | null;
  /** Due date of a task, in seconds since 1970. */
  due?: number | null;
  assignee?: string | null;
  author: string;
  created: number;
  updated: number;
  /** The symbol is not in the newest snapshot any more. */
  without_target: boolean;
}

/** A page of an answer. */
export interface Page<Item> {
  items: Item[];
  total: number;
  offset: number;
  limit: number;
  next_offset: number | null;
}

/** One row of `annotations.summary`: the counters of one annotated symbol. */
export interface SummaryRow {
  target: Target;
  critical: boolean;
  notes: number;
  tasks_done: number;
  tasks_total: number;
  without_target: boolean;
}

/** The counters of one symbol, as the UI joins them to a row. */
export interface Counters {
  critical: boolean;
  notes: number;
  tasksDone: number;
  tasksTotal: number;
}

/** The fields of a new or changed note or task. `null` clears a field of an update. */
export interface Draft {
  /** Assigns the annotation to another symbol (an update only). */
  target?: Target;
  title?: string;
  text?: string | null;
  status?: string;
  due?: number | null;
  assignee?: string | null;
}

/** A piece of a text in the Markdown subset: plain, bold, italic, code or a link. */
export interface Piece {
  kind: PieceKind;
  text: string;
  /** The address of a link; only `http` and `https`. */
  href?: string;
}
