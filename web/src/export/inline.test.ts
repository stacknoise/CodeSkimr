import { describe, expect, it } from "vitest";
import { safeInScript, swapFor, swapModules, swaps } from "../../vite.export";

describe("code that goes into the page", () => {
  it("cannot end the script element or open a comment", () => {
    const safe = safeInScript('a="</script><b>";b="<!--";c=/<\\/SCRIPT/');
    expect(safe.toLowerCase()).not.toContain("</script");
    expect(safe).not.toContain("<!--");
  });

  it("keeps the meaning of a string", () => {
    const safe = safeInScript('x="</script>"');
    expect(eval(safe.replace("x=", "(") + ")")).toBe("</script>");
  });
});

describe("the modules the export swaps", () => {
  it("are found by an id with backslashes too, as Windows writes it", () => {
    const table = swaps("/work/web/src");
    expect([...table.keys()].every((key) => !key.includes("\\"))).toBe(true);
    expect(swapFor(table, "/work/web/src/shared/api/commands.ts")).not.toBeNull();
    expect(swapFor(table, "\\work\\web\\src\\shared\\api\\commands.ts")).not.toBeNull();
    expect(swapFor(table, "/work/web/src/shared/api/client.ts")).toBeNull();
    const windows = new Map([["c:/work/web/src/app/router.ts", "own"]]);
    expect(swapFor(windows, "C:\\work\\web\\src\\app\\router.ts")).toBe("own");
    expect(swapFor(windows, "c:/work/web/src/app/router.ts")).toBe("own");
  });
});

describe("the swap of a module", () => {
  it("gets the id that the build gives the module of the swap anyway", async () => {
    const src = "/work/web/src";
    const plugin = swapModules(src);
    const canonical = "C:/work/web/src/export/commands.ts";
    const context = {
      resolve: async (source: string) =>
        source.startsWith(".")
          ? { id: `${src}/shared/api/commands.ts` }
          : { id: source.endsWith("export/commands.ts") ? canonical : source },
    };
    const resolveId = plugin.resolveId as (
      this: typeof context,
      source: string,
      importer: string,
      options: object,
    ) => Promise<string | null>;
    expect(await resolveId.call(context, "../shared/api/commands", `${src}/a.ts`, {})).toBe(
      canonical,
    );
    const other = { resolve: async () => ({ id: `${src}/shared/api/client.ts` }) };
    expect(await resolveId.call(other, "./client", `${src}/a.ts`, {})).toBeNull();
  });
});
