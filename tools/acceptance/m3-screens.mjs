// Takes screenshots of the four screens of milestone M3 (projects, explorer, quick search,
// settings) in all three themes from the real UI over the end-to-end server (release build), for
// the mock comparison of WP-49. Needs `npm run build` in web/ and a Chromium (CODESKIMR_CHROMIUM).
//
// Usage: node tools/acceptance/m3-screens.mjs <project-folder> <out-folder> [language]
// Files: <screen>-<theme>.png with screen = projects, explorer, search, settings.
import { copyFileSync, mkdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { root, startServer } from "./server.mjs";

const require = createRequire(path.join(root, "web/package.json"));
const { chromium } = require("@playwright/test");

const [folderArg, outArg, language = "de"] = process.argv.slice(2);
const folder = path.resolve(folderArg);
const out = path.resolve(outArg);
mkdirSync(out, { recursive: true });
const type = process.env.M3_TYPE ?? "ReverseIndex";

const server = await startServer();
const copy = path.join(
    mkdtempSync(path.join(tmpdir(), "codeskimr-m3-")),
    "launch.html",
);
copyFileSync(await server.launchFile(), copy);

const browser = await chromium.launch({
    executablePath: process.env.CODESKIMR_CHROMIUM,
});
const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
});
const page = await context.newPage();
await page.goto(`file://${copy}`);
await page.getByRole("textbox").first().fill(folder);
await page
    .getByRole("button")
    .filter({ hasText: /Add and scan|Hinzufügen/ })
    .first()
    .click();
const open = page.getByRole("button", { name: /CodeSkimr|codeskimr/ }).first();
await open.waitFor({ timeout: 180000 });
await page.waitForTimeout(3000);

const settle = (ms = 700) => page.waitForTimeout(ms);
for (const theme of ["hell", "petrol", "dunkel"]) {
    await page.evaluate(
        ([t, l]) => {
            localStorage.setItem("codeskimr.theme", t);
            localStorage.setItem("codeskimr.language", l);
        },
        [theme, language],
    );
    await page.goto(
        page
            .url()
            .split("#")[0]
            .replace(/\/[^/]*$/, "/"),
    );
    await page.waitForLoadState("networkidle");
    await settle();
    const snap = (name) =>
        page.screenshot({ path: path.join(out, `${name}-${theme}.png`) });
    await snap("projects");
    await page
        .getByRole("button", { name: /CodeSkimr|codeskimr/ })
        .first()
        .click();
    await settle();
    // Quick search: the overlay with a query and a chosen hit, then open the type in the explorer.
    await page.keyboard.press("Control+k");
    await page.keyboard.type(type.slice(0, 4));
    await settle(900);
    await snap("search");
    await page.keyboard.type(type.slice(4));
    await settle(600);
    await page.keyboard.press("Enter");
    await settle(1200);
    await page
        .locator("table tbody tr")
        .nth(1)
        .click()
        .catch(() => {});
    await settle();
    await snap("explorer");
    await page.getByRole("link", { name: /Settings|Einstellungen/ }).click();
    await settle();
    await snap("settings");
    await page
        .getByRole("link", { name: /Projects|Projekte/ })
        .first()
        .click();
}
server.stop();
await browser.close();
