import { beforeEach, describe, expect, it, vi } from "vitest";

const run = vi.hoisted(() => vi.fn());
const go = vi.hoisted(() => vi.fn());

vi.mock("../../shared/api/commands", () => ({ runCommand: run }));
vi.mock("../../app/router", () => ({ navigate: go }));

import { GRAPH_ROUTE } from "../../app/consts";
import { LEVEL } from "./consts";
import { graphLevel, setGraphLevel, wantedLayer, wantLayer } from "./graph";
import { showInGraph } from "./show";

describe("showInGraph", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    wantLayer(null);
    setGraphLevel(LEVEL.namespaces);
  });

  it("asks the map to select the layer of the symbol and opens the map", async () => {
    run.mockResolvedValue({ layer: "Core" });
    await showInGraph(7, 3);
    expect(wantedLayer()).toBe("Core");
    expect(graphLevel()).toBe(LEVEL.layers);
    expect(go).toHaveBeenCalledWith(GRAPH_ROUTE.path);
  });

  it("shows nothing for a symbol that no layer owns or that cannot be read", async () => {
    run.mockResolvedValue({ layer: null });
    await showInGraph(7, 3);
    run.mockRejectedValue(new Error("MCP-202"));
    await showInGraph(7, 3);
    expect(go).not.toHaveBeenCalled();
    expect(wantedLayer()).toBeNull();
  });
});
