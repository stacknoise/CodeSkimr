// The editor of the rules on the screen "Rules" (ADR 0112, mock 41): add a ban in the dialog with
// its preview, save it into the project file, and reach the layers in their tab.
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/rules-editor/ (`ui-*.png`).
import path from "node:path";
import type { Page } from "@playwright/test";
import { addScannedProject, expect, shot, test, type Harness } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/rules-editor");
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE = [
  "namespace Shop.Reports;",
  "public class Report { public int Run(Shop.Checkout.Cart cart) { return cart.Total() + cart.Total(); } }",
  "namespace Shop.Checkout;",
  "public class Cart { public int Total() { return 1; } }",
  "",
].join("\n");
const LAYERS = [
  "[[layers]]",
  'name = "Reports"',
  'namespaces = ["Shop.Reports"]',
  "",
  "[[layers]]",
  'name = "Checkout"',
  'namespaces = ["Shop.Checkout"]',
  "",
].join("\n");

/** Adds a scanned project with the layers, opens its screen "Rules" (in `theme` if given) and
 * opens the menu "Add rule". */
async function openRules(
  { page, harness }: { page: Page; harness: unknown },
  name: string,
  theme?: string,
) {
  if (theme !== undefined) {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((value) => localStorage.setItem("codeskimr.theme", value), theme);
  }
  const files = { ".codeskimr.toml": `# mine\n${LAYERS}` };
  const row = await addScannedProject(page, harness as Harness, name, SOURCE, files);
  await row.click({ position: { x: 4, y: 4 } });
  await page.getByRole("link", { name: "Rules", exact: true }).click();
  await page.getByRole("button", { name: "Add rule" }).click();
}

for (const theme of THEMES) {
  test(`adds a ban with a preview and shows the layers tab (${theme})`, async ({
    page,
    harness,
  }, info) => {
    await openRules({ page, harness }, `editor${theme}`, theme);
    await page.getByRole("menuitem", { name: /Ban/ }).click();
    await page.getByLabel(/This module/).fill("Shop.Reports");
    await page.getByLabel(/must not use this module/).fill("Shop.Checkout");
    await expect(page.getByRole("status")).toContainText("violation");
    await shot(page, SHOTS, `ui-ban-dialog-${theme}`, info);
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByText("Ban: Shop.Reports must not use Shop.Checkout")).toBeVisible();
    await shot(page, SHOTS, `ui-rules-${theme}`, info);
    await page.getByRole("tab", { name: "Layers" }).click();
    await expect(page.getByRole("tab", { name: "Layers", selected: true })).toBeVisible();
    await shot(page, SHOTS, `ui-layers-tab-${theme}`, info);
  });
}

test("switches the cycle rule on from the menu and the settings no longer offer the layers", async ({
  page,
  harness,
}) => {
  await openRules({ page, harness }, "editorcycles");
  await page.getByRole("menuitem", { name: /Forbid cycles/ }).click();
  await expect(page.getByText("No cycles between modules")).toBeVisible();
  await page.getByRole("link", { name: "Settings" }).click();
  await page.getByRole("tab", { name: "Analysis" }).click();
  await expect(page.getByRole("button", { name: "Edit layers" })).toHaveCount(0);
});
