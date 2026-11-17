import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { currentPath, navigate } from "../../app/router";
import { focusSymbol } from "../../shared/navigation/navigation";
import { clearUsages } from "../usages/usages";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { openOrder, ORDER, OUTLINE } from "../../tests/explorer";
import { currentMember, forgetExplorer, setMemberSort, treeScroll } from "./explorer";
import { INITIAL_SORT } from "./memberSort";
import { ExplorerScreen } from "./ExplorerScreen";
import { requestFocus } from "../../shared/navigation/navigation";

useServerSession();
afterEach(() => {
  setMemberSort(INITIAL_SORT);
  forgetExplorer();
  clearUsages();
  navigate("/");
});

/** Renders the screen for a project whose tree and type the server answers. */
function renderOrder() {
  openProject(1);
  serve({ "explorer.outline": () => answer(OUTLINE), "explorer.symbol": () => answer(ORDER) });
  return render(() => <ExplorerScreen />);
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

  it("opens the usages of a member from its count, of the type from the button and with U", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.click(screen.getAllByTitle("Show the usages")[0]!);
    expect(currentPath()).toBe("/usages");
    expect(focusSymbol()).toBe(9);
    navigate("/explorer");
    fireEvent.click(screen.getByRole("button", { name: /^Usages/ }));
    expect(focusSymbol()).toBe(7);
    navigate("/explorer");
    fireEvent.keyDown(document.body, { key: "u" });
    expect(currentPath()).toBe("/usages");
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
    expect(screen.queryByText("Sales")).toBeNull();
  });

  it("counts in the singular when there is one of a kind", async () => {
    renderOrder();
    expect(await screen.findByText(/^1 layer · 1 namespace · \d+ types?$/)).toBeTruthy();
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
    fireEvent.click(await screen.findByText("Shop"));
    fireEvent.click(await screen.findByText("Sales"));
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
    expect(await screen.findByText("Shop")).toBeTruthy();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.getAttribute("aria-activedescendant")).toBe("explorer-row-1");
    fireEvent.keyDown(tree, { key: "Enter" });
    expect(await screen.findByText("Sales")).toBeTruthy();
    fireEvent.keyDown(tree, { key: "ArrowLeft" });
    fireEvent.keyDown(tree, { key: "ArrowUp" });
    fireEvent.keyDown(tree, { key: "ArrowLeft" });
    await waitFor(() => expect(screen.queryByText("Shop")).toBeNull());
  });

  it("closes everything on Ctrl+Left and shows the selection on Ctrl+.", async () => {
    renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    fireEvent.keyDown(document.body, { key: "ArrowLeft", ctrlKey: true });
    const tree = screen.getByRole("tree");
    await waitFor(() => expect(within(tree).queryByText("Sales")).toBeNull());
    fireEvent.keyDown(document.body, { key: ".", ctrlKey: true });
    expect(await within(tree).findByText("Sales")).toBeTruthy();
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
    expect(screen.getByText("Sales")).toBeTruthy();
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
    fireEvent.click(await screen.findByRole("button", { name: /Line$/ }));
    expect(await screen.findByRole("treeitem", { name: /Line/ })).toBeTruthy();
  });
});

describe("the layer graph button", () => {
  const renderWith = (layer: string | null) => {
    openProject(1);
    serve({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer({ ...ORDER, layer }),
    });
    return render(() => <ExplorerScreen />);
  };

  it("is off for a symbol that no layer owns", async () => {
    renderWith(null);
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(screen.getByRole("button", { name: /Layer graph/ })).toBeDisabled();
  });

  it("is on for a symbol of a layer", async () => {
    renderWith("Domain");
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(screen.getByRole("button", { name: /Layer graph/ })).toBeEnabled();
  });
});

describe("the way back from a member", () => {
  it("leads from a member picked in the table back to the table, and only then", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(screen.queryByRole("button", { name: /Back to the members/ })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: /Total/ }));
    await waitFor(() => expect(currentMember()).toBe(8));
    fireEvent.click(await screen.findByRole("button", { name: /Back to the members/ }));
    await waitFor(() => expect(currentMember()).toBeNull());
    expect(await screen.findByText("decimal Total()")).toBeTruthy();
  });

  it("offers no way back to the table for a member reached from outside", async () => {
    requestFocus({ type: 7, member: 8, kind: "method" });
    renderOrder();
    expect(await screen.findByRole("heading", { name: "Total" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Back to the members/ })).toBeNull();
  });
});

describe("a navigation from another screen", () => {
  it("opens the rows down to a member and shows its type", async () => {
    requestFocus({ type: 7, member: 8, kind: "method" });
    renderOrder();
    expect(await screen.findByRole("heading", { name: "Total" })).toBeTruthy();
    const tree = screen.getByRole("tree");
    expect(await within(tree).findByText("Total")).toBeTruthy();
    expect(within(tree).getByText("Sales")).toBeTruthy();
    const cursor = tree.getAttribute("aria-activedescendant") ?? "";
    expect(document.getElementById(cursor)?.textContent).toContain("Total");
  });

  it("opens the rows down to a type", async () => {
    requestFocus({ type: 7, member: null, kind: null });
    renderOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    expect(within(screen.getByRole("tree")).getByText("Sales")).toBeTruthy();
  });
});

