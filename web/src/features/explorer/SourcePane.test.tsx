import { fireEvent, render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { openOrder, ORDER, OUTLINE } from "../../tests/explorer";
import { ExplorerScreen } from "./ExplorerScreen";
import type { SourceInfo } from "./types";

const TYPE_SOURCE: SourceInfo = {
  path: "src/Order.cs",
  first_line: 3,
  last_line: 6,
  available: true,
  lines: ["public class Order", "{", "    decimal Total() => 1m;", "}"],
  truncated: false,
};

useServerSession();

function renderWith(source: () => Response) {
  openProject(1);
  serve({
    "explorer.outline": () => answer(OUTLINE),
    "explorer.symbol": () => answer(ORDER),
    "explorer.source": source,
  });
  render(() => <ExplorerScreen />);
}

function asked(): unknown[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith("explorer.source"))
    .map(([, init]) => JSON.parse((init as RequestInit).body as string));
}

describe("the source column", () => {
  it("asks to pick something while nothing is selected", async () => {
    renderWith(() => answer(TYPE_SOURCE));
    expect(await screen.findByText("Pick a type or a member to see its source.")).toBeTruthy();
    expect(asked()).toEqual([]);
  });

  it("shows file, position and numbered code of the selected type", async () => {
    renderWith(() => answer(TYPE_SOURCE));
    await openOrder();
    const pane = await screen.findByRole("region", { name: "Source" });
    expect(await screen.findByText("src/Order.cs")).toBeTruthy();
    expect(pane.textContent).toContain("3–6");
    expect(pane.textContent).toContain("decimal Total() => 1m;");
    expect(pane.querySelector("[aria-hidden='true']")?.textContent).toBe("3\n4\n5\n6");
    expect(asked()).toEqual([{ project: 1, symbol: 7 }]);
  });

  it("shows the lines of a member when it is picked", async () => {
    renderWith((): Response => answer(TYPE_SOURCE));
    await openOrder();
    await screen.findByText("src/Order.cs");
    serve({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer(ORDER),
      "explorer.source": () =>
        answer({
          ...TYPE_SOURCE,
          first_line: 5,
          last_line: 5,
          lines: ["    decimal Total() => 1m;"],
        }),
    });
    fireEvent.click(await screen.findByRole("button", { name: /Total/ }));
    const pane = screen.getByRole("region", { name: "Source" });
    await screen.findByText("5", { selector: "pre[aria-hidden='true']" });
    expect(pane.textContent).toContain("decimal Total() => 1m;");
    expect(asked()).toContainEqual({ project: 1, symbol: 8 });
  });

  it("says so when the project is not a folder on this machine", async () => {
    renderWith(() => answer({ ...TYPE_SOURCE, available: false, lines: [] }));
    await openOrder();
    expect(
      await screen.findByText("The source of a Git project is not shown here yet."),
    ).toBeTruthy();
  });

  it("names the failure and keeps the rest of the screen", async () => {
    renderWith(() => answer({ code: "SRC-201" }, 404));
    await openOrder();
    const alerts = await screen.findAllByRole("alert");
    expect(alerts.length).toBeGreaterThan(0);
    expect(screen.getByRole("heading", { name: "Order" })).toBeTruthy();
  });

  it("says when the declaration was cut", async () => {
    renderWith(() => answer({ ...TYPE_SOURCE, truncated: true }));
    await openOrder();
    expect(await screen.findByText("Shows the first 4 lines of the declaration.")).toBeTruthy();
  });

  it("says when the file got shorter than the scan saw", async () => {
    renderWith(() => answer({ ...TYPE_SOURCE, lines: [] }));
    await openOrder();
    expect(
      await screen.findByText("The file is shorter than at the last scan. Scan the project again."),
    ).toBeTruthy();
  });
});
