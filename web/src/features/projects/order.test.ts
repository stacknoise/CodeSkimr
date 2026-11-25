import { describe, expect, it } from "vitest";
import { byName } from "./order";
import type { ProjectRow } from "./types";

const row = (id: number, name: string) => ({ id, name }) as ProjectRow;

describe("byName", () => {
  it("sorts by name without regard to case and counts digits as numbers", () => {
    const sorted = byName([row(1, "zeta"), row(2, "Alpha"), row(3, "app 10"), row(4, "App 2")]);
    expect(sorted.map((r) => r.name)).toEqual(["Alpha", "App 2", "app 10", "zeta"]);
  });

  it("keeps projects with the same name in the order of their ids", () => {
    const sorted = byName([row(7, "Same"), row(3, "same"), row(5, "SAME")]);
    expect(sorted.map((r) => r.id)).toEqual([3, 5, 7]);
  });

  it("does not change the given list", () => {
    const rows = [row(2, "b"), row(1, "a")];
    byName(rows);
    expect(rows.map((r) => r.id)).toEqual([2, 1]);
  });
});
