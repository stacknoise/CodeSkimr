import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, keepLaunchFile, test, watchPolicy } from "./fixtures";

/** A scan in a debug build loads the language packs and takes a few seconds. */
const SCAN_MS = 60_000;
const SOURCE =
  "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

function sampleFolder(): { folder: string; name: string } {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "calculator");
  mkdirSync(folder);
  writeFileSync(path.join(folder, "Calculator.cs"), SOURCE);
  return { folder, name: "calculator" };
}

test.describe("the project list", () => {
  test("a folder is added through the form, scanned and listed with its language", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const { folder, name } = sampleFolder();
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await expect(page.getByText("No projects yet")).toBeVisible();

    await page.getByLabel("New project").fill(folder);
    await page.getByRole("button", { name: "Add and scan" }).click();

    await expect(page.getByRole("button", { name: `Open ${name}` })).toBeVisible({
      timeout: SCAN_MS,
    });
    await expect(page.getByText(folder)).toBeVisible();
    await expect(page.getByText("C# 100 %")).toBeVisible({ timeout: SCAN_MS });
    await expect(page.getByText("current", { exact: true })).toBeVisible();
    await expect(page.getByRole("main").getByText(/^1 project · 1 type/)).toBeVisible();
    expect(await violations()).toEqual([]);

    // The list survives a reload, and the key R scans the project again.
    await page.reload();
    await expect(page.getByRole("button", { name: `Open ${name}` })).toBeVisible();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("r");
    await expect(page.getByText(/^incremental/)).toBeVisible({ timeout: SCAN_MS });
  });

  test("a relative path is refused in words", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByLabel("New project").fill("relative/folder");
    await page.getByRole("button", { name: "Add and scan" }).click();
    await expect(page.getByRole("alert")).toContainText("neither an existing folder");
  });

  test("the key N puts the cursor into the source field", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await expect(page.getByText("● Connected")).toBeVisible();
    await page.keyboard.press("n");
    await expect(page.getByLabel("New project")).toBeFocused();
  });
});
