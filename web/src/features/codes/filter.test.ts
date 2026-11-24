import { describe, expect, it } from "vitest";
import { ERROR_CODES, type ErrorCodeInfo } from "../../shared/api/error-codes.generated";
import { GROUP_ALL, SEVERITY_ALL } from "./consts";
import { allCodes, filterCodes, groupsOf } from "./filter";

const ALL = { query: "", severity: SEVERITY_ALL, group: GROUP_ALL };
const message = (code: ErrorCodeInfo) => `message of ${code.id}`;
const retired = () => allCodes().find((code) => code.deprecated);

describe("filterCodes", () => {
  it("hides the retired codes by default", () => {
    const shown = filterCodes(allCodes(), ALL, message);
    expect(shown.some((code) => code.deprecated)).toBe(false);
    expect(shown.length).toBeGreaterThan(0);
  });

  it("shows a retired code when its id is typed in full", () => {
    const code = retired();
    expect(code).toBeDefined();
    const shown = filterCodes(allCodes(), { ...ALL, query: code?.id.toLowerCase() ?? "" }, message);
    expect(shown.map((entry) => entry.id)).toEqual([code?.id]);
  });

  it("finds an id by a part of it, in any case", () => {
    const shown = filterCodes(allCodes(), { ...ALL, query: "src-2" }, message);
    expect(shown.map((code) => code.id)).toContain("SRC-201");
    expect(shown.every((code) => code.id.startsWith("SRC-2"))).toBe(true);
  });

  it("finds a code by its message", () => {
    const shown = filterCodes(allCodes(), { ...ALL, query: "MESSAGE OF PRJ-302" }, message);
    expect(shown.map((code) => code.id)).toEqual(["PRJ-302"]);
  });

  it("filters by severity and by group", () => {
    const warnings = filterCodes(allCodes(), { ...ALL, severity: "warning" }, message);
    expect(warnings.every((code) => code.severity === "warning")).toBe(true);
    const sources = filterCodes(allCodes(), { ...ALL, group: "SRC" }, message);
    expect(sources.every((code) => code.group === "SRC")).toBe(true);
  });
});

describe("groupsOf", () => {
  it("lists each group once, sorted", () => {
    const groups = groupsOf(Object.values(ERROR_CODES));
    expect(groups).toEqual([...new Set(groups)].sort());
    expect(groups).toContain("SRC");
  });
});
