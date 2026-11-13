import { beforeEach, describe, expect, it, vi } from "vitest";

function dark(matches: boolean) {
  vi.stubGlobal("matchMedia", () => ({ matches }));
}

async function freshTheme() {
  vi.resetModules();
  return import("./theme");
}

beforeEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
  dark(false);
});

describe("theme", () => {
  it("starts with Petrol on a light system and Dunkel on a dark one", async () => {
    expect((await freshTheme()).activeTheme()).toBe("petrol");
    dark(true);
    expect((await freshTheme()).activeTheme()).toBe("dunkel");
  });

  it("applies the choice at once and mirrors it for the script that runs before the first paint", async () => {
    const theme = await freshTheme();
    theme.selectTheme("hell");
    expect(document.documentElement.getAttribute("data-theme")).toBe("hell");
    expect(localStorage.getItem("codeskimr.theme")).toBe("hell");
    expect(theme.chosenTheme()).toBe("hell");
  });

  it("reads a stored choice and ignores a stored name that is no theme", async () => {
    localStorage.setItem("codeskimr.theme", "dunkel");
    expect((await freshTheme()).activeTheme()).toBe("dunkel");
    localStorage.setItem("codeskimr.theme", "neon");
    expect((await freshTheme()).chosenTheme()).toBeNull();
  });

  it("follows the system again after the choice is forgotten", async () => {
    const theme = await freshTheme();
    theme.selectTheme("hell");
    theme.followSystem();
    expect(localStorage.getItem("codeskimr.theme")).toBeNull();
    expect(document.documentElement.getAttribute("data-theme")).toBe("petrol");
  });

  it("takes the default theme of the server only for users without a choice", async () => {
    const theme = await freshTheme();
    theme.useDefaultTheme("hell");
    expect(theme.activeTheme()).toBe("hell");
    theme.useDefaultTheme("neon");
    expect(theme.activeTheme()).toBe("hell");
    theme.selectTheme("dunkel");
    theme.useDefaultTheme("petrol");
    expect(theme.activeTheme()).toBe("dunkel");
  });

  it("works where localStorage cannot be used", async () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    const theme = await freshTheme();
    theme.selectTheme("hell");
    expect(theme.activeTheme()).toBe("hell");
    vi.restoreAllMocks();
  });
});
