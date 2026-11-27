import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { LogPanel } from "./LogPanel";

function status(level: string, resets: number | null = null) {
  return {
    level,
    default_level: "warn",
    levels: ["off", "warn", "info", "debug", "trace"],
    adjustable: true,
    file: "/data/logs/codeskimr.log",
    size_bytes: 2048,
    max_file_bytes: 10_485_760,
    files_kept: 5,
    resets_in_seconds: resets,
    can_open_folder: true,
  };
}

function sent(command: string): unknown[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith(command))
    .map(([, init]) => JSON.parse(((init as RequestInit).body as string) || "null"));
}

useServerSession();

beforeEach(() => setLanguage("en"));

describe("the log panel", () => {
  it("shows the level, the file and the last lines", async () => {
    serve({
      "log.status": () => answer(status("warn")),
      "log.tail": () => answer({ lines: ["13:21:08 WARN indexer tool missing"] }),
    });
    render(() => <LogPanel />);
    expect(await screen.findByRole("radio", { name: "Warnings" })).toBeChecked();
    expect(screen.getByText("/data/logs/codeskimr.log")).toBeInTheDocument();
    expect(await screen.findByText("13:21:08 WARN indexer tool missing")).toBeInTheDocument();
    expect(screen.getByText("at most 5 files of 10 MB")).toBeInTheDocument();
  });

  it("changes the level at once and says when the default comes back", async () => {
    serve({
      "log.status": () => answer(status("warn")),
      "log.tail": () => answer({ lines: [] }),
      "log.set_level": () => answer(status("info", 86_340)),
    });
    render(() => <LogPanel />);
    fireEvent.click(await screen.findByRole("radio", { name: "Info" }));
    await waitFor(() => expect(sent("log.set_level")).toEqual([{ level: "info" }]));
    expect(await screen.findByText("Back to the default in 23 h 59 min")).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Info" })).toBeChecked();
  });

  it("empties the log and offers no level when another program owns the log", async () => {
    serve({
      "log.status": () => answer({ ...status("warn"), adjustable: false }),
      "log.tail": () => answer({ lines: ["a line"] }),
      "log.clear": () => answer(status("warn")),
    });
    render(() => <LogPanel />);
    await waitFor(() => expect(screen.getByRole("radio", { name: "Info" })).toBeDisabled());
    await screen.findByText("a line");
    fireEvent.click(screen.getByRole("button", { name: "Empty file" }));
    expect(await screen.findByText("Emptied")).toBeInTheDocument();
    expect(sent("log.clear")).toHaveLength(1);
  });
});
