import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { currentPath, navigate } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { pendingFocus, requestFocus } from "../../shared/navigation/navigation";
import { ANSWER } from "../../tests/usages";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { UsagesScreen } from "./UsagesScreen";
import { showUsages } from "../../shared/navigation/navigation";
import { clearUsages } from "./usages";

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

  it("shows the symbol, the counts, the impact and the cards of the graph", async () => {
    shown();
    expect(await screen.findByRole("heading", { name: "Cart.Total" })).toBeTruthy();
    expect(screen.getByText("decimal Total()")).toBeTruthy();
    expect(screen.getByText(/reaches 5 symbols in 2 layers, 2 of them public/)).toBeTruthy();
    const graph = screen.getByRole("region", { name: "Usage graph" });
    expect(within(graph).getByText("CartView.Render")).toBeTruthy();
    expect(within(graph).getByText("Callers · level 2")).toBeTruthy();
    expect(within(graph).getByText("Price.Value")).toBeTruthy();
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
    const mark = await screen.findByText("Total");
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

  it("makes a clicked card the focus", async () => {
    shown();
    const graph = await screen.findByRole("region", { name: "Usage graph" });
    fireEvent.click(within(graph).getByText("CartView.Render"));
    await waitFor(() => expect(lastRequest().symbol).toBe(11));
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
