import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { currentPath } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { activeProject, openProject } from "../../shared/project/active";
import { pointer } from "../../tests/pointer";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { selectType } from "../explorer/explorer";
import { DEPTH_ALL, LEVEL } from "./consts";
import {
  graphLevel,
  resetGraph,
  setGraphDepth,
  setGraphLevel,
  wantedLayer,
  wantLayer,
} from "./graph";
import { GraphScreen } from "./GraphScreen";

const ROW: ProjectRow = {
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

const BOX = { width: 160, height: 44, order: 0, layer: null, types: null, symbol: null };
const GRAPH = {
  snapshot: 5,
  focus: null,
  width: 400,
  height: 300,
  boxes: [
    { name: "Domain", label: "Domain", context: "", x: 120, y: 200, rank: 1, ...BOX },
    { name: "Ui", label: "Ui", context: "", x: 120, y: 0, rank: 0, ...BOX },
    { name: "Infra", label: "Infra", context: "", x: 300, y: 100, rank: 0, ...BOX },
  ],
  bands: [
    { name: "Ui", rank: 0, y: 0, height: 60 },
    { name: "Domain", rank: 1, y: 180, height: 60 },
  ],
  groups: [],
  links: [
    { from: "Ui", to: "Domain", count: 12, violates_layering: false, in_cycle: false },
    { from: "Domain", to: "Infra", count: 2, violates_layering: true, in_cycle: false },
  ],
};

useServerSession();

beforeEach(() => {
  resetGraph();
  setGraphLevel(LEVEL.layers);
  openProject(1);
  setLanguage("en");
});

function served(graph: unknown) {
  serve({ "projects.list": () => answer([ROW]), "graph.layers": () => answer(graph) });
}

describe("GraphScreen", () => {
  it("offers to set layers when the project has none", async () => {
    served({ ...GRAPH, boxes: [], links: [], bands: [] });
    render(() => <GraphScreen />);
    expect(await screen.findByRole("button", { name: "Set layers" })).toBeInTheDocument();
  });

  it("keeps the level buttons when there are no layers, so another level can be chosen", async () => {
    served({ ...GRAPH, boxes: [], links: [], bands: [] });
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Set layers" });
    fireEvent.click(screen.getByRole("button", { name: "Namespaces" }));
    await waitFor(() => expect(graphLevel()).toBe("namespaces"));
    expect(screen.getByRole("button", { name: "Namespaces" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
  });

  it("draws a box per layer with the totals and the breaks of the layering", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    expect(await screen.findByRole("button", { name: "Domain" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ui" })).toBeInTheDocument();
    expect(screen.getByText("3 nodes · 2 edges")).toBeInTheDocument();
    expect(screen.getByText("against layering (1)")).toBeInTheDocument();
  });

  it("draws a band with the name of every layer behind the boxes", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    const stripe = (name: string) =>
      screen.getAllByText(name).find((found) => found.parentElement?.tagName === "DIV")
        ?.parentElement;
    expect(stripe("Domain")?.style.top).toBe("180px");
    expect(stripe("Domain")?.style.height).toBe("60px");
    expect(stripe("Ui")?.style.top).toBe("0px");
  });

  it("does not hide the links of the namespace level behind the violations filter", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    const filter = screen.getByLabelText("violations only") as HTMLInputElement;
    fireEvent.click(filter);
    expect(filter.checked).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Namespaces" }));
    await waitFor(() => expect(filter.disabled).toBe(true));
    expect(filter.checked).toBe(false);
  });

  it("keeps the type focus off until the explorer has a type", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    expect((screen.getByRole("button", { name: "Type focus" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("asks for the type level with the type of the explorer as focus", async () => {
    served(GRAPH);
    await selectType(1, 77);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    fireEvent.click(screen.getByRole("button", { name: "Type focus" }));
    await waitFor(() => {
      const bodies = fetchMock.mock.calls.map((call) => String(call[1]?.body ?? ""));
      expect(bodies.some((b) => b.includes('"level":"types"') && b.includes('"focus":77'))).toBe(
        true,
      );
    });
  });

  it("asks for the namespace level when it is chosen", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    fireEvent.click(screen.getByRole("button", { name: "Namespaces" }));
    await waitFor(() => {
      const bodies = fetchMock.mock.calls.map((call) => String(call[1]?.body ?? ""));
      expect(bodies.some((body) => body.includes('"level":"namespaces"'))).toBe(true);
    });
  });

  it("offers the depth only on the namespace level and asks for it", async () => {
    setGraphLevel(LEVEL.layers);
    setGraphDepth(DEPTH_ALL);
    served(GRAPH);
    render(() => <GraphScreen />);
    await screen.findByRole("button", { name: "Domain" });
    expect(screen.queryByRole("group", { name: "Depth" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Namespaces" }));
    const depth = await screen.findByRole("group", { name: "Depth" });
    fireEvent.click(screen.getByRole("button", { name: "Domain" }));
    expect(screen.getByRole("heading", { name: "Domain" })).toBeInTheDocument();
    fireEvent.click(within(depth).getByRole("button", { name: "2 levels" }));
    await waitFor(() => expect(screen.queryByRole("heading", { name: "Domain" })).toBeNull());
    await waitFor(() => {
      const bodies = fetchMock.mock.calls.map((call) => String(call[1]?.body ?? ""));
      expect(bodies.some((body) => body.includes('"depth":2'))).toBe(true);
    });
    setGraphLevel(LEVEL.layers);
    setGraphDepth(DEPTH_ALL);
  });

  it("shows what a selected layer uses and what uses it", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Domain" }));
    expect(screen.getByRole("heading", { name: "Domain" })).toBeInTheDocument();
    expect(screen.getByText("Uses · 1")).toBeInTheDocument();
    expect(screen.getByText("Used by · 1")).toBeInTheDocument();
    expect(screen.getByText("12 references")).toBeInTheDocument();
    expect(screen.getByText("1 violation in the project:")).toBeInTheDocument();
    expect(screen.getByText(/Domain → Infra \(2\)/)).toBeInTheDocument();
  });

  it("widens the side panel with the keys of its separator", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Domain" }));
    const separator = screen.getByRole("separator");
    const before = Number(separator.getAttribute("aria-valuenow"));
    fireEvent.keyDown(separator, { key: "ArrowLeft" });
    expect(Number(separator.getAttribute("aria-valuenow"))).toBeGreaterThan(before);
  });

  it("selects the layer another screen asked for, once the map is there", async () => {
    served(GRAPH);
    wantLayer("Domain");
    render(() => <GraphScreen />);
    expect(await screen.findByText("Used by · 1")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Domain" })).toBeInTheDocument();
    expect(wantedLayer()).toBeNull();
  });

  it("opens the explorer with the project", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Ui" }));
    fireEvent.click(screen.getByRole("button", { name: /Open in Explorer/ }));
    expect(currentPath()).toBe("/explorer");
    expect(activeProject()).toBe(1);
  });

  it("says that a project without layers has none, and notes missing edges", async () => {
    served({ snapshot: 5, width: 0, height: 0, boxes: [], links: [] });
    render(() => <GraphScreen />);
    expect(await screen.findByText("No layers to show")).toBeInTheDocument();
    resetGraph();
  });

  it("notes that no dependencies were found yet", async () => {
    served({ ...GRAPH, links: [] });
    render(() => <GraphScreen />);
    expect(await screen.findByText(/No dependencies between the layers/)).toBeInTheDocument();
  });

  it("says that one layer alone needs a second one for a dependency map", async () => {
    served({
      ...GRAPH,
      boxes: [GRAPH.boxes[1]],
      bands: [GRAPH.bands[0]],
      links: [],
    });
    render(() => <GraphScreen />);
    expect(await screen.findByText(/needs at least two layers/)).toBeInTheDocument();
    expect(screen.queryByText(/No dependencies between the layers/)).toBeNull();
  });

  it("pans with the primary button on the empty ground but not on a box", async () => {
    served(GRAPH);
    render(() => <GraphScreen />);
    const box = await screen.findByRole("button", { name: /Ui/ });
    const stage = box.parentElement as HTMLElement;
    const area = stage.parentElement as HTMLElement;
    area.setPointerCapture = () => undefined;
    const drag = (target: Element) => {
      const before = stage.style.transform;
      pointer(target, "pointerdown", { button: 0 });
      pointer(area, "pointermove", { movementX: 30, movementY: 20 });
      pointer(area, "pointerup");
      return stage.style.transform !== before;
    };
    expect(drag(box)).toBe(false);
    expect(drag(area)).toBe(true);
  });

  it("draws the frames around namespaces and the short label in the boxes", async () => {
    setGraphLevel(LEVEL.namespaces);
    served({
      ...GRAPH,
      boxes: [{ ...GRAPH.boxes[1]!, name: "Shop.Cart", label: "Cart", context: "Shop" }],
      bands: [],
      links: [],
      groups: [{ name: "Shop", label: "Shop", x: 0, y: 0, width: 200, height: 100, depth: 0 }],
    });
    render(() => <GraphScreen />);
    expect((await screen.findAllByText("Shop")).length).toBe(2);
    expect(screen.getByRole("button", { name: /^Cart/ })).toHaveAttribute("aria-pressed");
  });

  it("is written in German too", async () => {
    setLanguage("de");
    served(GRAPH);
    render(() => <GraphScreen />);
    expect(await screen.findByText("alle Verbindungen schwach")).toBeInTheDocument();
    expect(screen.getByText("3 Knoten · 2 Kanten")).toBeInTheDocument();
  });
});
