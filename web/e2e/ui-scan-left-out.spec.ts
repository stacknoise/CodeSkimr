// A scan that leaves files out names them on the scan screen, with their code and how many more
// there were (issue #635, ADR 0085). The server is the real one; the folder holds one good C# file
// and more files in Windows-1252 than the scan names. `UPDATE_SCREENSHOTS=1` writes the screens to
// docs/screenshots/scan-left-out/.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, keepLaunchFile, SCAN_MS, shot, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/scan-left-out");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const GOOD =
  "namespace Left;\n\npublic class Counter\n{\n    public int Next(int a) { return a + 1; }\n}\n";
/** Two more than the default of `scan.max_left_out_listed` allows to name. */
const BROKEN_FILES = 22;
const LISTED = 20;
/** "Café" in Windows-1252: the byte 0xE9 is no valid UTF-8. */
const WINDOWS_1252 = Buffer.from([0x2f, 0x2f, 0x20, 0x43, 0x61, 0x66, 0xe9, 0x0a]);

for (const theme of THEMES) {
  test.describe(`a scan that left files out (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`names them with their code and counts the rest, ${theme}`, async ({
      page,
      harness,
    }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      const name = `left${theme}`;
      const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
      mkdirSync(folder);
      writeFileSync(path.join(folder, "Good.cs"), GOOD);
      for (let index = 0; index < BROKEN_FILES; index += 1) {
        const file = `Old${String(index).padStart(2, "0")}.cs`;
        writeFileSync(path.join(folder, file), WINDOWS_1252);
      }
      const { copy } = keepLaunchFile(await harness.launchFile());
      await page.goto(`file://${copy}`);
      await page.getByLabel("New project").fill(folder);
      await page.getByRole("button", { name: "Add and scan" }).click();
      await expect(page.getByText("Files left out", { exact: true })).toBeVisible({
        timeout: SCAN_MS,
      });
      await expect(page.getByText("Old00.cs")).toBeVisible();
      await expect(page.getByText("PRS-101")).toHaveCount(LISTED);
      await expect(page.getByText(/is not valid UTF-8 or UTF-16/)).toHaveCount(1);
      await page.getByText("Files left out", { exact: true }).scrollIntoViewIfNeeded();
      await shot(page, SHOTS, `scan-left-out-start-${theme}`, info);
      await page.getByText("and 2 more files").scrollIntoViewIfNeeded();
      await expect(page.getByText("and 2 more files")).toBeVisible();
      await shot(page, SHOTS, `scan-left-out-${theme}`, info);
    });
  });
}
