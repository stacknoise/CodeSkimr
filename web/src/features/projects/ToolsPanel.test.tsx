import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { ProjectsScreen } from "./ProjectsScreen";
import type { ProjectRow, ToolsInfo } from "./types";

/** A C# project whose indexer is not installed and whose SDK is older than global.json asks. */
const MISSING: ToolsInfo = {
  checked_at: 1_800_000_000,
  languages: [{ language: "csharp", level2_ready: false }],
  tools: [
    {
      tool: "dotnet",
      role: "toolchain",
      state: "too_old",
      languages: ["csharp"],
      found: "8.0.404",
      required: "9.0.100",
      purpose_key: "tool-dotnet-purpose",
      install_key: "tool-dotnet-install",
    },
    {
      tool: "scip-dotnet",
      role: "indexer",
      state: "missing",
      languages: ["csharp"],
      found: null,
      required: null,
      purpose_key: "tool-scip-dotnet-purpose",
      install_key: "tool-scip-dotnet-install",
    },
  ],
};

const READY: ToolsInfo = {
  checked_at: 1_800_000_100,
  languages: [{ language: "rust", level2_ready: true }],
  tools: [{ ...(MISSING.tools[0] as ToolsInfo["tools"][number]), state: "ready", tool: "cargo" }],
};

const ROW: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: null,
  types: 0,
  loc: 0,
  languages: [{ language: "csharp", files: 3, loc: 38_000 }],
  running: false,
  tools: null,
};

useServerSession();

async function openWarning(): Promise<HTMLElement> {
  serve({
    "projects.list": () => answer([{ ...ROW, tools: MISSING }]),
    "projects.tools": () => answer(MISSING),
  });
  render(() => <ProjectsScreen />);
  fireEvent.click(await screen.findByText("/work/shop"));
  return screen.findByRole("status");
}

describe("the tools of a project", () => {
  it("warns about a missing indexer with its name, purpose and how to install it", async () => {
    const warning = await openWarning();
    expect(warning).toHaveTextContent("scip-dotnet");
    expect(warning).toHaveTextContent("missing");
    expect(warning).toHaveTextContent("exact usages of C# code");
    expect(warning).toHaveTextContent('"dotnet tool install --global scip-dotnet"');
    expect(warning).toHaveTextContent("too old: 8.0.404 installed, 9.0.100 required");
    expect(warning).toHaveTextContent("for C#");
    expect(screen.getByText("2 tools missing")).toBeInTheDocument();
  });

  it("says the same in German", async () => {
    setLanguage("de");
    const warning = await openWarning();
    expect(warning).toHaveTextContent("fehlt");
    expect(warning).toHaveTextContent("Installation mit");
    expect(screen.getByText("2 Werkzeuge fehlen")).toBeInTheDocument();
  });

  it("checks the tools of the project that is selected, and shows the fresh result", async () => {
    serve({
      "projects.list": () => answer([{ ...ROW, tools: null }]),
      "projects.tools": () => answer(MISSING),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByText("/work/shop"));
    await screen.findByRole("status");
    const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith("projects.tools"));
    expect(JSON.parse(call?.[1].body)).toEqual({ project: 1 });
  });

  it("lists the languages that have level 2 and no warning when all tools are there", async () => {
    serve({
      "projects.list": () => answer([{ ...ROW, tools: READY }]),
      "projects.tools": () => answer(READY),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByText("/work/shop"));
    expect(await screen.findByText("Level 2 can run for: Rust")).toBeInTheDocument();
    expect(screen.getByText("All tools are there.")).toBeInTheDocument();
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("says that nothing was checked yet, and that a project has no level 2 language", async () => {
    serve({
      "projects.list": () => answer([ROW, { ...ROW, id: 2, name: "go-app", location: "/go" }]),
      "projects.tools": () =>
        answer({ checked_at: 1_800_000_200, languages: [], tools: [] } satisfies ToolsInfo),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByText("/go"));
    expect(await screen.findByText(/No language of this project has level 2/)).toBeInTheDocument();
  });

  it("keeps what the list says when the check itself fails, and can check again", async () => {
    let fail = true;
    serve({
      "projects.list": () => answer([{ ...ROW, id: 7, location: "/work/seven", tools: MISSING }]),
      "projects.tools": () => (fail ? answer({ code: "API-201" }, 404) : answer(READY)),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByText("/work/seven"));
    expect(await screen.findByRole("status")).toHaveTextContent("scip-dotnet");
    const again = await screen.findByRole("button", { name: "Check again" });
    fail = false;
    fireEvent.click(again);
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    expect(screen.getByText("Level 2 can run for: Rust")).toBeInTheDocument();
  });

  it("keeps the warning after a rescan, which does not check the tools again", async () => {
    serve({
      "projects.list": () => answer([{ ...ROW, tools: MISSING }]),
      "projects.tools": () => answer(MISSING),
      "projects.rescan": () =>
        answer({
          project: 1,
          snapshot: 6,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 0.2,
          tools: { checked_at: null, languages: [], tools: [] } satisfies ToolsInfo,
        }),
    });
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByText("/work/shop"));
    await screen.findByRole("status");
    fireEvent.keyDown(document, { key: "r" });
    await waitFor(() =>
      expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.rescan"))).toBe(
        true,
      ),
    );
    await waitFor(() => expect(screen.getByText("2 tools missing")).toBeInTheDocument());
    expect(screen.getByRole("status")).toHaveTextContent("scip-dotnet");
  });
});
