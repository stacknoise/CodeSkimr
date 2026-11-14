import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { runCommand } from "./commands";
import { lastQueryMs, measuredTimes, resetTimings } from "./timings";

useServerSession();

beforeEach(() => resetTimings());
afterEach(() => vi.restoreAllMocks());

describe("runCommand timings", () => {
  it("remembers the round trip of a command that measures a budget", async () => {
    serve({ "search.query": () => answer({}) });
    const clock = vi.spyOn(performance, "now");
    clock.mockReturnValueOnce(100).mockReturnValueOnce(104);
    await runCommand("search.query", { query: "a" });
    expect(measuredTimes()).toEqual({ search_keystroke: 4 });
    expect(lastQueryMs()).toBe(4);
  });

  it("measures nothing for other commands and for failed calls", async () => {
    serve({ "projects.list": () => answer([]) });
    await runCommand("projects.list");
    await expect(runCommand("usages.query", {})).rejects.toBeDefined();
    expect(measuredTimes()).toEqual({});
    expect(lastQueryMs()).toBeNull();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});
