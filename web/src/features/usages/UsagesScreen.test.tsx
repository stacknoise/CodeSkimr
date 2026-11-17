import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { currentPath, navigate } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { pendingFocus, requestFocus } from "../../shared/navigation/navigation";
import { pointer } from "../../tests/pointer";
import { ANSWER, VIEW } from "../../tests/usages";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { startView } from "./GraphView";
import type { Graph } from "./graph";
import { UsagesScreen } from "./UsagesScreen";
import { RailLink } from "../../app/Frame";
import { GRAPH_ROUTE } from "../../app/consts";
import { moveUsages, showUsages } from "../../shared/navigation/navigation";
import { clearUsages, focusSymbol } from "./usages";

useServerSession();
afterEach(() => {
  clearUsages();
  requestFocus(null);
  navigate("/");
});

const ORDER_SYMBOL = {
  snapshot: 5,
  id: 11,
  name: "Render",
  kind: "method",
  breadcrumb: [{ id: 3, name: "CartView", kind: "class" }],
};

function shown(body: unknown = ANSWER) {
  openProject(1);
  serve({ "usages.query": () => answer(body), "explorer.symbol": () => answer(ORDER_SYMBOL) });
  showUsages(10);
  return render(() => <UsagesScreen />);
}

/** What the screen sent to `usages.query` the last time. */
function lastRequest(): Record<string, unknown> {
  const calls = fetchMock.mock.calls.filter(([path]) => String(path).endsWith("usages.query"));
  return JSON.parse(calls[calls.length - 1]![1].body as string) as Record<string, unknown>;
}

