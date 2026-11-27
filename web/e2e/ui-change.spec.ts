// The screen "Change" compares two scans and shows what the difference hits (ADR 0097, mocks 29
// and erw-auswirkung), and the tab "Scan" offers "Compare with current" (mock 28 a). Keyboard paths
// of blueprint 15.4: the arrow keys of the tab list, the arrow keys of the table, the buttons.
// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/change/.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Page, TestInfo } from "@playwright/test";
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
async function addTwiceScanned(
  page: Page,
  harness: Harness,
  name: string,
  sources: { before: string; after: string } = { before: BEFORE, after: AFTER },
): Promise<void> {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
  mkdirSync(folder);
  writeFileSync(path.join(folder, `${name}.cs`), sources.before);
  const { copy } = keepLaunchFile(await harness.launchFile());
  await page.goto(`file://${copy}`);
  await submitFolder(page, folder);
  const row = await listedProject(page, name);
  writeFileSync(path.join(folder, `${name}.cs`), sources.after);
  await row.click({ position: { x: 4, y: 4 } });
  await showTab(page, "Scan");
  await page.getByRole("button", { name: "Scan again" }).click();
  await expect(page.getByRole("button", { name: "Compare with current" })).toBeVisible({
    timeout: SCAN_MS,
  });
}

/** Runs a test once in each theme, on a window of 1440 x 900. */
function inEachTheme(
  title: string,
  body: (page: Page, harness: Harness, info: TestInfo, theme: string) => Promise<void>,
): void {
  for (const theme of THEMES) {
    test(`${title} (${theme})`, async ({ page, harness }, info) => {
      await page.setViewportSize({ width: 1440, height: 900 });
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      await body(page, harness, info, theme);
    });
  }
}

inEachTheme(
  "compares the last two scans and lists what the change hits",
  async (page, harness, info, theme) => {
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
  },
);

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

const MANY = 120;
const LONG_PARAMETERS = Array.from({ length: 40 }, (_, index) => `decimal parameter${index}`).join(
  ", ",
);

/** A file with MANY classes; the second state changes the signature of every method, and the
 * method of the first class gets a signature that is far wider than the list. */
function manyClasses(changed: boolean): string {
  const lines = ["namespace Shop.Many;"];
  for (let index = 0; index < MANY; index += 1) {
    const parameters = changed && index === 0 ? LONG_PARAMETERS : "decimal value";
    const result = changed ? "double" : "decimal";
    const visibility = index % 2 === 0 ? "public" : "internal";
    lines.push(
      `${visibility} class Item${String(index).padStart(3, "0")} { public ${result} Run(${parameters}) { return 1; } }`,
    );
  }
  return `${lines.join("\n")}\n`;
}

/** The numbers of the count over the list: "12 of 120" (or "12 of 120 listed (130 in all)"). */
async function counts(page: Page): Promise<{ shown: number; listed: number }> {
  const text =
    (await page
      .getByRole("status")
      .filter({ hasText: / of \d+/ })
      .first()
      .textContent()) ?? "";
  const [shown, listed] = (text.match(/\d+/g) ?? []).map(Number);
  return { shown: shown ?? 0, listed: listed ?? 0 };
}

/** The box with the scrollbar of a list: the parent of the table. */
function scroller(page: Page, role: "grid" | "table", name: string) {
  return page.getByRole(role, { name }).locator("xpath=..");
}

