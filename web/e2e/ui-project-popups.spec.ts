// The modal window "New project" and the dialog "Remove project" (mock 24, variants C and D1) in
// the real application: they open, Esc closes them first, and the removal dialog starts on
// "Cancel". `UPDATE_SCREENSHOTS=1` writes the screens (1440 x 900, all three themes) to
// docs/screenshots/new-project-popup/.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, keepLaunchFile, listedProject, shot, submitFolder, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/new-project-popup");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const THEME_KEY = "codeskimr.theme";
const SOURCE =
  "namespace Popups;\n\npublic class Counter\n{\n    public int Next(int a) { return a + 1; }\n}\n";

for (const theme of THEMES) {
  test.describe(`the windows of the project list (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`New project and Remove project open at their place and Esc closes them, ${theme}`, async ({
      page,
      harness,
    }, info) => {
      await page.addInitScript(({ key, value }) => localStorage.setItem(key, value), {
        key: THEME_KEY,
        value: theme,
      });
      const name = `popups${theme}`;
      const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
      mkdirSync(folder);
      writeFileSync(path.join(folder, "Counter.cs"), SOURCE);
      const { copy } = keepLaunchFile(await harness.launchFile());
      await page.goto(`file://${copy}`);
      await submitFolder(page, folder);
      await listedProject(page, name);
      await page.getByRole("button", { name: `Open ${name}` }).click();

      await page.getByRole("button", { name: "New", exact: true }).click();
      const window = page.getByRole("dialog", { name: "New project" });
      await expect(window).toBeVisible();
      await expect(page.getByLabel("Folder", { exact: true })).toBeFocused();
      await expect(page.getByLabel("Folder", { exact: true })).toHaveValue("");
      await page.getByLabel("Folder", { exact: true }).fill(folder);
      await expect(page.getByLabel(/^Display name/)).not.toHaveValue("");
      await shot(page, SHOTS, `new-project-${theme}`, info);
      await page.keyboard.press("Escape");
      await expect(window).toHaveCount(0);

      await page.getByRole("button", { name: `Remove project ${name}` }).click();
      const dialog = page.getByRole("alertdialog");
      await expect(dialog).toBeVisible();
      await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
      await shot(page, SHOTS, `remove-project-${theme}`, info);
      await page.keyboard.press("Escape");
      await expect(dialog).toHaveCount(0);
      await expect(page.getByRole("button", { name: `Open ${name}` })).toBeVisible();
    });
  });
}
