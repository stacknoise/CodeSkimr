import { render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import { DATA } from "../tests/export-data";
import { installData } from "./commands";
import { ExportApp } from "./ExportApp";

vi.mock("../shared/api/commands", async () => {
  const offline = await import("./commands");
  return { runCommand: offline.runCommand };
});

beforeEach(() => {
  setLanguage("en");
  installData(DATA);
  openProject(DATA.project.id);
});

describe("ExportApp", () => {
  it("says which scan it is and that no usage has been verified", async () => {
    render(() => <ExportApp data={DATA} />);
    expect(screen.getByText(/Export of demo · scan 4/)).toBeInTheDocument();
    expect(screen.getByText(/0 verified, 3 unchecked/)).toBeInTheDocument();
    expect(screen.getByText(/source code is not included/)).toBeInTheDocument();
  });

  it("shows the tree from the data without asking any server", async () => {
    const fetched = vi.spyOn(globalThis, "fetch");
    render(() => <ExportApp data={DATA} />);
    expect(await screen.findByText("(no layer)")).toBeInTheDocument();
    expect(fetched).not.toHaveBeenCalled();
  });
});
