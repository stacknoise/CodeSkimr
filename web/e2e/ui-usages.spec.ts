// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
import { addScannedProject, expect, test, watchPolicy } from "./fixtures";

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
    await row.click({ position: { x: 4, y: 4 } });
    await page.keyboard.press("Enter");
    await page.getByText("(no layer)").click();
    await page.getByText("usagesdemo.Demo", { exact: true }).click();
    await page.getByText("Calculator", { exact: true }).click();
    await page.getByRole("button", { name: /Add/ }).first().click();

    await page.keyboard.press("u");
    const graph = page.getByRole("region", { name: "Usage graph" });
    await expect(graph.getByText("Checkout.Total")).toBeVisible();
    await expect(page.getByText("Hits 1")).toBeVisible();
    await expect(page.getByText("unchecked", { exact: true })).toBeVisible();
    await expect(page.getByText(/No indexer is set up/)).toBeVisible();

    // Enter shows the caller in the explorer.
    await page.keyboard.press("F8");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("tree")).toBeVisible();
    expect(await violations()).toEqual([]);
  });
});
