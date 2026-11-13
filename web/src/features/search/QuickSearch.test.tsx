import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { currentPath, navigate } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, serve, useServerSession } from "../../tests/commands";
import { pendingFocus, requestFocus } from "../explorer/explorer";
import { useSearchShortcut } from "./keys";
import { QuickSearch } from "./QuickSearch";
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
    expect(screen.getByRole("button", { name: "Types" }).getAttribute("aria-pressed")).toBe("true");
  });

  it("runs a command that was chosen with >", async () => {
    shown();
    type(">settings");
    await screen.findAllByRole("option");
    fireEvent.keyDown(input(), { key: "Enter" });
    expect(currentPath()).toBe("/settings");
  });

  it("copies the qualified name with Ctrl+C", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    shown();
    type("cart");
    await screen.findAllByRole("option");
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
});
