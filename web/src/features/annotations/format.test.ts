import { describe, expect, it } from "vitest";
import { dateFieldOf, dueDate, dueOfField, isLate, shortDate } from "./format";

describe("due dates", () => {
  it("keeps the day of a date field through a round trip, in every time zone", () => {
    const due = dueOfField("2026-11-20");
    expect(due).not.toBeNull();
    expect(dateFieldOf(due)).toBe("2026-11-20");
    expect(dueDate(due ?? 0, "en")).toBe("11/20/2026");
    expect(dueDate(due ?? 0, "de")).toBe("20.11.2026");
  });

  it("has no due date for an empty or invalid field", () => {
    expect(dueOfField("")).toBeNull();
    expect(dueOfField("not a date")).toBeNull();
    expect(dateFieldOf(null)).toBe("");
    expect(dateFieldOf(undefined)).toBe("");
  });

  it("is late from the day after the due day on", () => {
    const due = dueOfField("2026-11-20") ?? 0;
    const sameDay = Date.parse("2026-11-20T23:00:00Z") / 1000;
    const nextDay = Date.parse("2026-11-21T01:00:00Z") / 1000;
    expect(isLate(due, sameDay)).toBe(false);
    expect(isLate(due, nextDay)).toBe(true);
  });

  it("writes a time as a short date in the language", () => {
    const noon = Date.parse("2026-11-12T12:00:00Z") / 1000;
    expect(shortDate(noon, "de")).toBe("12.11.2026");
  });
});
