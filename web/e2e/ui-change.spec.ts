// The screen "Change" compares two scans and shows what the difference hits (ADR 0097, mocks 29
// and erw-auswirkung), and the tab "Scan" offers "Compare with current" (mock 28 a). Keyboard paths
// of blueprint 15.4: the arrow keys of the tab list, the arrow keys of the table, the buttons.
// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/change/.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Page } from "@playwright/test";
import {
  expect,
  keepLaunchFile,
  listedProject,
  SCAN_MS,
  shot,
  showTab,
  submitFolder,
  test,
  type Harness,
} from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/change");
const HISTORY_SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/scan-history");
const THEMES = ["hell", "petrol", "dunkel"];
const BEFORE = [
  "namespace Shop.Pricing;",
  "public class Price { public decimal Round(decimal value) { return value; } }",
  "public class Legacy { public decimal Calculate() { return 1; } }",
  "namespace Shop.Checkout;",
  "public class Flow { public decimal Total(Shop.Pricing.Price price) { return price.Round(1); } }",
  "",
].join("\n");
const AFTER = [
  "namespace Shop.Pricing;",
  "public class Price { public double Round(decimal value) { return 1; } }",
  "public class Discount { public decimal Apply() { return 1; } }",
  "namespace Shop.Checkout;",
  "public class Flow { public decimal Total(Shop.Pricing.Price price) { return price.Round(1); } }",
  "",
].join("\n");

/** Adds a folder project with two scans: the first of BEFORE, the second of AFTER. */
async function addTwiceScanned(page: Page, harness: Harness, name: string): Promise<void> {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
  mkdirSync(folder);
  writeFileSync(path.join(folder, `${name}.cs`), BEFORE);
  const { copy } = keepLaunchFile(await harness.launchFile());
  await page.goto(`file://${copy}`);
  await submitFolder(page, folder);
  const row = await listedProject(page, name);
  writeFileSync(path.join(folder, `${name}.cs`), AFTER);
  await row.click({ position: { x: 4, y: 4 } });
  await showTab(page, "Scan");
  await page.getByRole("button", { name: "Scan again" }).click();
  await expect(page.getByRole("button", { name: "Compare with current" })).toBeVisible({
    timeout: SCAN_MS,
  });
}

for (const theme of THEMES) {
  test(`compares the last two scans and lists what the change hits (${theme})`, async ({
    page,
    harness,
  }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
    await addTwiceScanned(page, harness, `change${theme}`);
    await page.getByRole("region", { name: "Recent scans" }).scrollIntoViewIfNeeded();
    await shot(page, HISTORY_SHOTS, `history-${theme}`, info);
    await page.getByRole("link", { name: "Change", exact: true }).click();

    const table = page.getByRole("grid", { name: "Symbols the change touches" });
    await expect(table.getByText("Shop.Pricing.Legacy.Calculate")).toBeVisible({
      timeout: SCAN_MS,
    });
    await expect(table.getByText("Shop.Pricing.Discount.Apply")).toBeVisible();
    await shot(page, SHOTS, `impact-${theme}`, info);

    // The button puts the text of `codeskimr change --markdown` on the clipboard.
    await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.getByRole("button", { name: "Copy as Markdown" }).click();
    await expect(page.getByRole("button", { name: "Copied" })).toBeVisible();
    const copied = await page.evaluate(() => navigator.clipboard.readText());
    expect(copied).toContain("<!-- codeskimr-change -->");
    expect(copied).toContain("Shop.Checkout.Flow.Total");

    // Keyboard: the arrow keys of the table select a symbol, the arrow keys of the tab list switch
    // the view.
    await table.focus();
    // The symbol with open places is selected first; the arrow keys move to the removed ones.
    const detail = page.getByRole("region", { name: "Usages of the selected symbol" });
    await expect(detail).toContainText("Changed signature");
    await page.keyboard.press("ArrowUp");
    await expect(detail).toContainText("Removed symbol");
    await page.getByRole("tab", { name: "Impact" }).focus();
    await page.keyboard.press("ArrowRight");
    await expect(page.getByRole("tab", { name: "Comparison", selected: true })).toBeVisible();
    await expect(page.getByLabel("Base (before)")).toBeVisible();
    await shot(page, SHOTS, `compare-${theme}`, info);
  });
}

test("'Compare with current' of the tab Scan opens the view Comparison with that scan as base", async ({
  page,
  harness,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await addTwiceScanned(page, harness, "changebutton");
  const button = page.getByRole("button", { name: "Compare with current" });
  await button.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("tab", { name: "Comparison", selected: true })).toBeVisible();
  await expect(page.getByLabel("Target (after)")).toContainText("Current scan");
});

test("every scan is a row of the history, also a scan without changes, and Change says nothing changed", async ({
  page,
  harness,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await addTwiceScanned(page, harness, "changesame");
  const rows = page.getByRole("region", { name: "Recent scans" }).locator("tbody tr");
  await expect(rows).toHaveCount(2);
  // Two scans in a row that find every file as it was: each one is a row of its own, with 0 changed.
  for (const count of [3, 4]) {
    await page.getByRole("button", { name: "Scan again" }).click();
    await expect(rows).toHaveCount(count, { timeout: SCAN_MS });
  }
  await expect(rows.first().locator("td").nth(4)).toHaveText("0");
  await expect(rows.nth(1).locator("td").nth(4)).toHaveText("0");
  // The newest scan against the one before it: the same state, so the answer is "nothing changed".
  await page.getByRole("link", { name: "Change", exact: true }).click();
  await expect(page.getByText(/Nothing changed: both scans have the same state/)).toBeVisible({
    timeout: SCAN_MS,
  });
});
