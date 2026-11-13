import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { openOrder, ORDER, OUTLINE } from "../../tests/explorer";
import { ExplorerScreen } from "./ExplorerScreen";
import { requestFocus } from "./explorer";

useServerSession();

/** Renders the screen for a project whose tree and type the server answers. */
function renderOrder() {
  openProject(1);
  serve({ "explorer.outline": () => answer(OUTLINE), "explorer.symbol": () => answer(ORDER) });
  render(() => <ExplorerScreen />);
}

describe("ExplorerScreen", () => {
  it("asks to open a project first", async () => {
    render(() => <ExplorerScreen />);
    expect(await screen.findByText("No project open")).toBeTruthy();
  });

  it("shows the structure, opens a type and shows its metrics and members", async () => {
    renderOrder();
    await openOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    expect(screen.getByText("An order of a customer.")).toBeTruthy();
    expect(screen.getByText("188")).toBeTruthy();
    expect(screen.getByText("decimal Total()")).toBeTruthy();
    expect(screen.getByText("Members 2")).toBeTruthy();
    expect(await screen.findByText("Fields etc.")).toBeTruthy();
  });

  it("filters the member table by chip", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.click(screen.getByRole("button", { name: "Properties" }));
    await waitFor(() => expect(screen.queryByText("decimal Total()")).toBeNull());
    expect(screen.getAllByText("int").length).toBeGreaterThan(0);
  });

  it("says that a project has not been scanned yet", async () => {
    openProject(2);
    serve({ "explorer.outline": () => answer({ snapshot: null, layers: [] }) });
    render(() => <ExplorerScreen />);
    expect(await screen.findByText(/not been scanned/)).toBeTruthy();
  });

  it("shows the message of a failed read", async () => {
    openProject(3);
    serve({ "explorer.outline": () => answer({ code: "PRJ-201" }, 404) });
    render(() => <ExplorerScreen />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("The project is not registered");
  });

  it("says the same in German", async () => {
    setLanguage("de");
    openProject(3);
    serve({ "explorer.outline": () => answer({ code: "PRJ-201" }, 404) });
    render(() => <ExplorerScreen />);
    expect((await screen.findByRole("alert")).textContent).toContain("nicht registriert");
  });

  it("says that the scan found no types", async () => {
    openProject(4);
    serve({ "explorer.outline": () => answer({ ...OUTLINE, layers: [] }) });
    render(() => <ExplorerScreen />);
    expect(await screen.findByText("The newest scan found no types.")).toBeTruthy();
  });

  it("says that the structure is being read until the answer is there", async () => {
    openProject(5);
    fetchMock.mockImplementation(() => new Promise(() => undefined));
    render(() => <ExplorerScreen />);
    expect(await screen.findByText(/Reading the structure/)).toBeTruthy();
  });

  it("starts closed with the overview", async () => {
    renderOrder();
    expect(await screen.findByText("Overview")).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Order" })).toBeNull();
    expect(screen.queryByText("Shop.Domain")).toBeNull();
  });
});

describe("the click rules of the tree", () => {
  const caretOf = (name: string) =>
    screen.getByText(name).closest("[role=treeitem]")!.querySelector("[class*=caret]")!;
  const GROUP_NAME = "Properties & fields";

  it("selects a type that was not selected without opening it", async () => {
    renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    expect(screen.queryByText(GROUP_NAME)).toBeNull();
  });

  it("opens and closes the selected type on the next clicks", async () => {
    renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    fireEvent.click(screen.getByText("Order", { selector: "[class*=label]" }));
    expect(await screen.findByText(GROUP_NAME)).toBeTruthy();
    fireEvent.click(screen.getByText("Order", { selector: "[class*=label]" }));
    await waitFor(() => expect(screen.queryByText(GROUP_NAME)).toBeNull());
  });

  it("lets the caret only open and close, and leaves the selection", async () => {
    renderOrder();
    fireEvent.click(await screen.findByText("Domain"));
    fireEvent.click(await screen.findByText("Shop.Domain"));
    fireEvent.click(caretOf("Order"));
    expect(await screen.findByText(GROUP_NAME)).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Order" })).toBeNull();
  });

  it("opens and closes a group on its own", async () => {
    renderOrder();
    await openOrder();
    fireEvent.click(screen.getByText("Order", { selector: "[class*=label]" }));
    const tree = screen.getByRole("tree");
    await within(tree).findByText("Properties & fields");
    fireEvent.click(within(tree).getByText("Methods"));
    expect(await within(tree).findByText("Total")).toBeTruthy();
    fireEvent.click(within(tree).getByText("Methods"));
    await waitFor(() => expect(within(tree).queryByText("Total")).toBeNull());
  });
});

