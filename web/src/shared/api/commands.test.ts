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

  it("measures the layer map only on the level of the layers", async () => {
    serve({ "graph.layers": () => answer({}) });
    const clock = vi.spyOn(performance, "now");
    clock.mockReturnValue(100);
    await runCommand("graph.layers", { project: 1, level: "namespaces" });
    await runCommand("graph.layers", { project: 1, level: "types", focus: 3 });
    expect(measuredTimes()).toEqual({});
    await runCommand("graph.layers", { project: 1 });
    expect(measuredTimes()).toEqual({ graph_layout: 0 });
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
