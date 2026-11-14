// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
import { addScannedProject, selectAdd, expect, test, watchPolicy } from "./fixtures";

const SOURCE = [
  "namespace Demo;",
  "",
  "public class Calculator",
  "{",
  "    public int Add(int a, int b) { return a + b; }",
  "}",
  "",
  "public class Checkout",
  "{",
  "    public int Total(Calculator calculator)",
  "    {",
  "        return calculator.Add(1, 2);",
  "    }",
  "}",
  "",
].join("\n");

test.describe("the usages screen", () => {
  test("U in the explorer shows who calls a member, marked as unchecked", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const row = await addScannedProject(page, harness, "usagesdemo", SOURCE);
    await selectAdd(page, row, "Demo");

    await page.keyboard.press("u");
    const graph = page.getByRole("region", { name: "Usage graph" });
    await expect(graph.getByText("Checkout.Total")).toBeVisible();
    await expect(page.getByText("Hits 1")).toBeVisible();
    await expect(page.getByText("unchecked", { exact: true })).toBeVisible();
    await expect(page.getByText(/No indexer is set up/)).toBeVisible();
    // The code line of the hit comes with the token of the page, the symbol is marked.
    const code = page.getByRole("region", { name: "Hits" }).locator("code");
    await expect(code).toHaveText("return calculator.Add(1, 2);");
    await expect(code.locator("mark")).toHaveText("Add");

    // Enter shows the caller in the explorer.
    await page.keyboard.press("F8");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("tree")).toBeVisible();
    expect(await violations()).toEqual([]);
  });

  test("keeps the hit list in view however many callers there are", async ({ page, harness }) => {
    const callers = Array.from(
      { length: 40 },
      (_, index) => `public class Caller${index} { int Run(Calculator c) { return c.Add(1, 2); } }`,
    );
    const source = [SOURCE, ...callers, ""].join("\n");
    const row = await addScannedProject(page, harness, "usagesmany", source);
    await selectAdd(page, row, "Demo");
    await page.keyboard.press("u");
    await expect(page.getByText("Hits 41")).toBeVisible();
    await expect(page.getByText("Number = places")).toBeVisible();
    const list = page.getByRole("region", { name: "Hits" });
    const box = await list.boundingBox();
    const viewport = page.viewportSize();
    expect(box).not.toBeNull();
    expect((box?.y ?? 0) + (box?.height ?? 0)).toBeLessThanOrEqual(viewport?.height ?? 0);
  });
});
