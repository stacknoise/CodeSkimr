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
    runCommand.mockResolvedValueOnce({ theme: "dunkel", language: "de" });
    preferences.syncPreferences();
    setState({ kind: "connected" });
    await settle();
    expect(theme.chosenTheme()).toBe("dunkel");
    expect(i18n.currentLanguage()).toBe("de");
    expect(localStorage.getItem("codeskimr.theme")).toBe("dunkel");
    expect(runCommand).toHaveBeenCalledTimes(1);
  });

  it("sends the choice of this browser if the server holds none", async () => {
    const { preferences, theme } = await fresh();
    theme.selectTheme("petrol");
    runCommand.mockResolvedValueOnce({ theme: null, language: null });
    runCommand.mockResolvedValueOnce({ theme: "petrol", language: null });
    preferences.syncPreferences();
    setState({ kind: "connected" });
    await settle();
    expect(runCommand).toHaveBeenLastCalledWith("preferences.set", {
      theme: "petrol",
      language: null,
    });
  });

  it("sends a later choice and ignores names the UI does not offer", async () => {
    const { preferences, theme } = await fresh();
    runCommand.mockResolvedValueOnce({ theme: "nonexistent", language: "xx" });
    preferences.syncPreferences();
    setState({ kind: "connected" });
    await settle();
    expect(theme.chosenTheme()).toBeNull();
    runCommand.mockResolvedValueOnce({ theme: "hell", language: null });
    theme.selectTheme("hell");
    await settle();
    expect(runCommand).toHaveBeenLastCalledWith("preferences.set", {
      theme: "hell",
      language: null,
    });
  });

  it("keeps working with its own copy when the server does not answer", async () => {
    const { preferences, theme } = await fresh();
    runCommand.mockRejectedValue(new Error("down"));
    preferences.syncPreferences();
    setState({ kind: "connected" });
    await settle();
    theme.selectTheme("petrol");
    await settle();
    expect(theme.chosenTheme()).toBe("petrol");
    expect(localStorage.getItem("codeskimr.theme")).toBe("petrol");
  });
});
