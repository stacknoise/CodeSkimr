import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { answer, serve, useServerSession } from "../../tests/commands";
import { EffectivePanel } from "./EffectivePanel";
import type { EffectiveSettings } from "./effective";

const LISTING: EffectiveSettings = {
  user_file: { path: "/home/me/config.toml", found: true },
  overrides: ["level2.timeout_seconds=1800"],
  settings: [
    { key: "scan.max_workers", value: "2", origin: "user", hidden: false },
    { key: "scan.excluded_folders", value: '["bin", ".git"]', origin: "default", hidden: false },
    { key: "level2.enabled", value: "true", origin: "user", hidden: false },
    { key: "level2.timeout_seconds", value: "1800", origin: "cli", hidden: false },
    { key: "ui.default_theme", value: '"petrol"', origin: "default", hidden: false },
    { key: "server.port", value: "7421", origin: "default", hidden: false },
  ],
};

useServerSession();

async function shown() {
  serve({ "settings.effective": () => answer(LISTING) });
  render(() => <EffectivePanel />);
  await screen.findByText("scan.max_workers");
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
    await shown();
    expect(screen.getByText("/home/me/config.toml")).toBeInTheDocument();
    expect(screen.getByText("--set level2.timeout_seconds=1800")).toBeInTheDocument();
  });

  it("narrows the list by search and to the values that differ from the default", async () => {
    await shown();
    fireEvent.input(screen.getByLabelText("Search a setting …"), { target: { value: "port" } });
    expect(screen.getByText("server.port")).toBeInTheDocument();
    expect(screen.queryByText("scan.max_workers")).toBeNull();
    fireEvent.input(screen.getByLabelText("Search a setting …"), { target: { value: "" } });
    fireEvent.click(screen.getByLabelText("Only values that differ from the default"));
    expect(screen.queryByText("server.port")).toBeNull();
    expect(screen.getByText("3 of 6 settings shown")).toBeInTheDocument();
  });

  it("offers no way to change a value", async () => {
    await shown();
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("never shows the value of a hidden setting", async () => {
    serve({
      "settings.effective": () =>
        answer({
          ...LISTING,
          settings: [{ key: "x.api_token", value: "", origin: "user", hidden: true }],
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
});
