import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { currentPath, navigate } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { pendingFocus, requestFocus } from "../explorer/explorer";
import { useSearchShortcut } from "./keys";
import { QuickSearch } from "./QuickSearch";
import { SearchField } from "./SearchField";
import { closeSearch, openSearch } from "./search";
import type { Hit } from "./types";

const CART: Hit = {
  id: 4,
  name: "Cart",
  kind: "class",
  qualified_name: "Shop.Cart",
  path: "src/Cart.cs",
  parent: 2,
};
const ADD: Hit = {
  id: 9,
  name: "AddItem",
  kind: "method",
  qualified_name: "Shop.Cart.AddItem",
  path: "src/Cart.cs",
  parent: 4,
};

useServerSession();
afterEach(() => {
  closeSearch();
  requestFocus(null);
  navigate("/");
});

function shown(hits: Hit[] = [CART, ADD], snapshot: number | null = 1) {
  openProject(1);
  serve({ "search.query": () => answer({ snapshot, hits }) });
  const view = render(() => {
    useSearchShortcut(openSearch);
    return <QuickSearch />;
  });
  openSearch();
  return view;
}

/** Replaces the clipboard of the page and renders the search with two hits, typed and listed. */
async function withClipboard() {
  const writeText = vi.fn().mockResolvedValue(undefined);
  vi.stubGlobal("navigator", { clipboard: { writeText } });
  shown();
  type("cart");
  await screen.findAllByRole("option");
  return writeText;
}

const input = () => screen.getByRole("combobox");
const type = (text: string) => fireEvent.input(input(), { target: { value: text } });

