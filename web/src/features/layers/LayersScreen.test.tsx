import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, useServerSession } from "../../tests/commands";
import { PROJECT_ROW } from "../../tests/project-row";
import { resetLayers } from "./layers";
import { LayersScreen } from "./LayersScreen";
import type { LayersData } from "./types";

const SCANNED = {
  project: 1,
  snapshot: 2,
  files: 1,
  symbols: 1,
  references: 0,
  seconds: 0.1,
  tools: { checked: 0, tools: [] },
};
const NAMESPACES = [
  { name: "Shop.Domain", types: 3, layer: null },
  { name: "Shop.Ui", types: 2, layer: null },
];

useServerSession();

beforeEach(() => {
  resetLayers();
  openProject(1);
  setLanguage("en");
});

/** A server that keeps the layers it was given and shows them after the next read. */
function server(first: Partial<LayersData> = {}) {
  let current: LayersData = {
    editable: true,
    scanned: true,
    layers: [],
    namespaces: NAMESPACES,
    file_version: "5",
    ...first,
  };
  fetchMock.mockImplementation((path: string, init?: RequestInit) => {
    const id = path.replace("/api/v1/commands/", "");
    if (id === "layers.set") {
      const sent = JSON.parse(String(init?.body)) as { layers: LayersData["layers"] };
      current = { ...current, layers: sent.layers };
      return Promise.resolve(answer({ layers: sent.layers.length }));
    }
    return Promise.resolve(
      id === "layers.get"
        ? answer(current)
        : id === "projects.list"
          ? answer([PROJECT_ROW])
          : id === "projects.rescan"
            ? answer(SCANNED)
            : answer({ snapshot: 2, layers: [], thresholds: {} }),
    );
  });
}

describe("LayersScreen", () => {
  it("builds a layer from the ticked namespaces and saves it with a new scan", async () => {
    server();
    render(() => <LayersScreen />);
    fireEvent.click(await screen.findByRole("checkbox", { name: "Select Shop.Domain" }));
    fireEvent.input(screen.getByLabelText("Name of the new layer"), {
      target: { value: "domain" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create layer from selection" }));
    expect(screen.getByText("Shop.Domain", { selector: "span" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Save and scan again" }));
    expect(await screen.findByText("Saved. The new scan is done.")).toBeInTheDocument();
    const calls = fetchMock.mock.calls.map(([path]) => String(path));
    expect(calls.some((path) => path.endsWith("layers.set"))).toBe(true);
    const sent = fetchMock.mock.calls.find(([path]) => String(path).endsWith("layers.set"));
    expect(JSON.parse((sent?.[1] as { body: string }).body)).toMatchObject({ file_version: "5" });
    expect(calls.some((path) => path.endsWith("projects.rescan"))).toBe(true);
  });

  it("does not offer to save layers that are not valid", async () => {
    server({ layers: [{ name: "domain", namespaces: ["Shop.Domain"] }] });
    render(() => <LayersScreen />);
    const name = await screen.findByLabelText("Name of the layer");
    fireEvent.input(name, { target: { value: "" } });
    expect(screen.getByRole("button", { name: "Save and scan again" })).toBeDisabled();
    expect(
      screen.getByText("Every layer needs a name of its own and at least one namespace."),
    ).toBeInTheDocument();
  });

  it("says why a project that was not scanned or is no folder has nothing to edit", async () => {
    server({ scanned: false });
    render(() => <LayersScreen />);
    expect(await screen.findByText(/has not been scanned yet/)).toBeInTheDocument();
  });

  it("explains that a Git project cannot be edited", async () => {
    server({ editable: false });
    render(() => <LayersScreen />);
    await waitFor(() => expect(screen.getByText(/no project folder/)).toBeInTheDocument());
  });
});
