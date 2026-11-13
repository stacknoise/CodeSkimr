// Takes screenshots of the three screens of milestone M4 (dependencies, scan and performance,
// usages) from the real UI over the end-to-end server (release build), for the mock comparison
// of WP-54. Needs `npm run build` in web/ and a Chromium (CODESKIMR_CHROMIUM).
//
// Usage: node tools/acceptance/m4-screens.mjs <project-folder> <out-folder> [theme] [language]
import { mkdirSync } from "node:fs";
import path from "node:path";
import { startServer } from "./server.mjs";
import { openUi } from "./client.mjs";

const [folderArg, outArg, theme = "hell", language = "de"] =
    process.argv.slice(2);
const folder = path.resolve(folderArg);
const out = path.resolve(outArg);
mkdirSync(out, { recursive: true });

const server = await startServer();
const init = [
    ([t, l]) => {
        localStorage.setItem("codeskimr.theme", t);
        localStorage.setItem("codeskimr.language", l);
    },
    [theme, language],
];
const { browser, page } = await openUi(server, folder, "codeskimr-m4-", {
    init,
});
await page.waitForTimeout(500);
const snap = (name) =>
    page.screenshot({ path: path.join(out, `${name}-${theme}.png`) });
const open = page.getByRole("button", { name: /CodeSkimr/ }).first();
await open.waitFor({ timeout: 120000 });
await open.click();
await page.waitForTimeout(800);
await snap("explorer");

const rail = await page.getByRole("link").allTextContents();
console.log("rail links:", rail);
await page.getByRole("link", { name: /Dependencies|Abhängigkeiten/ }).click();
await page.waitForTimeout(1500);
await snap("graph");
await page.getByRole("button", { name: "domain" }).click();
await page.waitForTimeout(800);
await snap("graph-selected");
await page
    .getByRole("link", {
        name: /Scans and performance|Scans und Leistung|Scan/,
    })
    .first()
    .click();
await page.waitForTimeout(1500);
await snap("scan");
// Usages: search a type, open it in the explorer, select a member and press U.
await page.keyboard.press("Control+k");
await page.keyboard.type(process.env.M4_TYPE ?? "ReverseIndex");
await page.waitForTimeout(800);
await page.keyboard.press("Enter");
await page.waitForTimeout(1500);
await snap("explorer-type");
await page.locator("table tbody tr").nth(2).click();
await page.keyboard.press("u");
await page.waitForTimeout(1500);
await snap("usages");
await page.evaluate(() =>
    window.scrollTo(0, document.documentElement.scrollHeight),
);
await page.waitForTimeout(500);
await snap("usages-scrolled");
console.log(
    "document scroll height",
    await page.evaluate(() => document.documentElement.scrollHeight),
);
server.stop();
await browser.close();
