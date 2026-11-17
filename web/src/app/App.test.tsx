import { render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { openProject } from "../shared/project/active";
import { answer, serve, useServerSession } from "../tests/commands";
import { PROJECT_PATHS, PROJECTS_ROUTE, SETTINGS_ROUTE } from "./consts";
import { App } from "./App";
import { currentPath, navigate } from "./router";

useServerSession();

afterEach(() => {
  openProject(null as unknown as number);
  navigate(PROJECTS_ROUTE.path);
});

describe("App without an open project", () => {
  it.each(PROJECT_PATHS)("sends %s back to the projects", async (path) => {
    vi.stubGlobal(
      "WebSocket",
      class {
        close() {}
      },
    );
    serve({ "projects.list": () => answer([]), "status.info": () => answer({}) });
    navigate(path);
    render(() => <App />);
    await vi.waitFor(() => expect(currentPath()).toBe(PROJECTS_ROUTE.path));
  });

  it("leaves the settings alone", async () => {
    vi.stubGlobal(
      "WebSocket",
      class {
        close() {}
      },
    );
    serve({ "projects.list": () => answer([]), "status.info": () => answer({}) });
    navigate(SETTINGS_ROUTE.path);
    render(() => <App />);
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(currentPath()).toBe(SETTINGS_ROUTE.path);
  });
});
