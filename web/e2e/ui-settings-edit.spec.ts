// Changing a number of the configuration in the tab "Effective settings" (ADR 0117, mock 45
// variant A): a value is saved in the file of the app, shows its source and can go back to the
// default; a value that is security relevant keeps a grey button with the reason.
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/settings-edit/ (`ui-*.png`).
import path from "node:path";
import { addScannedProject, expect, shot, test, type Harness } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/settings-edit");
const THEMES = ["hell", "petrol", "dunkel"];
const KEY = "scan.max_workers";
const SOURCE = "namespace Shop;\npublic class Item {}\n";
const LOCKED = "level2.enabled";

for (const theme of THEMES) {
  test(`changes a number, shows its source and sets it back (${theme})`, async ({
    page,
    harness,
  }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((value) => localStorage.setItem("codeskimr.theme", value), theme);
    const added = await addScannedProject(page, harness as Harness, `settings${theme}`, SOURCE);
    await added.click({ position: { x: 4, y: 4 } });
    await page.locator('a[aria-label="Settings"]').click();
    await page.getByRole("tab", { name: "Effective settings" }).click();
    const row = page.getByRole("row").filter({ hasText: KEY });
    await expect(row).toBeVisible();
    const locked = page.getByRole("row").filter({ hasText: LOCKED });
    await expect(locked.getByRole("button", { name: "Change" })).toBeDisabled();
    await row.getByRole("button", { name: "Change" }).click();
    const dialog = page.getByRole("dialog", { name: "Change value" });
    await dialog.getByLabel("New value").fill("3");
    await expect(dialog.getByText("Applies from the next scan")).toBeVisible();
    await shot(page, SHOTS, `ui-dialog-${theme}`, info);
    await dialog.getByRole("button", { name: "Save" }).click();
    await expect(dialog).toBeHidden();
    await expect(row.getByText("Saved in the app")).toBeVisible();
    await expect(row.getByRole("cell", { name: "3", exact: true })).toBeVisible();
    await shot(page, SHOTS, `ui-table-${theme}`, info);
    await row.getByRole("button", { name: "Default" }).click();
    await expect(row.getByText("Saved in the app")).toBeHidden();
  });
}
