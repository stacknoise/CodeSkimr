// Runs after projects.spec.ts on purpose: the worker shares one server, and the project list test expects it empty.
import { addScannedProject, expect, keepLaunchFile, test, watchPolicy } from "./fixtures";

const SOURCE =
  "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

test.describe("the explorer", () => {
  test("a scanned project opens with Enter and shows its types and members", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const row = await addScannedProject(page, harness, "explorerdemo", SOURCE);
    await row.click({ position: { x: 4, y: 4 } });
    await page.keyboard.press("Enter");
    await expect(page.getByRole("tree")).toBeVisible();
    await page.getByText("(no layer)").click();
    await page.getByText("explorerdemo.Demo", { exact: true }).click();
    await page.getByText("Calculator", { exact: true }).click();

    await expect(page.getByRole("heading", { name: "Calculator" })).toBeVisible();
    await expect(page.getByRole("row", { name: /Add.*public.*1/ })).toBeVisible();
    await expect(page.getByText("Members 1")).toBeVisible();
    expect(await violations()).toEqual([]);
  });

  test("without an open project the explorer asks for one", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByRole("link", { name: "Explorer" }).click();
    await expect(page.getByText("No project open")).toBeVisible();
  });
});
