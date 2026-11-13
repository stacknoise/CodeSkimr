import { beforeEach, describe, expect, it, vi } from "vitest";

const run = vi.hoisted(() => vi.fn());
const open = vi.hoisted(() => vi.fn());
const rows = vi.hoisted(() => vi.fn());
const config = vi.hoisted(() => vi.fn());

vi.mock("../../shared/api/commands", () => ({ runCommand: run }));
vi.mock("../../shared/editor/editor", async (original) => ({
  ...(await original<object>()),
  openLink: open,
}));
vi.mock("../projects/projects", () => ({ projects: rows }));
vi.mock("../../shared/settings/ui-config", () => ({ uiConfig: config }));

import { openInEditor } from "./editor";

const SETTINGS = {
  editor: { url_template: "vscode://file/{path}:{line}", allowed_schemes: ["vscode"] },
  default_theme: "petrol",
};
const SYMBOL = { path: "src/a b.cs", first_line: 12 };

describe("openInEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    run.mockResolvedValue(SYMBOL);
    config.mockReturnValue(SETTINGS);
    rows.mockReturnValue([{ id: 7, kind: "folder", location: "/work/app/" }]);
  });

  it("opens the declaration of the symbol at its line", async () => {
    await openInEditor(7, 3);
    expect(open).toHaveBeenCalledWith("vscode://file//work/app/src/a%20b.cs:12");
  });

  it("does nothing for a Git project or while the settings are not loaded", async () => {
    rows.mockReturnValue([{ id: 7, kind: "git", location: "https://example.test/x.git" }]);
    await openInEditor(7, 3);
    config.mockReturnValue(null);
    rows.mockReturnValue([{ id: 7, kind: "folder", location: "/work/app" }]);
    await openInEditor(7, 3);
    expect(open).not.toHaveBeenCalled();
  });
});
