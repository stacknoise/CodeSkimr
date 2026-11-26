import { describe, expect, it } from "vitest";
import { overlapOf } from "./overlap";
import type { ProjectRow } from "./types";

const row = (id: number, location: string, kind = "folder"): ProjectRow =>
  ({ id, name: `p${id}`, kind, location }) as ProjectRow;

describe("overlapOf", () => {
  it("finds the project that holds the folder, with the folder inside it", () => {
    const found = overlapOf("/work/portal/cli/", [row(1, "/work/portal")]);
    expect(found?.project.id).toBe(1);
    expect(found?.folder).toBe("cli");
  });

  it("takes the innermost project and ignores the case and the kind of slash on Windows", () => {
    const rows = [row(1, "C:\\Work\\Portal"), row(2, "C:\\Work\\Portal\\Api")];
    expect(overlapOf("c:/work/portal/api/src", rows)?.project.id).toBe(2);
  });

  it("finds nothing for the project folder itself, a neighbour or a Git project", () => {
    expect(overlapOf("/work/portal", [row(1, "/work/portal")])).toBeNull();
    expect(overlapOf("/work/portal-2/cli", [row(1, "/work/portal")])).toBeNull();
    expect(overlapOf("/work/portal/cli", [row(1, "/work/portal", "git")])).toBeNull();
  });
});