describe("the quick search", () => {
  it("opens with Ctrl+K and closes with Esc", async () => {
    shown();
    closeSearch();
    expect(screen.queryByRole("dialog")).toBeNull();
    fireEvent.keyDown(document.body, { key: "k", ctrlKey: true });
    expect(await screen.findByRole("dialog")).toBeTruthy();
    fireEvent.keyDown(input(), { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("asks to open a project first", () => {
    render(() => <QuickSearch />);
    openProject(null as unknown as number);
    openSearch();
    expect(screen.getByText(/Open a project first/)).toBeTruthy();
  });

  it("groups the hits, marks the match and counts them", async () => {
    shown();
    type("cart");
    expect(await screen.findByText("2 hits", { exact: false })).toBeTruthy();
    const list = screen.getByRole("listbox");
    expect(within(list).getByText("Types")).toBeTruthy();
    expect(within(list).getByText("Members")).toBeTruthy();
    expect(list.querySelector("mark")?.textContent).toBe("Cart");
    expect(screen.getAllByRole("option")).toHaveLength(2);
  });

  it("shows the chosen hit in the preview", async () => {
    shown();
    type("cart");
    await screen.findAllByRole("option");
    const preview = screen.getByRole("complementary");
    expect(within(preview).getByText("Shop.Cart")).toBeTruthy();
    expect(within(preview).getByText("src/Cart.cs")).toBeTruthy();
  });

  it("opens a type in the explorer with Enter", async () => {
    shown();
    type("cart");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "Enter" });
    expect(pendingFocus()).toEqual({ type: 4, member: null, kind: null });
    expect(currentPath()).toBe("/explorer");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("opens a member within its type", async () => {
    shown();
    type("cart");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "Enter" });
    expect(pendingFocus()).toEqual({ type: 4, member: 9, kind: "method" });
  });

  it("limits the list with a prefix and with Tab", async () => {
    shown();
    type("m:cart");
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(1));
    type("cart");
    fireEvent.keyDown(input(), { key: "Tab" });
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(1));
    expect(screen.getByRole("button", { name: "t: Types" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
  });

  it("runs a command that was chosen with >", async () => {
    shown();
    type(">settings");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "Enter" });
    expect(currentPath()).toBe("/settings");
  });

  it("copies the qualified name with Ctrl+C", async () => {
    const writeText = await withClipboard();
    fireEvent.keyDown(input(), { key: "c", ctrlKey: true });
    expect(writeText).toHaveBeenCalledWith("Shop.Cart");
    vi.unstubAllGlobals();
  });

  it("says when nothing was found", async () => {
    shown([]);
    type("zzz");
    expect(await screen.findByText("No hits.")).toBeTruthy();
  });

  it("says when the project has not been scanned", async () => {
    shown([], null);
    type("zzz");
    expect(await screen.findByText(/not been scanned/)).toBeTruthy();
  });

  it("says when the search failed", async () => {
    openProject(1);
    serve({ "search.query": () => answer({ code: "PRJ-201" }, 404) });
    render(() => <QuickSearch />);
    openSearch();
    type("zzz");
    expect(await screen.findByText("The search failed.")).toBeTruthy();
  });

  it("speaks German", async () => {
    setLanguage("de");
    shown();
    type("cart");
    expect(await screen.findByText(/Treffer/)).toBeTruthy();
    expect(within(screen.getByRole("listbox")).getByText("Typen")).toBeTruthy();
  });

  it("does not copy over a selection in the field, and copies with Cmd+C", async () => {
    const writeText = await withClipboard();
    (input() as HTMLInputElement).setSelectionRange(0, 2);
    fireEvent.keyDown(input(), { key: "c", ctrlKey: true });
    expect(writeText).not.toHaveBeenCalled();
    (input() as HTMLInputElement).setSelectionRange(1, 1);
    fireEvent.keyDown(input(), { key: "c", metaKey: true });
    expect(writeText).toHaveBeenCalledWith("Shop.Cart");
    vi.unstubAllGlobals();
  });

  it("gives the focus back when it closes", async () => {
    const button = document.createElement("button");
    document.body.append(button);
    button.focus();
    shown();
    expect(document.activeElement).toBe(input());
    fireEvent.keyDown(input(), { key: "Escape" });
    await waitFor(() => expect(document.activeElement).toBe(button));
    button.remove();
  });

  it("opens with Ctrl+K from a field and keeps the text when it is open already", async () => {
    shown();
    closeSearch();
    const other = document.createElement("input");
    document.body.append(other);
    other.focus();
    fireEvent.keyDown(other, { key: "k", ctrlKey: true });
    expect(await screen.findByRole("dialog")).toBeTruthy();
    type("cart");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "k", ctrlKey: true });
    expect((input() as HTMLInputElement).value).toBe("cart");
    expect(screen.getAllByRole("option")).toHaveLength(2);
    other.remove();
  });

  it("opens from the search field of the top bar", async () => {
    openProject(1);
    render(() => (
      <>
        <SearchField />
        <QuickSearch />
      </>
    ));
    fireEvent.click(screen.getByRole("button", { name: /Ctrl K/ }));
    expect(await screen.findByRole("dialog")).toBeTruthy();
  });

  it("moves the choice up, goes back a chip with Shift+Tab and opens a hit by click", async () => {
    shown();
    type("cart");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "ArrowUp" });
    expect(screen.getAllByRole("option")[0]?.getAttribute("aria-selected")).toBe("true");
    fireEvent.keyDown(input(), { key: "Tab", shiftKey: true });
    expect(screen.getByRole("button", { name: "> Commands" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
    fireEvent.keyDown(input(), { key: "Tab" });
    fireEvent.click(await screen.findByText("AddItem", { selector: "[class*=label]" }));
    expect(pendingFocus()?.member).toBe(9);
  });

  it("drops the answer of an earlier search that arrives late", async () => {
    openProject(1);
    let release: (value: Response) => void = () => undefined;
    fetchMock
      .mockImplementationOnce(() => new Promise<Response>((resolve) => (release = resolve)))
      .mockImplementationOnce(() => Promise.resolve(answer({ snapshot: 1, hits: [ADD] })));
    render(() => <QuickSearch />);
    openSearch();
    type("a");
    type("ad");
    await screen.findByRole("option", { name: /AddItem/ });
    release(answer({ snapshot: 1, hits: [CART] }));
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(screen.queryByText("Cart", { selector: "[class*=label]" })).toBeNull();
    expect(screen.getAllByRole("option")).toHaveLength(1);
  });

  it("forgets the time of an earlier search when a search fails", async () => {
    shown();
    type("cart");
    await screen.findByText(/hits/);
    serve({ "search.query": () => answer({ code: "PRJ-201" }, 404) });
    type("carts");
    expect(await screen.findByText("The search failed.")).toBeTruthy();
    expect(screen.queryByText(/hits/)).toBeNull();
  });

  it("says the hints in German", async () => {
    setLanguage("de");
    shown([]);
    expect(screen.getByText(/Namen eingeben/)).toBeTruthy();
    type("zzz");
    expect(await screen.findByText("Keine Treffer.")).toBeTruthy();
    serve({ "search.query": () => answer({ code: "PRJ-201" }, 404) });
    type("zzzz");
    expect(await screen.findByText("Die Suche ist fehlgeschlagen.")).toBeTruthy();
  });
});
