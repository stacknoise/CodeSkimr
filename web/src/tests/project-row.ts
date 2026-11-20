import type { ProjectRow } from "../features/projects/types";

/** A registered folder project that was not scanned yet, for the tests of the screens. */
export const PROJECT_ROW: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: null,
  types: 3,
  loc: 10,
  languages: [],
  running: false,
};
