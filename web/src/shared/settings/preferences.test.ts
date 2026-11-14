import { beforeEach, describe, expect, it, vi } from "vitest";

const runCommand = vi.fn();

vi.mock("../api/commands", () => ({ runCommand: (...args: unknown[]) => runCommand(...args) }));

let setState: (state: { kind: string }) => void = () => undefined;

async function fresh() {
  vi.resetModules();
  // The state of the session is a signal of the same Solid as the modules under test, so the
  // mock is made again after every reset.
  vi.doMock("../session/session", async () => {
    const { createSignal } = await import("solid-js");
    const [sessionState, setSessionState] = createSignal<{ kind: string }>({ kind: "connecting" });
    return { sessionState, setSessionState };
  });
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
  const session = (await import("../session/session")) as unknown as {
    setSessionState: typeof setState;
  };
  setState = session.setSessionState;
  const preferences = await import("./preferences");
  const theme = await import("../theme/theme");
  const i18n = await import("../i18n/i18n");
  return { preferences, theme, i18n };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const sets = () => runCommand.mock.calls.filter(([id]) => id === "preferences.set");

/** Answers `preferences.get` with `server`, and every `preferences.set` with what it was given. */
function serverHolds(server: { theme: string | null; language: string | null }) {
  runCommand.mockImplementation((id: string, input: unknown) =>
    Promise.resolve(id === "preferences.get" ? server : input),
  );
}

async function connect(preferences: { syncPreferences: () => void }) {
  preferences.syncPreferences();
  setState({ kind: "connected" });
  await settle();
}

beforeEach(() => {
  runCommand.mockReset();
});

describe("preferences on the server", () => {
  it("reads nothing before the session is connected", async () => {
    const { preferences } = await fresh();
    preferences.syncPreferences();
    await settle();
    expect(runCommand).not.toHaveBeenCalled();
  });

  it("applies what the server holds, also over the copy of this browser", async () => {
    const { preferences, theme, i18n } = await fresh();
    theme.selectTheme("hell");
    serverHolds({ theme: "dunkel", language: "de" });
    await connect(preferences);
    expect(theme.chosenTheme()).toBe("dunkel");
    expect(i18n.currentLanguage()).toBe("de");
    expect(localStorage.getItem("codeskimr.theme")).toBe("dunkel");
    expect(sets()).toHaveLength(0);
  });

  it("sends the choice of this browser if the server holds none", async () => {
    const { preferences, theme } = await fresh();
    theme.selectTheme("petrol");
    serverHolds({ theme: null, language: null });
    await connect(preferences);
    expect(sets()).toEqual([["preferences.set", { theme: "petrol", language: null }]]);
  });

  it("takes over a part and sends the other part that only this browser has", async () => {
    const { preferences, theme, i18n } = await fresh();
    theme.selectTheme("petrol");
    serverHolds({ theme: null, language: "de" });
    await connect(preferences);
    expect(i18n.currentLanguage()).toBe("de");
    expect(theme.chosenTheme()).toBe("petrol");
    expect(sets()).toEqual([["preferences.set", { theme: "petrol", language: "de" }]]);
  });

  it("changing only the theme does not erase a language another browser stored", async () => {
    const { preferences, theme } = await fresh();
    serverHolds({ theme: null, language: "de" });
    await connect(preferences);
    theme.selectTheme("hell");
    await settle();
    expect(sets()).toEqual([["preferences.set", { theme: "hell", language: "de" }]]);
  });

  it("ignores names the UI does not offer", async () => {
    const { preferences, theme } = await fresh();
    serverHolds({ theme: "nonexistent", language: "xx" });
    await connect(preferences);
    expect(theme.chosenTheme()).toBeNull();
    expect(sets()).toHaveLength(0);
  });

  it("ends with the last choice when changes come in while a call is under way", async () => {
    const { preferences, theme } = await fresh();
    serverHolds({ theme: null, language: null });
    await connect(preferences);
    const release: (() => void)[] = [];
    runCommand.mockImplementation((id: string, input: unknown) =>
      id === "preferences.set"
        ? new Promise((resolve) => release.push(() => resolve(input)))
        : Promise.resolve(input),
    );
    theme.selectTheme("hell");
    await settle();
    theme.selectTheme("dunkel");
    theme.selectTheme("petrol");
    await settle();
    expect(release).toHaveLength(1);
    release[0]?.();
    await settle();
    release[1]?.();
    await settle();
    expect(sets().map(([, input]) => (input as { theme: string }).theme)).toEqual([
      "hell",
      "petrol",
    ]);
  });

  it("reads the server again at the next change if it did not answer at start", async () => {
    const { preferences, theme } = await fresh();
    runCommand.mockRejectedValue(new Error("down"));
    await connect(preferences);
    theme.selectTheme("petrol");
    await settle();
    expect(theme.chosenTheme()).toBe("petrol");
    expect(localStorage.getItem("codeskimr.theme")).toBe("petrol");
    serverHolds({ theme: null, language: "de" });
    theme.selectTheme("hell");
    await settle();
    expect(sets()).toEqual([["preferences.set", { theme: "hell", language: "de" }]]);
  });
});
