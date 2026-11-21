/** Ids of the commands of the layer editor (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  get: "layers.get",
  set: "layers.set",
} as const;

/** Where a qualified name is split into segments: `::` of Rust paths or `.` of namespaces. */
export const SEPARATOR = /::|\./g;
/** What follows a prefix when a name lies below it. */
export const SEPARATOR_AT_START = /^(?:::|\.)/;

/** Trees with at most this many nodes are shown opened; larger ones only show the first level. */
export const OPEN_ALL_LIMIT = 60;

/** The custom property that carries the depth of a row to its style sheet. */
export const DEPTH_PROPERTY = "--depth";

/** How far a layer moves in the list. */
export const MOVE = { up: -1, down: 1 } as const;

/** Ids of fields, so that labels and tests find them. */
export const NEW_NAME_FIELD_ID = "layers-new-name";

/** The marks of a row that can be opened. */
export const OPENER = { open: "▾", closed: "▸" } as const;

/** The mark that takes a namespace out of a layer. */
export const REMOVE_MARK = "×";
/** The marks of the buttons that move a layer. */
export const MOVE_MARK = { up: "↑", down: "↓" } as const;
