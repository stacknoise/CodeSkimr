// Runs after projects.spec.ts on purpose: the worker shares one server, and the project list test expects it empty.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, keepLaunchFile, test, watchPolicy } from "./fixtures";

/** A scan in a debug build loads the language packs and takes a few seconds. */
const SCAN_MS = 60_000;
const SOURCE =
  "namespace Demo;\n\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

test.describe("the explorer", () => {
  test("a scanned project opens with Enter and shows its types and members", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "calculator");
    mkdirSync(folder);
    writeFileSync(path.join(folder, "Calculator.cs"), SOURCE);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByLabel("New project").fill(folder);
    await page.getByRole("button", { name: "Add and scan" }).click();
    await expect(page.getByRole("button", { name: "Open calculator" })).toBeVisible({
      timeout: SCAN_MS,
    });

    await expect(page.getByText("current", { exact: true })).toBeVisible({ timeout: SCAN_MS });

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await page.getByText("(no layer)").click();
    await page.getByText("Calculator.Demo", { exact: true }).click();
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
