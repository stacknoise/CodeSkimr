import { describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { UI_COMMANDS } from "./commands";
import { FILTER } from "./consts";
import { buildEntries, listRows } from "./entries";
import { moveChoice, nextChip, onSearchKey } from "./keys";
import { parseQuery, splitMatch } from "./parse";
import type { Hit } from "./types";

const hit = (id: number, name: string, kind: string): Hit => ({
  id,
  name,
  kind,
  qualified_name: `Shop.${name}`,
  path: "Shop.cs",
  parent: null,
  project: 1,
  project_name: "shop",
});
const HITS = [hit(1, "Cart", "class"), hit(2, "Add", "method"), hit(3, "ICart", "interface")];

describe("parseQuery", () => {
  it("takes the filter from a prefix and trims the text", () => {
    expect(parseQuery("t: cart", FILTER.all)).toEqual({ filter: FILTER.types, text: "cart" });
    expect(parseQuery("m:add", FILTER.all)).toEqual({ filter: FILTER.members, text: "add" });
    expect(parseQuery(">  scan", FILTER.all)).toEqual({ filter: FILTER.commands, text: "scan" });
    expect(parseQuery("f: Cart.cs", FILTER.all)).toEqual({ filter: FILTER.files, text: "Cart.cs" });
    expect(parseQuery("@cart", FILTER.all)).toEqual({ filter: FILTER.projects, text: "cart" });
  });

  it("takes the chip when there is no prefix", () => {
    expect(parseQuery(" cart ", FILTER.members)).toEqual({ filter: FILTER.members, text: "cart" });
  });
});

describe("splitMatch", () => {
  it("marks the first occurrence in any case", () => {
    expect(splitMatch("BoardBar", "bar")).toEqual({ pre: "Board", hit: "Bar", post: "" });
  });

  it("marks nothing when the text does not occur", () => {
    expect(splitMatch("Cart", "zz")).toEqual({ pre: "Cart", hit: "", post: "" });
    expect(splitMatch("Cart", "")).toEqual({ pre: "Cart", hit: "", post: "" });
  });
});

describe("buildEntries, files and all projects", () => {
  const FILE = { path: "src/Cart.cs", loc: 232, first_type: 1, project: 1, project_name: "shop" };

  it("shows the files after the members, and alone for the filter f:", () => {
    const all = buildEntries(HITS, [FILE], UI_COMMANDS, FILTER.all, "cart");
    expect(all.map((entry) => entry.group)).toEqual(["types", "types", "members", "files"]);
    const only = buildEntries(HITS, [FILE], UI_COMMANDS, FILTER.files, "cart");
    expect(only.map((entry) => entry.label)).toEqual(["src/Cart.cs"]);
    expect(only[0]?.file).toEqual(FILE);
  });

  it("keeps symbols and files, but no commands, for all projects", () => {
    const wide = buildEntries(HITS, [FILE], UI_COMMANDS, FILTER.projects, "scan");
    expect(wide.map((entry) => entry.group)).toEqual(["types", "types", "members", "files"]);
  });
});

describe("buildEntries", () => {
  it("sorts the hits into types and members, types first", () => {
    const entries = buildEntries(HITS, [], UI_COMMANDS, FILTER.all, "a");
    expect(entries.map((entry) => entry.group).slice(0, 3)).toEqual(["types", "types", "members"]);
  });

  it("keeps only one group for a filter", () => {
    const members = buildEntries(HITS, [], UI_COMMANDS, FILTER.members, "a");
    expect(members.map((entry) => entry.label)).toEqual(["Add"]);
  });

  it("offers every command for a bare > and the matching ones for a text", () => {
    setLanguage("en");
    expect(buildEntries([], [], UI_COMMANDS, FILTER.commands, "")).toHaveLength(UI_COMMANDS.length);
    const found = buildEntries([], [], UI_COMMANDS, FILTER.commands, "explorer");
    expect(found.map((entry) => entry.command?.label)).toEqual([I18N_KEYS.NAV_EXPLORER]);
  });

  it("shows no command for an empty text under All", () => {
    expect(buildEntries([], [], UI_COMMANDS, FILTER.all, "")).toEqual([]);
  });
});

describe("listRows", () => {
  it("puts a title before each group", () => {
    const rows = listRows(buildEntries(HITS, [], UI_COMMANDS, FILTER.all, "cart"));
    expect(rows.filter((row) => row.entry === null).map((row) => row.group)).toEqual([
      "types",
      "members",
    ]);
    expect(rows[0]?.entry).toBeNull();
    expect(rows[1]?.index).toBe(0);
  });
});

describe("the keys", () => {
  it("moves the choice and stops at the ends", () => {
    expect(moveChoice("ArrowDown", 0, 3)).toBe(1);
    expect(moveChoice("ArrowDown", 2, 3)).toBe(2);
    expect(moveChoice("ArrowUp", 0, 3)).toBe(0);
    expect(moveChoice("x", 0, 3)).toBe(-1);
    expect(moveChoice("ArrowDown", 0, 0)).toBe(-1);
  });

  it("changes the chip with Tab and wraps around", () => {
    expect(nextChip(FILTER.all, 1)).toBe(FILTER.types);
    expect(nextChip(FILTER.all, -1)).toBe(FILTER.projects);
    expect(nextChip(FILTER.projects, 1)).toBe(FILTER.all);
  });

  it("opens in the editor with Alt+Enter, in the map with Ctrl+Enter and only opens with a plain Enter", () => {
    const entries = buildEntries(HITS, [], UI_COMMANDS, FILTER.all, "a");
    const press = (init: KeyboardEventInit) => {
      const actions = {
        close: vi.fn(),
        choose: vi.fn(),
        activate: vi.fn(),
        chip: vi.fn(),
        copy: vi.fn(),
        editor: vi.fn(),
        graph: vi.fn(),
      };
      const event = new KeyboardEvent("keydown", { key: "Enter", cancelable: true, ...init });
      onSearchKey(event, entries, 0, actions, false);
      return { actions, event };
    };
    const alt = press({ altKey: true });
    expect(alt.actions.editor).toHaveBeenCalledWith(entries[0]);
    expect(alt.actions.activate).not.toHaveBeenCalled();
    expect(alt.event.defaultPrevented).toBe(true);
    const plain = press({});
    expect(plain.actions.activate).toHaveBeenCalledWith(entries[0]);
    expect(plain.actions.editor).not.toHaveBeenCalled();
    const ctrl = press({ ctrlKey: true });
    expect(ctrl.actions.graph).toHaveBeenCalledWith(entries[0]);
    expect(ctrl.actions.activate).not.toHaveBeenCalled();
    const both = press({ altKey: true, ctrlKey: true });
    expect(both.actions.editor).not.toHaveBeenCalled();
    expect(both.actions.graph).not.toHaveBeenCalled();
    expect(both.actions.activate).not.toHaveBeenCalled();
  });
});
