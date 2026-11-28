import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { ConfigSourcesPanel, EffectivePanel } from "./EffectivePanel";
import { setEffectiveSearch } from "./effective";
import type { EffectiveSettings } from "./effective";

const LISTING: EffectiveSettings = {
  user_file: { path: "/home/me/config.toml", found: true },
  overrides: ["level2.timeout_seconds=1800"],
  settings: [
    {
      key: "scan.max_workers",
      value: "2",
      origin: "user",
      hidden: false,
      default: "0",
      edit: "scan",
      pending: null,
    },
    {
      key: "scan.excluded_folders",
      value: '["bin", ".git"]',
      origin: "default",
      hidden: false,
      default: "0",
      edit: "scan",
      pending: null,
    },
    {
      key: "level2.enabled",
      value: "true",
      origin: "user",
      hidden: false,
      default: "false",
      edit: "locked",
      pending: null,
    },
    {
      key: "level2.timeout_seconds",
      value: "1800",
      origin: "cli",
      hidden: false,
      default: "600",
      edit: "cli",
      pending: null,
    },
    {
      key: "ui.default_theme",
      value: '"petrol"',
      origin: "default",
      hidden: false,
      default: "0",
      edit: "scan",
      pending: null,
    },
    {
      key: "server.port",
      value: "7421",
      origin: "default",
      hidden: false,
      default: "7421",
      edit: "locked",
      pending: null,
    },
  ],
};

useServerSession();

beforeEach(() => setEffectiveSearch(""));

async function shown() {
  serve({ "settings.effective": () => answer(LISTING) });
  render(() => <EffectivePanel />);
  await screen.findByText("scan.max_workers");
}

/** Shows the panel for a value that is saved and waits for the next start. */
function renderPending() {
  serve({
    "settings.effective": () =>
      answer({
        ...LISTING,
        settings: [{ ...LISTING.settings[0]!, edit: "start", pending: "500" }],
      }),
  });
  render(() => <EffectivePanel />);
}

describe("EffectivePanel", () => {
  it("shows each value in words with the layer that set it", async () => {
    await shown();
    expect(screen.getByText("bin, .git")).toBeInTheDocument();
    expect(screen.getByText("petrol")).toBeInTheDocument();
    expect(screen.getByText("on")).toBeInTheDocument();
    expect(screen.getByText("1800 s")).toBeInTheDocument();
    expect(screen.getByText("Command line")).toBeInTheDocument();
    expect(screen.getAllByText("config.toml")).toHaveLength(2);
    expect(screen.getByText("6 of 6 settings shown")).toBeInTheDocument();
  });

  it("names the file that was read and the flags of the start", async () => {
    serve({ "settings.effective": () => answer(LISTING) });
    render(() => <ConfigSourcesPanel />);
    await screen.findByText("/home/me/config.toml");
    expect(screen.getByText("/home/me/config.toml")).toBeInTheDocument();
    expect(screen.getByText("--set level2.timeout_seconds=1800")).toBeInTheDocument();
  });

  it("narrows the list by search and to the values that differ from the default", async () => {
    await shown();
    setEffectiveSearch("port");
    expect(await screen.findByText("server.port")).toBeInTheDocument();
    expect(screen.queryByText("scan.max_workers")).toBeNull();
    setEffectiveSearch("");
    fireEvent.click(screen.getByLabelText("Only values that differ from the default"));
    expect(screen.queryByText("server.port")).toBeNull();
    expect(screen.getByText("3 of 6 settings shown")).toBeInTheDocument();
  });

  it("greys out the change button of a value that is locked or set by a flag, with the reason", async () => {
    await shown();
    const buttons = screen.getAllByRole("button", { name: "Change" });
    expect(buttons).toHaveLength(6);
    const locked = buttons.filter((button) => (button as HTMLButtonElement).disabled);
    expect(locked.map((button) => button.getAttribute("title"))).toEqual([
      "Security relevant: only in the configuration file",
      "Set with --set at the start, the command line wins",
      "Security relevant: only in the configuration file",
    ]);
  });

  it("saves a new number from the dialog and reads the settings again", async () => {
    let value = "2";
    serve({
      "settings.effective": () =>
        answer({
          ...LISTING,
          settings: [{ ...LISTING.settings[0]!, value, origin: "ui" }],
        }),
      "ui_settings.set_value": () => {
        value = "5";
        return answer({ indexer_memory_mb: {} });
      },
    });
    render(() => <EffectivePanel />);
    await waitFor(() => expect(screen.getAllByRole("button", { name: "Change" })).toHaveLength(1));
    fireEvent.click(screen.getByRole("button", { name: "Change" }));
    const field = screen.getByLabelText("New value");
    expect(screen.getByText("Applies from the next scan")).toBeInTheDocument();
    fireEvent.input(field, { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(screen.getByText("5")).toBeInTheDocument();
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("set_value"))!;
    expect(JSON.parse((call[1] as RequestInit).body as string)).toEqual({
      key: "scan.max_workers",
      value: 5,
    });
  });

  it("refuses an entry that is no whole number without asking the server", async () => {
    await shown();
    fireEvent.click(screen.getAllByRole("button", { name: "Change" })[0]!);
    fireEvent.input(screen.getByLabelText("New value"), { target: { value: "many" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("whole number");
    expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("set_value"))).toBe(false);
  });

  it("shows the registered text when the server refuses the value", async () => {
    serve({
      "settings.effective": () => answer(LISTING),
      "ui_settings.set_value": () => answer({ code: "CFG-101" }, 400),
    });
    render(() => <EffectivePanel />);
    await screen.findByText("scan.max_workers");
    fireEvent.click(screen.getAllByRole("button", { name: "Change" })[0]!);
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("offers the saved value and Back to default for a value that waits for the next start", async () => {
    renderPending();
    await waitFor(() => expect(screen.getAllByRole("button", { name: "Change" })).toHaveLength(1));
    fireEvent.click(screen.getByRole("button", { name: "Change" }));
    expect(screen.getByLabelText("New value")).toHaveValue("500");
    expect(screen.getByRole("button", { name: "Back to default" })).toBeEnabled();
  });

  it("notes a value that is saved and waits for the next start", async () => {
    renderPending();
    expect(
      await screen.findByText("Saved: 500, applies from the next start of the app"),
    ).toBeInTheDocument();
  });

  it("never shows the value of a hidden setting", async () => {
    serve({
      "settings.effective": () =>
        answer({
          ...LISTING,
          settings: [
            {
              key: "x.api_token",
              value: "",
              origin: "user",
              hidden: true,
              default: "",
              edit: "locked",
              pending: null,
            },
          ],
        }),
    });
    render(() => <EffectivePanel />);
    expect(await screen.findByText("hidden")).toBeInTheDocument();
  });

  it("says so when the settings cannot be loaded", async () => {
    serve({});
    render(() => <EffectivePanel />);
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
  });

  it("says so on the tab Info when the configuration file cannot be read", async () => {
    serve({});
    render(() => <ConfigSourcesPanel />);
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
  });
});