describe("UsagesScreen", () => {
  it("asks to open a project first, then to choose a symbol", async () => {
    clearUsages();
    const view = render(() => <UsagesScreen />);
    expect(await screen.findByText("No project open")).toBeTruthy();
    view.unmount();
    openProject(1);
    clearUsages();
    render(() => <UsagesScreen />);
    expect(await screen.findByText("No symbol chosen")).toBeTruthy();
  });

  it("widens the side panel and keeps the width for the next visit", async () => {
    shown();
    await screen.findByRole("heading", { name: "Cart.Total" });
    const separators = screen.getAllByRole("separator");
    const panel = separators.find((item) => item.getAttribute("aria-orientation") === "vertical")!;
    const before = Number(panel.getAttribute("aria-valuenow"));
    fireEvent.keyDown(panel, { key: "ArrowLeft" });
    const after = Number(panel.getAttribute("aria-valuenow"));
    expect(after).toBeGreaterThan(before);
    expect(localStorage.getItem("codeskimr.layout.usages.panel")).toBe(String(after));
  });

  it("offers no buttons to the other levels of the map", async () => {
    shown();
    await screen.findByRole("heading", { name: "Cart.Total" });
    expect(screen.queryByRole("button", { name: "Namespaces" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Types" })).toBeNull();
  });

  it("goes back to the symbol in the explorer with the back button", async () => {
    shown();
    await screen.findByRole("heading", { name: "Cart.Total" });
    fireEvent.click(screen.getByRole("button", { name: /Back to the symbol/ }));
    await waitFor(() => expect(currentPath()).toBe("/explorer"));
  });

  it("leads back to the symbol the user came from, not to the one moved to in the graph", async () => {
    shown();
    serve({
      "usages.query": () => answer(ANSWER),
      "explorer.symbol": () =>
        answer({ ...ORDER_SYMBOL, id: 10, kind: "class", qualified_name: "Cart", breadcrumb: [] }),
    });
    await screen.findByRole("heading", { name: "Cart.Total" });
    moveUsages(11);
    await waitFor(() => expect(focusSymbol()).toBe(11));
    fireEvent.click(screen.getByRole("button", { name: /Back to the symbol/ }));
    await waitFor(() => expect(currentPath()).toBe("/explorer"));
    expect(pendingFocus()).toEqual({ type: 10, member: null, kind: null });
  });

  it("keeps the graph button of the rail marked on the symbol view", async () => {
    navigate("/usages");
    render(() => <RailLink route={GRAPH_ROUTE} />);
    expect(screen.getByRole("link")).toHaveAttribute("aria-current", "page");
  });

  it("shows the symbol, the counts, the impact and the cards of the graph", async () => {
    shown();
    expect(await screen.findByRole("heading", { name: "Cart.Total" })).toBeTruthy();
    expect(screen.getByText("decimal Total()")).toBeTruthy();
    expect(screen.getByText(/reaches 5 symbols in 2 layers, 2 of them public/)).toBeTruthy();
    const graph = screen.getByRole("region", { name: "Usage graph" });
    expect(within(graph).getByText("Render")).toBeTruthy();
    expect(within(graph).getByText("Callers · level 2")).toBeTruthy();
    expect(within(graph).getByText("Value")).toBeTruthy();
  });

  it("explains the lines and the number on the cards with a legend", async () => {
    shown();
    await screen.findByText("Hits 4");
    expect(screen.getByText("calls")).toBeTruthy();
    expect(screen.getByText("uses")).toBeTruthy();
    expect(screen.getByText("presumed (via interface)")).toBeTruthy();
    expect(screen.getByText("Number = places")).toBeTruthy();
    expect(screen.getByRole("region", { name: "Hits" })).toBeTruthy();
  });

  it("shows the code line of a hit with the used symbol marked, and keeps the list without it", async () => {
    openProject(1);
    serve({
      "usages.query": () => answer(ANSWER),
      "explorer.symbol": () => answer(ORDER_SYMBOL),
      "usages.code": () =>
        answer({
          available: true,
          lines: [{ path: "src/Ui.cs", line: 40, text: "var sum = cart.Total();" }],
        }),
    });
    showUsages(10);
    render(() => <UsagesScreen />);
    const mark = await screen.findByText("Total", { selector: "mark" });
    expect(mark.tagName).toBe("MARK");
    expect(mark.parentElement?.textContent).toBe("var sum = cart.Total();");
    expect(screen.getByText("src/Ui.cs:40")).toBeTruthy();
  });

  it("keeps the hit list when the code cannot be read", async () => {
    shown();
    await screen.findByText("Hits 4");
    expect(screen.getByText("src/Ui.cs:40")).toBeTruthy();
    expect(document.querySelector("mark")).toBeNull();
  });

  it("marks every hit unchecked or presumed and says that the result is preliminary", async () => {
    shown();
    await screen.findByText("Hits 4");
    expect(screen.getAllByText("unchecked").length).toBe(3);
    expect(screen.getAllByText("presumed").length).toBeGreaterThan(0);
    expect(screen.getByText(/No indexer is set up for this project/)).toBeTruthy();
    expect(screen.getByText("0 places verified, 3 unchecked, 1 of them presumed.")).toBeTruthy();
    expect(
      screen.getByText(
        "Of the listed symbols, reached through verified references: 0, only through unchecked references: 5.",
      ),
    ).toBeTruthy();
  });

  it("shows a verified hit as verified", async () => {
    const verified = structuredClone(ANSWER);
    verified.incoming!.hits[0]!.verified = true;
    verified.summary = { ...verified.summary, verified: 1, unchecked: 2 };
    shown(verified);
    await screen.findByText("Hits 4");
    expect(screen.getByText("verified")).toBeTruthy();
  });

  it("reads again with the direction, the depth and the tests of the toolbar", async () => {
    shown();
    await screen.findByText("Hits 4");
    expect(lastRequest()).toMatchObject({ project: 1, symbol: 10, way: "both", depth: 2 });
    fireEvent.click(screen.getByRole("button", { name: "who calls" }));
    await waitFor(() => expect(lastRequest().way).toBe("incoming"));
    fireEvent.change(screen.getByLabelText("Depth"), { target: { value: "0" } });
    await waitFor(() => expect(lastRequest().depth).toBe(0));
    fireEvent.click(screen.getByLabelText("Show tests"));
    await waitFor(() => expect(lastRequest().include_tests).toBe(false));
  });

  it("names the callee of an outgoing hit and opens it, not the symbol in focus", async () => {
    shown({ ...ANSWER, incoming: null });
    await screen.findByText("Hits 1");
    const list = screen.getByText("Hits 1").closest("div")!.parentElement!;
    expect(within(list).getByText("Shop.Domain.Price.Value")).toBeTruthy();
    fireEvent.keyDown(document.body, { key: "F8" });
    fireEvent.keyDown(document.body, { key: "Enter" });
    await waitFor(() => expect(currentPath()).toBe("/explorer"));
    expect(pendingFocus()?.member).toBe(14);
  });

  it("warns about missing indexers also on an empty list", async () => {
    shown({ ...ANSWER, incoming: { total: 0, hits: [] }, outgoing: null });
    expect(await screen.findByText(/does not rule out usages that were missed/)).toBeTruthy();
  });

  it("shows a clicked card in the side panel and makes a double-clicked card the focus", async () => {
    shown();
    const first = fetchMock.getMockImplementation()!;
    fetchMock.mockImplementation((path: string, init: RequestInit) =>
      String(path).endsWith("usages.query") && String(init?.body).includes('"symbol":11')
        ? Promise.resolve(answer({ ...ANSWER, symbol: VIEW, signature: "void Render()" }))
        : first(path, init),
    );
    const graph = await screen.findByRole("region", { name: "Usage graph" });
    fireEvent.click(within(graph).getByText("Render"));
    expect(await screen.findByText("void Render()")).toBeTruthy();
    expect(focusSymbol()).not.toBe(11);
    fireEvent.dblClick(within(graph).getByText("Render"));
    await waitFor(() => expect(focusSymbol()).toBe(11));
  });

  it("steps through the hits with F8 and opens the selected one in the explorer", async () => {
    shown();
    await screen.findByText("Hits 4");
    fireEvent.keyDown(document.body, { key: "F8" });
    fireEvent.keyDown(document.body, { key: "Enter" });
    await waitFor(() => expect(currentPath()).toBe("/explorer"));
    expect(pendingFocus()).toEqual({ type: 3, member: 11, kind: "method" });
  });

  it("says that nothing uses the symbol", async () => {
    shown({
      ...ANSWER,
      incoming: { total: 0, hits: [] },
      outgoing: null,
      impact: { total: 0, public_interfaces: 0, verified: 0, unchecked: 0, layers: [] },
    });
    expect(await screen.findByText(/No usages found/)).toBeTruthy();
    expect(screen.getByText("A change reaches no other symbol.")).toBeTruthy();
  });

  it("shows the message of a failed read", async () => {
    openProject(1);
    serve({ "usages.query": () => answer({ code: "MCP-202" }, 404) });
    showUsages(10);
    render(() => <UsagesScreen />);
    expect((await screen.findByRole("alert")).textContent).toBeTruthy();
  });

  it("speaks German", async () => {
    setLanguage("de");
    shown();
    expect(await screen.findByText("Fundstellen 4")).toBeTruthy();
    expect(screen.getAllByText("ungeprüft").length).toBe(3);
  });
});

describe("the graph of the usages", () => {
  const planeOf = () =>
    screen.getByRole("region", { name: "Usage graph" }).firstElementChild as HTMLElement;

  it("zooms with the wheel and the buttons and goes back to the whole picture", async () => {
    shown();
    await screen.findByRole("heading", { name: "Cart.Total" });
    const graph = screen.getByRole("region", { name: "Usage graph" });
    expect(planeOf().style.transform).toContain("scale(1)");
    fireEvent.wheel(graph, { deltaY: -100 });
    expect(planeOf().style.transform).not.toContain("scale(1)");
    fireEvent.click(screen.getByRole("button", { name: "Show everything" }));
    expect(planeOf().style.transform).toContain("scale(1)");
    fireEvent.click(screen.getByRole("button", { name: "Zoom out" }));
    expect(planeOf().style.transform).not.toContain("scale(1)");
  });

  /** Shows the graph and returns it with a way to drag on `target` with `button` and tell whether
   * the picture moved. */
  async function dragging() {
    shown();
    await screen.findByRole("heading", { name: "Cart.Total" });
    const graph = screen.getByRole("region", { name: "Usage graph" });
    graph.setPointerCapture = () => undefined;
    const before = planeOf().style.transform;
    return {
      graph,
      drag: (target: Element, button: number) => {
        pointer(target, "pointerdown", { button });
        pointer(graph, "pointermove", { movementX: 30, movementY: 20 });
        pointer(graph, "pointerup");
        return planeOf().style.transform !== before;
      },
    };
  }

  it("moves the picture when the empty ground or the middle button is dragged, not a card", async () => {
    const { graph, drag } = await dragging();
    expect(drag(within(graph).getByText("Render"), 0)).toBe(false);
    expect(drag(graph, 0)).toBe(true);
  });

  it("moves the picture when the middle button is dragged", async () => {
    const { graph, drag } = await dragging();
    expect(drag(graph, 1)).toBe(true);
  });

  it("opens a small graph in the middle of the area, at its own size", () => {
    const graph = { width: 400, height: 200, cards: [] } as unknown as Graph;
    expect(startView(graph, { width: 1000, height: 600 })).toEqual({ scale: 1, x: 300, y: 200 });
  });

  it("opens a graph that is a little too large smaller, and a huge one on the focus", () => {
    const big = { width: 1500, height: 600, cards: [] } as unknown as Graph;
    const view = startView(big, { width: 1000, height: 600 });
    expect(view.scale).toBeCloseTo(0.624, 3);
    const tall = {
      width: 500,
      height: 4000,
      cards: [{ focus: true, x: 100, y: 2000, height: 40 }],
    } as unknown as Graph;
    const centred = startView(tall, { width: 1000, height: 600 });
    expect(centred.scale).toBe(0.6);
    expect(centred.x).toBe(350);
    expect(centred.y).toBeCloseTo(300 - 2020 * 0.6, 5);
  });
});
