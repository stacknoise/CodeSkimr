// The tab "Conventions" on the screen "Rules" (ADR 0116, mock 44): the convention check of the
// console for the project folder, with a finding for a function that is too long.
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/conventions-check/ (`ui-*.png`).
import path from "node:path";
import { addScannedProject, expect, shot, test, type Harness } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/conventions-check");
const THEMES = ["hell", "petrol", "dunkel"];
const STATEMENTS = 50;
const SOURCE = [
  "namespace Shop;",
  "public class Long",
  "{",
  "    public int Sum(int[] values)",
  "    {",
  "        var total = 0;",
  ...Array.from({ length: STATEMENTS }, (_, index) => `        total += values[${index}];`),
  "        return total;",
  "    }",
  "}",
  "",
].join("\n");

for (const theme of THEMES) {
  test(`checks the folder and lists the function that is too long (${theme})`, async ({
    page,
    harness,
  }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((value) => localStorage.setItem("codeskimr.theme", value), theme);
    const row = await addScannedProject(page, harness as Harness, `check${theme}`, SOURCE);
    await row.click({ position: { x: 4, y: 4 } });
    await page.getByRole("link", { name: "Rules", exact: true }).click();
    await page.getByRole("tab", { name: "Conventions" }).click();
    await expect(page.getByText(/No result yet/)).toBeVisible();
    await page.getByRole("button", { name: "Check now" }).click();
    await expect(page.getByText("CHK-104", { exact: true })).toBeVisible();
    await expect(page.getByText(/lines, at most 40/)).toBeVisible();
    await expect(page.getByRole("cell", { name: "Shop.Long.Sum" }).first()).toBeVisible();
    await shot(page, SHOTS, `ui-conventions-${theme}`, info);
  });
}
