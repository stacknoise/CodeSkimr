import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { chooseProject, chooseTab, SOURCE_LABEL } from "../../tests/pageTabs";
import { refreshProjects } from "./projects";
import { ProjectsScreen } from "./ProjectsScreen";
import type { ScopeAnswer } from "./scope";
import type { ProjectRow } from "./types";

const TAB_SCOPE = /^(Scope|Umfang)$/;

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
  languages: [],
  running: false,
  tools: null,
};

const SCOPE: ScopeAnswer = {
  project: 1,
  live: true,
  scope: { languages: [], folders: ["vendor"] },
  languages: [
    { language: "typescript", files: 120, covered: true },
    { language: "rust", files: 38, covered: true },
  ],
  folders: [
    {
      path: "cli",
      files: 38,
      covered: true,
      left_out: false,
      languages: [{ language: "rust", files: 38 }],
      project_files: ["Cargo.toml"],
    },
    {
      path: "vendor",
      files: 900,
      covered: false,
      left_out: true,
      languages: [{ language: "typescript", files: 900 }],
      project_files: [],
    },
    {
      path: "vendor/lib",
      files: 10,
      covered: false,
      left_out: false,
      languages: [{ language: "typescript", files: 10 }],
      project_files: [],
    },
  ],
  folders_more: 0,
  root_project_files: ["package.json"],
};

useServerSession();

async function openScope(): Promise<void> {
  serve({
    "projects.list": () => answer([ROW]),
    "scope.get": () => answer(SCOPE),
    "scope.set": () => answer({ project: 1, scope: SCOPE.scope, changed: true }),
    "projects.rescan": () => answer({ project: 1 }),
  });
  await refreshProjects();
  render(() => <ProjectsScreen />);
  await chooseProject();
  await chooseTab(TAB_SCOPE);
  await screen.findByRole("checkbox", { name: /Rust/ });
}

function calls(id: string): unknown[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith(`/${id}`))
    .map(([, init]) => JSON.parse(String((init as RequestInit).body)));
}

describe("the scope of a project", () => {
  it("shows languages, folders with their level 2 files and what is left out", async () => {
    await openScope();
    expect(screen.getByRole("checkbox", { name: /TypeScript/ })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: /vendor$/ })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name: /lib$/ })).toBeDisabled();
    expect(screen.getByText("Level 2: Cargo.toml")).toBeInTheDocument();
    expect(screen.getByText("1 of 3 folders")).toBeInTheDocument();
    expect(screen.getByText(/A project of its own is not needed/)).toBeInTheDocument();
  });

  it("names the consequences of a language that is left out and saves, then scans", async () => {
    await openScope();
    expect(screen.getByRole("button", { name: "Save & rescan" })).toBeDisabled();
    fireEvent.click(screen.getByRole("checkbox", { name: /Rust/ }));
    expect(await screen.findByText(/38 files in Rust disappear/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Apply & rescan" }));
    await waitFor(() => expect(calls("scope.set")).toHaveLength(1));
    expect(calls("scope.set")[0]).toEqual({ project: 1, languages: ["rust"], folders: ["vendor"] });
    await waitFor(() => expect(calls("projects.rescan")).toHaveLength(1));
  });

  it("takes all folders back in with the root row and resets a change", async () => {
    await openScope();
    fireEvent.click(screen.getByRole("checkbox", { name: "Take all folders back in" }));
    expect(await screen.findByText(/Taken back in/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(screen.queryByText(/Taken back in/)).not.toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: /vendor$/ })).not.toBeChecked();
  });

  it("hints in the window New project that a folder lies in a project, and does not block", async () => {
    serve({ "projects.list": () => answer([ROW]), "projects.add": () => answer({}) });
    await refreshProjects();
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "New" }));
    fireEvent.input(screen.getByLabelText(SOURCE_LABEL), { target: { value: "/work/shop/cli" } });
    expect(await screen.findByText(/already in the project "shop"/)).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /Cover it in the project "shop"/ })).toBeChecked();
    fireEvent.click(screen.getByRole("radio", { name: /project of its own anyway/ }));
    expect(screen.getByRole("button", { name: "Add and scan" })).toBeEnabled();
  });
});
