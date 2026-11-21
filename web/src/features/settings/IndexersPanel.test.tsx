import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, serve, useServerSession } from "../../tests/commands";
import type { ToolsInfo } from "../projects/types";
import { IndexersPanel } from "./IndexersPanel";

const LISTING: ToolsInfo = {
  checked_at: 1_800_000_000,
  languages: [],
  tools: [
    {
      tool: "scip-typescript",
      role: "indexer",
      state: "ready",
      languages: ["typescript"],
      found: "0.6.2",
      required: "0.3.11",
      path: "C:\\npm\\scip-typescript.cmd",
      purpose_key: "tool-scip-typescript-purpose",
      install_key: "tool-scip-typescript-install",
    },
    {
      tool: "scip-python",
      role: "indexer",
      state: "not_runnable",
      languages: ["python"],
      found: "SyntaxError: Invalid regular expression",
      required: "0.6.0",
      path: "C:\\npm\\scip-python.cmd",
      purpose_key: "tool-scip-python-purpose",
      install_key: "tool-scip-python-install",
    },
    {
      tool: "scip-dotnet",
      role: "indexer",
      state: "missing",
      languages: ["csharp"],
      found: null,
      required: null,
      path: null,
      purpose_key: "tool-scip-dotnet-purpose",
      install_key: "tool-scip-dotnet-install",
    },
  ],
};

useServerSession();

beforeEach(() => setLanguage("en"));

describe("the indexers of this computer", () => {
  it("shows state, version, path and the error line of an indexer that does not start", async () => {
    serve({ "tools.indexers": () => answer(LISTING) });
    render(() => <IndexersPanel />);
    const panel = await screen.findByRole("region", { name: /Indexers for exact usages/ });
    await screen.findByText("scip-python");
    expect(panel).toHaveTextContent("found, but it does not start");
    expect(panel).toHaveTextContent("SyntaxError: Invalid regular expression");
    expect(panel).toHaveTextContent("C:\\npm\\scip-typescript.cmd");
    expect(panel).toHaveTextContent("0.6.2");
    expect(panel).toHaveTextContent("missing");
    expect(screen.getAllByRole("button", { name: "Copy" })).not.toHaveLength(0);
  });

  it("copies the install command", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    serve({ "tools.indexers": () => answer(LISTING) });
    render(() => <IndexersPanel />);
    await screen.findByText("scip-dotnet");
    const buttons = screen.getAllByRole("button", { name: "Copy" });
    fireEvent.click(buttons[buttons.length - 1] as HTMLElement);
    await waitFor(() =>
      expect(writeText).toHaveBeenCalledWith("dotnet tool install --global scip-dotnet"),
    );
  });

  it("says the same in German", async () => {
    setLanguage("de");
    serve({ "tools.indexers": () => answer(LISTING) });
    render(() => <IndexersPanel />);
    await screen.findByText("scip-python");
    expect(screen.getByText("gefunden, startet aber nicht")).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: "Erneut prüfen" })).toBeInTheDocument();
  });

  it("tells that the check got no answer", async () => {
    serve({ "tools.indexers": () => answer({}, 500) });
    render(() => <IndexersPanel />);
    expect(await screen.findByRole("alert")).toHaveTextContent("could not be checked");
  });
});