describe("the keys of the tree", () => {
  it("moves the cursor with the arrows and opens a row with Enter", async () => {
    renderOrder();
    const tree = await screen.findByRole("tree");
    await screen.findByText("Domain");
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.getAttribute("aria-activedescendant")).toBe("explorer-row-0");
    fireEvent.keyDown(tree, { key: "Enter" });
    expect(await screen.findByText("Shop.Domain")).toBeTruthy();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.getAttribute("aria-activedescendant")).toBe("explorer-row-1");
    fireEvent.keyDown(tree, { key: "ArrowLeft" });
    fireEvent.keyDown(tree, { key: "ArrowUp" });
    fireEvent.keyDown(tree, { key: "ArrowLeft" });
    await waitFor(() => expect(screen.queryByText("Shop.Domain")).toBeNull());
  });

  it("closes everything on Ctrl+Left and shows the selection on Ctrl+.", async () => {
    renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    fireEvent.keyDown(document.body, { key: "ArrowLeft", ctrlKey: true });
    const tree = screen.getByRole("tree");
    await waitFor(() => expect(within(tree).queryByText("Shop.Domain")).toBeNull());
    fireEvent.keyDown(document.body, { key: ".", ctrlKey: true });
    expect(await within(tree).findByText("Shop.Domain")).toBeTruthy();
  });
});

describe("the details", () => {
  it("shows the colour of a metric together with its number", async () => {
    renderOrder();
    await openOrder();
    const maintainability = await screen.findByText("25");
    expect(maintainability.className).toContain("poor");
  });

  it("names the limits in the tip of a tile", async () => {
    renderOrder();
    await openOrder();
    const label = await screen.findByText("Maintainability");
    expect(label.parentElement?.getAttribute("title")).toMatch(/65.*35/);
  });

  it("says that a symbol has no members", async () => {
    openProject(1);
    serve({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer({ ...ORDER, members: [] }),
    });
    render(() => <ExplorerScreen />);
    await openOrder();
    expect(await screen.findByText("This symbol has no members.")).toBeTruthy();
  });

  it("keeps the tree when a symbol cannot be read", async () => {
    openProject(1);
    serve({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer({ code: "MCP-202" }, 404),
    });
    render(() => <ExplorerScreen />);
    await openOrder();
    expect((await screen.findAllByRole("alert")).length).toBeGreaterThan(0);
    expect(screen.getByText("Shop.Domain")).toBeTruthy();
  });

  it("reveals a nested type of the table in the tree", async () => {
    const line = { ...ORDER.members[0]!, id: 12, name: "Line", kind: "class", signature: null };
    openProject(1);
    serve({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer({ ...ORDER, members: [...ORDER.members, line] }),
    });
    render(() => <ExplorerScreen />);
    await openOrder();
    fireEvent.click(await screen.findByRole("button", { name: /Line/ }));
    expect(await screen.findByRole("treeitem", { name: /Line/ })).toBeTruthy();
  });
});

describe("a navigation from another screen", () => {
  it("opens the rows down to a member and shows its type", async () => {
    requestFocus({ type: 7, member: 8, kind: "method" });
    renderOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    const tree = screen.getByRole("tree");
    expect(await within(tree).findByText("Total")).toBeTruthy();
    expect(within(tree).getByText("Shop.Domain")).toBeTruthy();
    const cursor = tree.getAttribute("aria-activedescendant") ?? "";
    expect(document.getElementById(cursor)?.textContent).toContain("Total");
  });

  it("opens the rows down to a type", async () => {
    requestFocus({ type: 7, member: null, kind: null });
    renderOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    expect(within(screen.getByRole("tree")).getByText("Shop.Domain")).toBeTruthy();
  });
});
