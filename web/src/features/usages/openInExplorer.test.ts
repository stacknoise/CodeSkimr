import { afterEach, describe, expect, it } from "vitest";
import { navigate } from "../../app/router";
import { pendingFocus, requestFocus } from "../../shared/navigation/navigation";
import { answer, serve, useServerSession } from "../../tests/commands";
import { openInExplorer } from "./usages";

useServerSession();
afterEach(() => {
  requestFocus(null);
  navigate("/");
});

const PLACE = { id: 11, qualified_name: "A.B.run", kind: "method", layer: null };

function symbolWith(breadcrumb: Array<{ id: number; name: string; kind: string }>) {
  serve({ "explorer.symbol": () => answer({ breadcrumb }) });
}

describe("openInExplorer", () => {
  it("opens a member in the nearest type of its path", async () => {
    symbolWith([
      { id: 1, name: "A", kind: "namespace" },
      { id: 3, name: "Outer", kind: "class" },
      { id: 4, name: "Inner", kind: "class" },
    ]);
    await openInExplorer(1, PLACE);
    expect(pendingFocus()).toEqual({ type: 4, member: 11, kind: "method" });
  });

  it("skips a function that stands between the member and its type", async () => {
    symbolWith([
      { id: 3, name: "Outer", kind: "class" },
      { id: 5, name: "helper", kind: "function" },
    ]);
    await openInExplorer(1, PLACE);
    expect(pendingFocus()).toEqual({ type: 3, member: 11, kind: "method" });
  });

  it("opens the symbol itself when no type is in its path", async () => {
    symbolWith([{ id: 1, name: "A", kind: "namespace" }]);
    await openInExplorer(1, PLACE);
    expect(pendingFocus()).toEqual({ type: 11, member: null, kind: "method" });
  });
});
