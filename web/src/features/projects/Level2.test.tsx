import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, fetchMock, useServerSession } from "../../tests/commands";
import { resetLevel2 } from "./level2";
import { ProjectsScreen } from "./ProjectsScreen";
import type { ProjectRow } from "./types";

const ROW: ProjectRow = {
  id: 4,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: null,
  types: 0,
  loc: 0,
  languages: [],
  running: false,
  tools: null,
};

const CONFIRM = "/api/v1/level2/confirm-window";
const POLL_MS = 2000;

/** The fake of the server until the real one exists: its answers follow ADR 0071. */
interface Fake {
  status: Record<string, unknown>;
  window: () => Response;
  revoked: number;
  confirms: number;
}

function fake(status: Fake["status"], window: Fake["window"] = () => answer({ launch: "window" })) {
  const state: Fake = { status, window, revoked: 0, confirms: 0 };
  fetchMock.mockImplementation((path: string) => {
    if (path === CONFIRM) {
      state.confirms += 1;
      return Promise.resolve(state.window());
    }
    const id = path.replace("/api/v1/commands/", "");
    if (id === "projects.list") {
      return Promise.resolve(answer([ROW]));
    }
    if (id === "level2.status") {
      return Promise.resolve(answer(state.status));
    }
    if (id === "level2.revoke") {
      state.revoked += 1;
      state.status = { ...state.status, state: "off", confirmed: false };
      return Promise.resolve(answer({}));
    }
    return Promise.resolve(answer({ code: "API-201" }, 404));
  });
  return state;
}

const WINDOW = { state: "off", confirmed: false, launch: "window" };
const COPY = { ...WINDOW, launch: "copy" };

useServerSession();
afterEach(() => {
  vi.useRealTimers();
  resetLevel2();
});

const FAKE_TIMERS = ["setInterval", "clearInterval", "setTimeout", "clearTimeout", "Date"] as const;

/** Switches on, lets the longest wait pass and expects the notice "not confirmed" with a free switch. */
async function waitUntilTimedOut(toggle: () => HTMLElement): Promise<void> {
  vi.useFakeTimers({ toFake: [...FAKE_TIMERS] });
  fireEvent.click(toggle());
  await screen.findByText("Finish the confirmation in the console window");
  await vi.advanceTimersByTimeAsync(301_000);
  expect(await screen.findByText(/did not arrive/)).toBeInTheDocument();
  expect(screen.getByText(/You can try again/)).toBeInTheDocument();
  expect(toggle()).toHaveAttribute("aria-disabled", "false");
}

/** Shows the screen with the project selected, so that its level 2 section is there. */
async function theSwitch(): Promise<() => HTMLElement> {
  // The list is read again after the first render, which replaces the row: ask for the switch
  // each time instead of keeping an element that may be gone.
  await screen.findByRole("switch", { name: /(Level 2|Stufe 2) (for|für) shop/ });
  fireEvent.click(await screen.findByText("/work/shop"));
  return () => screen.getByRole("switch", { name: /(Level 2|Stufe 2) (for|für) shop/ });
}

async function loaded(): Promise<() => HTMLElement> {
  render(() => <ProjectsScreen />);
  const toggle = await theSwitch();
  await waitFor(() => expect(toggle()).toHaveAttribute("aria-disabled", "false"));
  return toggle;
}