describe("a navigation while the tree is read again", () => {
  it("keeps the type the navigation read, so the details do not fall back to the overview", async () => {
    const first = renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    first.unmount();
    requestFocus({ type: 30, member: null, kind: null });
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      const other = String(init?.body ?? "").includes('"symbol":30');
      return Promise.resolve(
        answer(
          path.endsWith("explorer.outline")
            ? OUTLINE
            : other
              ? { ...ORDER, id: 30, name: "Other" }
              : ORDER,
        ),
      );
    });
    render(() => <ExplorerScreen />);
    expect(await screen.findByRole("heading", { name: "Other" })).toBeTruthy();
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(screen.getByRole("heading", { name: "Other" })).toBeTruthy();
  });
});

describe("coming back to the explorer", () => {
  it("keeps the open rows, the selection and the filter when another screen was shown", async () => {
    const first = renderOrder();
    await openOrder();
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    first.unmount();
    render(() => <ExplorerScreen />);
    expect(await screen.findByRole("heading", { name: "Order" })).toBeTruthy();
    expect(screen.getAllByText("Sales").length).toBeGreaterThan(0);
    expect(screen.getAllByRole("treeitem", { selected: true }).length).toBe(1);
  });

  it("starts from nothing for another project", async () => {
    const first = renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    first.unmount();
    openProject(2);
    render(() => <ExplorerScreen />);
    expect(await screen.findByText("Domain")).toBeTruthy();
    expect(screen.queryByText("Sales")).toBeNull();
  });

  it("reads the tree again on the way back and drops a symbol that is gone", async () => {
    const first = renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    first.unmount();
    const renamed = {
      ...OUTLINE,
      layers: [
        {
          name: "Domain",
          namespaces: [
            { name: "Shop.Sales", types: [{ id: 7, name: "Order2", kind: "class", members: 2 }] },
          ],
        },
      ],
    };
    serve({
      "explorer.outline": () => answer(renamed),
      "explorer.symbol": () => answer({ code: "gone" }, 404),
    });
    render(() => <ExplorerScreen />);
    expect(await screen.findByText("Order2")).toBeTruthy();
    expect(screen.getAllByText("Sales").length).toBeGreaterThan(0);
  });

  it("scrolls the tree to where it was", async () => {
    const first = renderOrder();
    await openOrder();
    await screen.findByRole("heading", { name: "Order" });
    const tree = screen.getByRole("tree");
    tree.scrollTop = 52;
    fireEvent.scroll(tree);
    expect(treeScroll()).toBe(52);
    first.unmount();
    render(() => <ExplorerScreen />);
    await screen.findByRole("heading", { name: "Order" });
    await waitFor(() => expect(screen.getByRole("tree").scrollTop).toBe(52));
  });
});

describe("sorting the member table", () => {
  const names = () =>
    Array.from(document.querySelectorAll("tbody td:first-child button")).map((cell) =>
      (cell.textContent ?? "").trim(),
    );

  it("sorts by name at first and by a column when its head is pressed, turning it round on the second press", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(names().length).toBe(2);
    expect(names()[0]).toContain("Id");
    expect(screen.getByRole("columnheader", { name: /Name/ }).getAttribute("aria-sort")).toBe(
      "ascending",
    );
    fireEvent.click(screen.getByRole("button", { name: /^CC/ }));
    expect(names()[0]).toContain("Total");
    expect(screen.getByRole("columnheader", { name: /CC/ }).getAttribute("aria-sort")).toBe(
      "descending",
    );
    fireEvent.click(screen.getByRole("button", { name: /^CC/ }));
    // A member without a value stays last, also when the order is turned round.
    expect(names()[0]).toContain("Total");
    expect(names()[1]).toContain("Id");
    // A third press removes the sort: no column is marked and the rows follow the code.
    fireEvent.click(screen.getByRole("button", { name: /^CC/ }));
    expect(screen.getByRole("columnheader", { name: /CC/ }).getAttribute("aria-sort")).toBeNull();
  });

  it("steps through the members in the order of the table", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    const step = () => fireEvent.keyDown(document.body, { key: "ArrowDown", altKey: true });
    fireEvent.click(screen.getByRole("button", { name: /^CC/ }));
    step();
    await waitFor(() => expect(currentMember()).toBe(8));
    step();
    await waitFor(() => expect(currentMember()).toBe(9));
  });

  it("steps only through the members that the chip keeps", async () => {
    renderOrder();
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.click(screen.getByRole("button", { name: "Properties" }));
    fireEvent.keyDown(document.body, { key: "ArrowDown", altKey: true });
    await waitFor(() => expect(currentMember()).toBe(9));
    fireEvent.keyDown(document.body, { key: "ArrowDown", altKey: true });
    await waitFor(() => expect(currentMember()).toBe(9));
  });
});

describe("a member selected in the tree", () => {
  it("shows only the header and the source, with the member as the title", async () => {
    renderOrder();
    await openOrder();
    fireEvent.click(
      await screen
        .findByRole("heading", { name: "Order" })
        .then(() => screen.getAllByRole("button", { name: /Total/ })[0]!),
    );
    expect(await screen.findByRole("heading", { name: "Total" })).toBeTruthy();
    expect(screen.queryByText("Members 2")).toBeNull();
    expect(screen.queryByText("Codelines")).toBeNull();
    const crumbs = document.querySelector("[class*=crumbs]")?.textContent ?? "";
    expect(crumbs).toContain("Order");
    expect(crumbs).toContain("Total");
    expect(screen.getByRole("button", { name: /^Usages/ })).toBeTruthy();
    // The source sits in the middle column, under the header, and no third column is left.
    const details = screen.getByRole("region", { name: "Details" });
    expect(within(details).getByRole("region", { name: "Source" })).toBeTruthy();
    // Only the tree has a separator left; the one of the source column is gone.
    expect(screen.getAllByRole("separator")).toHaveLength(1);
  });
});
