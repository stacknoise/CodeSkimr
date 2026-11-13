import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { currentPath } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { activeProject, openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { resetGraph, wantedLayer, wantLayer } from "./graph";
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

const BOX = { width: 160, height: 44, order: 0, layer: null, types: null };
const GRAPH = {
  snapshot: 5,
  width: 400,
  height: 300,
  boxes: [
    { name: "Domain", x: 120, y: 200, rank: 1, ...BOX },
    { name: "Ui", x: 120, y: 0, rank: 0, ...BOX },
    { name: "Infra", x: 300, y: 100, rank: 0, ...BOX },
  ],
  bands: [
    { name: "Ui", rank: 0, y: 0, height: 60 },
    { name: "Domain", rank: 1, y: 180, height: 60 },
  ],
  links: [
    { from: "Ui", to: "Domain", count: 12, violates_layering: false, in_cycle: false },
    { from: "Domain", to: "Infra", count: 2, violates_layering: true, in_cycle: false },
  ],
};

useServerSession();

beforeEach(() => {
  resetGraph();
  openProject(1);
  setLanguage("en");
});

function served(graph: unknown) {
  serve({ "projects.list": () => answer([ROW]), "graph.layers": () => answer(graph) });
}

describe("GraphScreen", () => {
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

  it("is written in German too", async () => {
    setLanguage("de");
    served(GRAPH);
    render(() => <GraphScreen />);
    expect(await screen.findByText("alle Verbindungen schwach")).toBeInTheDocument();
    expect(screen.getByText("3 Knoten · 2 Kanten")).toBeInTheDocument();
  });
});
