import type { ProjectRow } from "../features/projects/types";

/** A project row of the list, as `projects.list` answers it; `changes` overrides single fields. */
export function projectRow(changes: Partial<ProjectRow> = {}): ProjectRow {
  return {
    id: 4,
    name: "shop",
    kind: "folder",
    location: "/work/shop",
    branch: null,
    commit: null,
    scan: null,
    types: 0,
    loc: 0,
    languages: [],
    running: false,
    tools: null,
    ...changes,
  };
}
