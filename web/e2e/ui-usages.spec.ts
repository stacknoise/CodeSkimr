// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
import path from "node:path";
import { addScannedProject, selectAdd, expect, shot, test, watchPolicy } from "./fixtures";

// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/usages-description/.
const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/usages-description");
const THEMES = ["hell", "petrol", "dunkel"];
const PATH_SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/usages-path");
const DOCUMENTED = [
  "namespace Demo;",
  "",
  "public class Calculator",
  "{",
  "    /// <summary>Adds two whole numbers and returns the sum.</summary>",
  "    public int Add(int a, int b) { return a + b; }",
  "}",
  "",
  "public class Checkout",
  "{",
  "    public int Total(Calculator calculator) { return calculator.Add(1, 2); }",
  "}",
  "",
].join("\n");

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
    await expect(graph.getByText("Total", { exact: true })).toBeVisible();
    await expect(page.getByText("Hits 1")).toBeVisible();
    await expect(page.getByText("unchecked (level 1)", { exact: true })).toBeVisible();
    await expect(page.getByText(/Level 2 has not checked these hits/)).toBeVisible();
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

  for (const theme of THEMES) {
    test(`lights the path of the clicked hit in the graph (${theme})`, async ({
      page,
      harness,
    }, info) => {
      await page.setViewportSize({ width: 1440, height: 900 });
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      const callers = ["Alpha", "Beta", "Gamma"].map(
        (name) => `public class ${name} { int Run(Calculator c) { return c.Add(1, 2); } }`,
      );
      const source = [SOURCE, ...callers, ""].join("\n");
      const row = await addScannedProject(page, harness, `usagespath${theme}`, source);
      await selectAdd(page, row, "Demo");
      await page.keyboard.press("u");
      const graph = page.getByRole("region", { name: "Usage graph" });
      await expect(page.getByText("Hits 4")).toBeVisible();
      await expect(graph.locator("button[class*=dim]")).toHaveCount(0);
      await page.getByRole("region", { name: "Hits" }).getByText("Demo.Beta.Run").click();
      await expect(graph.locator("button[class*=dim]")).toHaveCount(3);
      await expect(graph.locator('button[title="Demo.Beta.Run"]')).not.toHaveClass(/dim/);
      await shot(page, PATH_SHOTS, `usages-path-${theme}`, info);
    });
  }

  for (const theme of THEMES) {
    test(`shows the description of the code before the impact (${theme})`, async ({
      page,
      harness,
    }, info) => {
      await page.setViewportSize({ width: 1440, height: 900 });
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      const row = await addScannedProject(page, harness, `usagesdoc${theme}`, DOCUMENTED);
      await selectAdd(page, row, "Demo");
      await page.keyboard.press("u");
      await expect(page.getByRole("heading", { name: "Description" })).toBeVisible();
      await expect(page.getByText("Adds two whole numbers and returns the sum.")).toBeVisible();
      await shot(page, SHOTS, `usages-description-${theme}`, info);
    });
  }
});
