import { describe, expect, it } from "vitest";
import { safeInScript } from "../../vite.export";

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
