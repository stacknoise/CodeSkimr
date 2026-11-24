// Every column of the explorer and hit tables can be widened and narrowed at the handle on its right edge.
// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/columns/.
import path from "node:path";
import type { Page } from "@playwright/test";
import { addScannedProject, expect, shot, test, type Harness, openInExplorer } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/columns");
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE = [
  "namespace Demo;",
  "public class Calculator { public int Add(int a, int b) { return a + b; } }",
  "public class Checkout { public int Total(Calculator c) { return c.Add(1, 2); } }",
  "",
].join("\n");

/** The handle of the column `name` (the first one when a screen has two tables). */
const grip = (page: Page, name: string) =>
  page
    .getByRole("separator", { name: `Change width of column ${name} (drag, or arrow keys)` })
    .first();

/** Drags the handle of a column by `dx` pixels and returns how much the column grew. */
async function drag(page: Page, name: string, dx: number): Promise<number> {
  const handle = grip(page, name);
  const cell = handle.locator("xpath=..");
  const before = (await cell.boundingBox())?.width ?? 0;
  const box = await handle.boundingBox();
  const x = (box?.x ?? 0) + (box?.width ?? 0) / 2;
  const y = (box?.y ?? 0) + (box?.height ?? 0) / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + dx / 2, y, { steps: 4 });
  await page.mouse.move(x + dx, y, { steps: 4 });
  await page.mouse.up();
  return ((await cell.boundingBox())?.width ?? 0) - before;
}

for (const theme of THEMES) {
  test(`widens columns by dragging in all tables (${theme})`, async ({ page, harness }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
    const row = await addScannedProject(page, harness as Harness, `columns${theme}`, SOURCE);

    // The member table of the explorer.
    await row.click();
    await openInExplorer(page);
    await page.getByText("Demo", { exact: true }).click();
    await page.getByText("Calculator", { exact: true }).click();
    await expect(page.getByText("Members 1")).toBeVisible();
    expect(await drag(page, "Type / signature", 40)).toBeGreaterThan(30);
    await shot(page, SHOTS, `explorer-${theme}`, info);
    await page.getByRole("button", { name: /Add/ }).first().click();

    // The hit list: the place and the certainty column have a handle as well.
    await page.keyboard.press("u");
    await expect(page.getByText("Hits 1")).toBeVisible();
    for (const name of ["Kind", "Used in", "Code", "Place", "Certainty"]) {
      await expect(grip(page, name)).toBeVisible();
    }
    expect(await drag(page, "Place", 30)).toBeGreaterThan(20);
    expect(await drag(page, "Certainty", 60)).toBeGreaterThan(40);
    await shot(page, SHOTS, `usages-${theme}`, info);
  });
}
