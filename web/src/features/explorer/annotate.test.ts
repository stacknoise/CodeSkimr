import { describe, expect, it } from "vitest";
import { memberTarget } from "./annotate";
import type { MemberRow } from "./types";

const member: MemberRow = {
  id: 1,
  name: "Work",
  qualified_name: "Shop.Order.Work",
  kind: "method",
  visibility: "public",
  signature: "void Work(int x)",
  parameters: [],
  complexity: null,
  complexity_rating: null,
  usages: 0,
  first_line: 1,
  last_line: 2,
};

describe("the target of a member", () => {
  it("names no parameters for a member without overloads", () => {
    expect(memberTarget(member)).toEqual({ kind: "method", name: "Shop.Order.Work" });
  });

  it("carries the parameters of an overloaded member, so that its annotations are its own", () => {
    expect(memberTarget({ ...member, parameters: ["int x"] })).toEqual({
      kind: "method",
      name: "Shop.Order.Work",
      parameters: ["int x"],
    });
  });
});
