import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { refreshProjects } from "../features/projects/projects";
import type { ProjectRow } from "../features/projects/types";
import { SESSION_PATH } from "../shared/api/consts";
import { setLanguage } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import { startSession } from "../shared/session/session";
import { answer, serve, useServerSession } from "../tests/commands";
import { TopBar } from "./Frame";
import { ProjectCrumb } from "./ProjectCrumb";

const FOLDER: ProjectRow = {
  id: 7,
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
};
const GIT: ProjectRow = {
  ...FOLDER,
  id: 8,
  name: "api",
  kind: "git",
  branch: "release/2",
};

useServerSession();

async function shown(id: number) {
  serve({
    [SESSION_PATH]: () => answer({}),
    "projects.list": () => answer([FOLDER, GIT]),
  });
  await startSession();
  await refreshProjects();
  openProject(id);
  return render(() => (
    <TopBar>
      <ProjectCrumb />
    </TopBar>
  ));
}

describe("ProjectCrumb", () => {
  it("names the project and the working folder of a folder source", async () => {
    setLanguage("en");
    await shown(7);
    expect(screen.getByRole("link", { name: "shop" })).toHaveAttribute("href", "/");
    expect(screen.getByText("Working folder")).toBeInTheDocument();
  });

  it("names the branch of a Git source", async () => {
    setLanguage("en");
    await shown(8);
    expect(screen.getByRole("link", { name: "api" })).toBeInTheDocument();
    expect(screen.getByText("release/2")).toBeInTheDocument();
  });
});
