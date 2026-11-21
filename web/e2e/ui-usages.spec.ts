// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
import path from "node:path";
import type { Page } from "@playwright/test";
import {
  addScannedProject,
  selectAdd,
  expect,
  shot,
  test,
  watchPolicy,
  type Harness,
} from "./fixtures";

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

/** The demo source plus one extra caller class per name (or `count` numbered ones). */
function withCallers(names: string[] | number): string {
  const list =
    typeof names === "number" ? Array.from({ length: names }, (_, i) => `Caller${i}`) : names;
  const callers = list.map(
    (name) => `public class ${name} { int Run(Calculator c) { return c.Add(1, 2); } }`,
  );
  return [SOURCE, ...callers, ""].join("\n");
}

/** Opens the usages screen of `Demo` for the given source in a wide window with the theme set. */
async function openThemed(
  page: Page,
  harness: Harness,
  theme: string,
  project: string,
  source: string,
) {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
  const row = await addScannedProject(page, harness, `${project}${theme}`, source);
  await selectAdd(page, row, "Demo");
  await page.keyboard.press("u");
}

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
    const row = await addScannedProject(page, harness, "usagesmany", withCallers(40));
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
      await openThemed(page, harness, theme, "usagespath", withCallers(["Alpha", "Beta", "Gamma"]));
      const graph = page.getByRole("region", { name: "Usage graph" });
      await expect(page.getByText("Hits 4")).toBeVisible();
      await expect(graph.locator("button[class*=dim]")).toHaveCount(0);
      await page.getByRole("region", { name: "Hits" }).getByText("Demo.Beta.Run").click();
      await expect(graph.locator("button[class*=dim]")).toHaveCount(3);
      await expect(graph.locator('button[title="Demo.Beta.Run"]')).not.toHaveClass(/dim/);
      await shot(page, PATH_SHOTS, `usages-path-${theme}`, info);
    });
  }

  test("the accuracy card says what the stored report knows about the language (ADR 0072)", async ({
    page,
    harness,
  }) => {
    const row = await addScannedProject(page, harness, "usagesreport", SOURCE);
    await selectAdd(page, row, "Demo");
    await page.keyboard.press("u");
    // The real report of a scan from the browser without a confirmation: stage 2 was skipped.
    await expect(
      page.getByText("Level 2 skipped C#: level 2 is not confirmed for this project."),
    ).toBeVisible();
  });

  test("the accuracy card tells a stage 2 run of the language that checked in part", async ({
    page,
    harness,
  }) => {
    await page.route("**/api/v1/commands/usages.query", async (route) => {
      const response = await route.fetch();
      const body = (await response.json()) as Record<string, unknown>;
      const level2 = {
        at: 1_700_000_000,
        origin: "browser",
        languages: [{ language: "csharp", outcome: "checked", files: 5, stale: 2 }],
      };
      const last_scan = { at: 1_700_000_000, level: "2", origin: "browser" };
      await route.fulfill({ response, json: { ...body, scan_report: { last_scan, level2 } } });
    });
    const row = await addScannedProject(page, harness, "usagespart", SOURCE);
    await selectAdd(page, row, "Demo");
    await page.keyboard.press("u");
    await expect(
      page.getByText(
        "Level 2 checked C# in part: 5 files, 2 changed since the run and keep level 1.",
      ),
    ).toBeVisible();
    await page.unrouteAll({ behavior: "ignoreErrors" });
  });

  for (const theme of THEMES) {
    test(`shows the description of the code before the impact (${theme})`, async ({
      page,
      harness,
    }, info) => {
      await openThemed(page, harness, theme, "usagesdoc", DOCUMENTED);
      await expect(page.getByRole("heading", { name: "Description" })).toBeVisible();
      await expect(page.getByText("Adds two whole numbers and returns the sum.")).toBeVisible();
      await shot(page, SHOTS, `usages-description-${theme}`, info);
    });
  }
});
