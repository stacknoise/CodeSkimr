import { describe, expect, it } from "vitest";
import { jsonText, listText, placeText, siteBadge } from "./format";
import { PREVIEW, USAGE } from "../../tests/rename";

describe("the list of a rename preview", () => {
  it("tells a declaration, a sure, a probable and a preliminary place apart", () => {
    expect(siteBadge({ ...USAGE, kind: "declaration" })).toBe("declaration");
    expect(siteBadge(USAGE)).toBe("sure");
    expect(siteBadge({ ...USAGE, verified: false, confidence: "probable" })).toBe("probable");
    expect(siteBadge({ ...USAGE, verified: false })).toBe("unchecked");
  });

  it("names a place as path and line", () => {
    expect(placeText("a/b.cs", 4)).toBe("a/b.cs:4");
  });

  it("copies places and text hits as one line each", () => {
    expect(listText(PREVIEW).split("\n")).toEqual([
      "Orders/IOrders.cs:12\tdeclaration",
      "Api/Orders.cs:41\tprobable",
      "docs/api.md:3\tdocumentation\tcalls PlaceOrder",
    ]);
  });

  it("copies the whole answer as JSON", () => {
    expect(JSON.parse(jsonText(PREVIEW))).toEqual(PREVIEW);
  });
});
