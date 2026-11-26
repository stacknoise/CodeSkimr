// The screen "Rules" shows the violations of the architecture rules of the project file (ADR 0104).
// Runs after the other screens on purpose: the worker shares one server (see projects.spec.ts).
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/rules/.
import path from "node:path";
import { addScannedProject, expect, shot, test, type Harness } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/rules");
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE = [
  "namespace Shop.Reports;",
  "public class Report { public int Run(Shop.Checkout.Cart cart) { return cart.Total() + cart.Total(); } }",
  "namespace Shop.Checkout;",
  "public class Cart { public int Total() { return 1; } }",
  "",
].join("\n");
const RULES = [
  "[[rules.forbid]]",
  'from = "Shop.Reports"',
  'to = "Shop.Checkout"',
  "",
  "[[rules.public_api]]",
  'module = "Shop.Data"',
  'internal = ["Shop.Data.Internal"]',
  "",
].join("\n");

for (const theme of THEMES) {
  test(`shows a violated and a kept rule (${theme})`, async ({ page, harness }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
    const row = await addScannedProject(page, harness as Harness, `rules${theme}`, SOURCE, {
      ".codeskimr.toml": RULES,
    });
    await row.click({ position: { x: 4, y: 4 } });
    await page.getByRole("link", { name: "Rules", exact: true }).click();
    await expect(page.getByText("Ban: Shop.Reports must not use Shop.Checkout")).toBeVisible();
    await expect(page.getByText("Only through the public interface")).toBeVisible();
    await expect(page.getByText("kept", { exact: true })).toBeVisible();
    await shot(page, SHOTS, `rules-${theme}`, info);
  });
}