inEachTheme(
  "a long list scrolls in its own box, the page keeps its height, and the filter narrows it",
  async (page, harness, info, theme) => {
    await addTwiceScanned(page, harness, `changemany${theme}`, {
      before: manyClasses(false),
      after: manyClasses(true),
    });
    await page.getByRole("link", { name: "Change", exact: true }).click();
    const grid = page.getByRole("grid", { name: "Symbols the change touches" });
    await expect(grid).toBeVisible({ timeout: SCAN_MS });

    // The page does not grow: the window holds, the list has its own scrollbar.
    const heights = await page.evaluate(() => ({
      page: document.documentElement.scrollHeight,
      window: window.innerHeight,
      main: document.querySelector("main")?.scrollHeight ?? 0,
      mainBox: document.querySelector("main")?.clientHeight ?? 0,
    }));
    expect(heights.page).toBeLessThanOrEqual(heights.window);
    expect(heights.main).toBeLessThanOrEqual(heights.mainBox);
    const list = scroller(page, "grid", "Symbols the change touches");
    const box = await list.evaluate((el) => ({ all: el.scrollHeight, shown: el.clientHeight }));
    expect(box.all).toBeGreaterThan(box.shown);
    const bottom = (await page.viewportSize())?.height ?? 0;
    expect((await list.boundingBox())?.y ?? bottom).toBeLessThan(bottom);
    expect(
      ((await list.boundingBox())?.y ?? 0) + ((await list.boundingBox())?.height ?? 0),
    ).toBeLessThanOrEqual(bottom);
    await list.evaluate((el) => el.scrollTo(0, el.scrollHeight));
    await expect(grid.locator("tbody tr").last()).toBeInViewport();
    await shot(page, SHOTS, `impact-many-${theme}`, info);

    // The filter row narrows the list; the keyboard reaches it and the count says how many pass.
    const search = page.getByRole("searchbox", { name: "Search by qualified name" });
    await search.focus();
    await page.keyboard.type("Item00");
    const all = (await counts(page)).listed;
    expect(all).toBeGreaterThanOrEqual(MANY);
    await expect.poll(async () => (await counts(page)).shown).toBeLessThan(all);
    const named = (await counts(page)).shown;
    expect(named).toBeGreaterThan(0);
    await page.getByRole("button", { name: "Only public" }).focus();
    await page.keyboard.press("Enter");
    await expect.poll(async () => (await counts(page)).shown).toBeLessThan(named);
    // A press on a head sorts what the filter left; the selected row stays selected.
    const selectedBefore = await grid.locator("tr[aria-selected='true']").textContent();
    await grid.getByRole("button", { name: "Symbol" }).focus();
    await page.keyboard.press("Enter");
    await expect(grid.getByRole("columnheader", { name: /Symbol/ })).toHaveAttribute(
      "aria-sort",
      "ascending",
    );
    expect(await grid.locator("tr[aria-selected='true']").textContent()).toBe(selectedBefore);
    await search.fill("no such symbol");
    await expect(page.getByText("No symbol matches the filter.")).toBeVisible();
    await page.getByRole("button", { name: "Reset filter" }).click();
    await expect.poll(async () => (await counts(page)).shown).toBe(all);

    // The view Comparison: the same filter row, its list scrolls too, and no cell reaches over the
    // edge of the box, not even the very long signature.
    await page.getByRole("tab", { name: "Comparison" }).click();
    const compare = scroller(page, "table", "The symbols of the difference");
    await expect(compare).toBeVisible();
    const sizes = await compare.evaluate((el) => ({
      all: el.scrollHeight,
      shown: el.clientHeight,
    }));
    expect(sizes.all).toBeGreaterThan(sizes.shown);
    const edge = (await compare.boundingBox())?.x ?? 0;
    const right = edge + ((await compare.boundingBox())?.width ?? 0);
    const cells = page.getByRole("table", { name: "The symbols of the difference" }).locator("td");
    for (const cell of await cells.all()) {
      const rect = await cell.boundingBox();
      expect((rect?.x ?? 0) + (rect?.width ?? 0)).toBeLessThanOrEqual(right + 1);
    }
    expect(await compare.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
    await expect(page.getByRole("button", { name: "Only with open places" })).toBeDisabled();
    await search.fill("Item00");
    await expect.poll(async () => (await counts(page)).shown).toBe(named);
    const compareTable = page.getByRole("table", { name: "The symbols of the difference" });
    await compareTable.getByRole("button", { name: "File" }).click();
    await expect(compareTable.getByRole("columnheader", { name: /File/ })).toHaveAttribute(
      "aria-sort",
      "ascending",
    );
    await shot(page, SHOTS, `compare-many-${theme}`, info);
  },
);
