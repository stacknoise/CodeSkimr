// The statistics of a project (ADR 0081). Runs after ui-search.spec.ts: the worker shares one
// server and every scan adds a project.
//
// `UPDATE_SCREENSHOTS=1` writes the screen to docs/screenshots/ in all three themes.
import path from "node:path";
import type { Page } from "@playwright/test";
import {
  expect,
  keepLaunchFile,
  openScannedProject,
  SCAN_MS,
  shot,
  test,
  watchPolicy,
  type Harness,
} from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots");
const THEMES = ["hell", "petrol", "dunkel"];
const VIEWPORT = { width: 1440, height: 900 };

/** A price that is used twice, a helper nobody uses and a method with a few branches. */
const SOURCE = `namespace Shop;

public class Price
{
    public int Cents() { return 1; }
}

public class Basket
{
    public int Sum(Price first, Price second)
    {
        if (first == null) { return 0; }
        if (second == null) { return first.Cents(); }
        return first.Cents() + second.Cents();
    }

    private int Forgotten() { return 2; }
}
`;

/** Scans the sample as the project `name`, opens it and shows its statistics. */
async function openStatistics(page: Page, harness: Harness, name: string): Promise<void> {
  await openScannedProject(page, harness, name, SOURCE);
  await page.getByRole("link", { name: "Statistics" }).click();
  await expect(page.getByRole("heading", { name: "Most used" })).toBeVisible({
    timeout: SCAN_MS,
  });
}

test.describe("the statistics", () => {
  test("stay shut until a project is open", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await expect(page.getByRole("link", { name: "Statistics" })).toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });

  test("rank the symbols of a scanned project and lead into the explorer", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    await openStatistics(page, harness, "statsshop");

    await page.getByRole("tab", { name: /No usage found/ }).click();
    const table = page.getByRole("table", { name: "Ranking" });
    await expect(table.getByText("Forgotten")).toBeVisible();
    await expect(page.getByText(/does not mean dead code/)).toBeVisible();

    await page.getByRole("tab", { name: "Complexity" }).click();
    await expect(page.getByRole("img", { name: /How complex/ })).toBeVisible();
    await expect(table.getByText("Sum")).toBeVisible();
    await table.getByText("Sum").click();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("tree")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Sum" })).toBeVisible();
    expect(await violations()).toEqual([]);
  });

  for (const theme of THEMES) {
    test(`look right in the theme ${theme}`, async ({ page, harness }, info) => {
      await page.setViewportSize(VIEWPORT);
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      await openStatistics(page, harness, `statsview${theme}`);
      await shot(page, SHOTS, `stats-${theme}`, info);
      await page.getByRole("tab", { name: "Complexity" }).click();
      await expect(page.getByRole("img", { name: /How complex/ })).toBeVisible();
      await shot(page, SHOTS, `stats-complexity-${theme}`, info);
    });
  }
});