describe("the level 2 switch: switching on", () => {
  it("asks the server for the console window and waits for the confirmation", async () => {
    const server = fake(WINDOW);
    const toggle = await loaded();
    expect(toggle()).toHaveAttribute("aria-checked", "false");
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    fireEvent.click(toggle());
    expect(await screen.findByText("Finish the confirmation in the console window")).toBeVisible();
    expect(server.confirms).toBe(1);
    const call = fetchMock.mock.calls.find(([path]) => path === CONFIRM) as [string, RequestInit];
    expect(JSON.parse(call[1].body as string)).toEqual({ project_id: ROW.id });
    // The page learns the result only by asking again; typing the name happens in the window.
    server.status = { ...WINDOW, state: "on", confirmed: true };
    await vi.advanceTimersByTimeAsync(POLL_MS);
    await waitFor(() => expect(toggle()).toHaveAttribute("aria-checked", "true"));
    expect(screen.queryByText("Finish the confirmation in the console window")).toBeNull();
    expect(screen.getByText(/Level 2 is on\./)).toBeInTheDocument();
  });

  it("keeps the focus on the switch when the button Switch on disappears", async () => {
    fake(WINDOW);
    const toggle = await loaded();
    fireEvent.click(await screen.findByRole("button", { name: "Switch on" }));
    await screen.findByText("Finish the confirmation in the console window");
    expect(screen.queryByRole("button", { name: "Switch on" })).toBeNull();
    expect(toggle()).toHaveFocus();
  });

  it("does not open a second window while the first one waits", async () => {
    const server = fake(WINDOW);
    const toggle = await loaded();
    fireEvent.click(toggle());
    await screen.findByText("Finish the confirmation in the console window");
    expect(toggle()).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(toggle());
    expect(server.confirms).toBe(1);
  });

  it("stops asking after the longest wait and offers to switch on again", async () => {
    fake(WINDOW);
    await waitUntilTimedOut(await loaded());
  });

  it("ends the wait even when the question for the state never gets an answer", async () => {
    const server = fake(WINDOW);
    const toggle = await loaded();
    const answerStatus = fetchMock.getMockImplementation();
    fetchMock.mockImplementation((path: string) =>
      path.endsWith("level2.status") && server.confirms > 0
        ? new Promise<Response>(() => undefined)
        : answerStatus?.(path),
    );
    await waitUntilTimedOut(toggle);
  });

  it("cancels the wait: the notice goes, the asking stops and the switch is free", async () => {
    const server = fake(WINDOW);
    const toggle = await loaded();
    fireEvent.click(toggle());
    await screen.findByText("Finish the confirmation in the console window");
    expect(screen.getByText(/pause of 30 seconds applies/)).toBeInTheDocument();
    expect(toggle()).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(screen.getByRole("button", { name: "Cancel waiting" }));
    expect(screen.queryByText("Finish the confirmation in the console window")).toBeNull();
    expect(screen.getByText(/Waiting ended/)).toBeInTheDocument();
    expect(toggle()).toHaveAttribute("aria-disabled", "false");
    expect(toggle()).toHaveFocus();
    const asked = fetchMock.mock.calls.length;
    vi.useFakeTimers({ toFake: [...FAKE_TIMERS] });
    await vi.advanceTimersByTimeAsync(10 * POLL_MS);
    expect(fetchMock.mock.calls.length).toBe(asked);
    fireEvent.click(toggle());
    await screen.findByText("Finish the confirmation in the console window");
    expect(server.confirms).toBe(2);
  });

  it("cancels the wait with Escape", async () => {
    fake(WINDOW);
    const toggle = await loaded();
    fireEvent.click(toggle());
    const title = await screen.findByText("Finish the confirmation in the console window");
    fireEvent.keyDown(title, { key: "Escape" });
    expect(screen.getByText(/Waiting ended/)).toBeInTheDocument();
    expect(toggle()).toHaveAttribute("aria-disabled", "false");
  });

  it("shows the command with a copy button where no window can open", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    fake(COPY, () => answer({ launch: "copy", command: `codeskimr level2 confirm ${ROW.id}` }));
    fireEvent.click((await loaded())());
    expect(await screen.findByText(`codeskimr level2 confirm ${ROW.id}`)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() =>
      expect(writeText).toHaveBeenCalledWith(`codeskimr level2 confirm ${ROW.id}`),
    );
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("shows the error and the command when the window could not be started (APP-604)", async () => {
    fake(WINDOW, () => answer({ code: "APP-604" }, 500));
    fireEvent.click((await loaded())());
    expect(await screen.findByRole("alert")).toHaveTextContent("could not be started");
    expect(screen.getByText(`codeskimr level2 confirm ${ROW.id}`)).toBeVisible();
  });

  it("tells to wait 30 seconds after a window was opened a moment ago (API-504)", async () => {
    fake(WINDOW, () => answer({ code: "API-504" }, 429));
    fireEvent.click((await loaded())());
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("opened a moment ago");
    expect(alert).toHaveTextContent("Wait 30 seconds");
    expect(screen.queryByText("Finish the confirmation in the console window")).toBeNull();
  });

  it("goes by the code in the body, not by the status (CFG-105 as 409)", async () => {
    const server = fake(WINDOW, () => answer({ code: "CFG-105" }, 409));
    fireEvent.click((await loaded())());
    expect(await screen.findByRole("alert")).toHaveTextContent("not switched on");
    expect(server.confirms).toBe(1);
    expect(screen.queryByText(/Wait 30 seconds/)).toBeNull();
  });

  it("does not treat a 429 without the known code as a pause (API-504 only by code)", async () => {
    fake(WINDOW, () => answer({ code: "APP-604" }, 429));
    fireEvent.click((await loaded())());
    expect(await screen.findByRole("alert")).toHaveTextContent("could not be started");
    expect(screen.queryByText(/Wait 30 seconds/)).toBeNull();
    expect(screen.getByText(`codeskimr level2 confirm ${ROW.id}`)).toBeVisible();
  });
});

describe("the level 2 switch: switching off", () => {
  const ON = { state: "on", confirmed: true, launch: "window" };

  it("asks first, and Escape leaves the question with the focus on the switch", async () => {
    const server = fake(ON);
    const toggle = await loaded();
    expect(toggle()).toHaveAttribute("aria-checked", "true");
    fireEvent.click(toggle());
    const ask = await screen.findByRole("group", { name: "Switch off level 2 for shop?" });
    expect(ask).toHaveTextContent("Checked results stay until the next scan");
    expect(screen.getByRole("button", { name: "Yes, switch off" })).toHaveFocus();
    fireEvent.keyDown(ask, { key: "Escape" });
    await waitFor(() => expect(toggle()).toHaveFocus());
    expect(server.revoked).toBe(0);
    expect(toggle()).toHaveAttribute("aria-checked", "true");
  });

  it("revokes after the answer yes and says that checked results stay", async () => {
    const server = fake(ON);
    const toggle = await loaded();
    fireEvent.click(toggle());
    fireEvent.click(await screen.findByRole("button", { name: "Yes, switch off" }));
    await waitFor(() => expect(toggle()).toHaveAttribute("aria-checked", "false"));
    expect(server.revoked).toBe(1);
    const notice = screen.getByText(/Level 2 is off\. Checked results stay/);
    expect(within(notice.parentElement as HTMLElement).getByRole("button")).toHaveTextContent(
      "Scan again",
    );
  });
});

describe("the level 2 switch: focus after the notice of a switch-off", () => {
  it("puts the focus on the switch when Scan again leaves", async () => {
    fake({ state: "on", confirmed: true, launch: "window" });
    const toggle = await loaded();
    fireEvent.click(toggle());
    fireEvent.click(await screen.findByRole("button", { name: "Yes, switch off" }));
    fireEvent.click(await screen.findByRole("button", { name: "Scan again" }));
    expect(screen.queryByRole("button", { name: "Scan again" })).toBeNull();
    expect(toggle()).toHaveFocus();
  });
});

describe("the level 2 switch: states that cannot be switched on", () => {
  it("not enabled: shows the two config lines, writes nothing and offers no click", async () => {
    const server = fake({
      state: "not_enabled",
      confirmed: false,
      launch: "window",
    });
    render(() => <ProjectsScreen />);
    const toggle = await theSwitch();
    await waitFor(() => expect(screen.getByText(/never changes this setting/)).toBeVisible());
    expect(screen.getByText(/\[level2\]/)).toHaveTextContent("enabled = true");
    expect(screen.getByText("File: in your config.toml in the data folder")).toBeVisible();
    expect(screen.queryByText(/\/home\/me/)).toBeNull();
    expect(toggle()).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(toggle());
    expect(server.confirms).toBe(0);
  });

  it("not enabled with a confirmation: switching off stays possible", async () => {
    fake({ state: "not_enabled", confirmed: true, launch: "window" });
    render(() => <ProjectsScreen />);
    const toggle = await theSwitch();
    await waitFor(() => expect(toggle()).toHaveAttribute("aria-checked", "true"));
    expect(toggle()).toHaveAttribute("aria-disabled", "false");
  });

  it("locked: says why and offers nothing", async () => {
    const server = fake({ state: "locked", confirmed: false, launch: "copy" });
    render(() => <ProjectsScreen />);
    const toggle = await theSwitch();
    await waitFor(() => expect(screen.getByText(/team server does not run/)).toBeVisible());
    fireEvent.click(toggle());
    expect(server.confirms).toBe(0);
    expect(screen.queryByRole("button", { name: "Switch on" })).toBeNull();
  });

  it("says the same in German", async () => {
    setLanguage("de");
    fake(WINDOW);
    render(() => <ProjectsScreen />);
    const toggle = await theSwitch();
    await waitFor(() => expect(toggle()).toHaveAttribute("aria-disabled", "false"));
    fireEvent.click(toggle());
    expect(await screen.findByText("Bestätigung im Konsolenfenster abschließen")).toBeVisible();
  });
});
