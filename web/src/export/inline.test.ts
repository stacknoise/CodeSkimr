import { describe, expect, it } from "vitest";
import { safeInScript, swapFor, swaps } from "../../vite.export";

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
