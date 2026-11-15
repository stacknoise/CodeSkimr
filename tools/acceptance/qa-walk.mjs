// QA walk before 1.0: opens every screen of the rail in each theme and language over the
// end-to-end server (release build), takes a screenshot of each, and records what a reviewer
// would trip over: console errors, failed requests, horizontal overflow, text that looks like an
// untranslated key and strings that are identical in German and English.
// Needs `npm run build` in web/ and a Chromium (CODESKIMR_CHROMIUM).
//
// Usage: node tools/acceptance/qa-walk.mjs <project-folder> <out-folder>
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { startServer } from "./server.mjs";
import { openUi } from "./client.mjs";

const [folderArg, outArg] = process.argv.slice(2);
const folder = path.resolve(folderArg);
const out = path.resolve(outArg);
mkdirSync(out, { recursive: true });

const THEMES = ["hell", "petrol", "dunkel"];
const LANGUAGES = ["de", "en"];
const SUSPICIOUS = /^[a-z][a-z0-9]*(-[a-z0-9]+)+$(?<!scip-[a-z]+|haac-bridge)|undefined|NaN|\[object|\{\s*\$/;

const server = await startServer();
const report = { findings: [], texts: {} };

/** Texts of the page that a person reads, one per element without children. */
const visibleTexts = (page) =>
    page.evaluate(() =>
        [...document.querySelectorAll("body *")]
            .filter((e) => e.children.length === 0 && e.offsetParent !== null)
            .map((e) => (e.textContent ?? "").trim())
            .filter((t) => t.length > 0),
    );

/** The server keeps theme and language (ADR 0030), so they are chosen on the settings screen. */
async function choosePreferences(page, theme, language) {
    await page.getByRole("link").last().click();
    await page.waitForTimeout(500);
    await page
        .getByRole("button", { name: language.toUpperCase(), exact: true })
        .click();
    await page.waitForTimeout(300);
    const current = await page.locator("html").getAttribute("data-theme");
    if (current !== theme) {
        // The selected card shows a badge, the other two a button, in the order of THEMES.
        const others = THEMES.filter((t) => t !== current);
        await page
            .getByRole("button", { name: /^(Auswählen|Select)$/ })
            .nth(others.indexOf(theme))
            .click();
    }
    await page.waitForTimeout(500);
    const marked = await page.locator("html").getAttribute("data-theme");
    console.log(`chosen ${theme}/${language}; html data-theme=${marked}`);
}

/** Collects what goes wrong in the page: console errors, page errors, failed requests and http errors. */
function watchProblems(page, problems) {
    page.on("console", (m) => {
        if (m.type() === "error")
            problems.push(`console error: ${m.text().slice(0, 200)}`);
    });
    page.on("pageerror", (e) => problems.push(`page error: ${e.message}`));
    page.on("requestfailed", (r) =>
        problems.push(`request failed: ${r.url()}`),
    );
    page.on("response", (r) => {
        if (r.status() >= 400)
            problems.push(`http ${r.status()}: ${r.url()}`);
    });
}

async function walk(theme, language) {
    const { browser, page } = await openUi(server, folder, "codeskimr-qa-");
    const problems = [];
    watchProblems(page, problems);
    await page
        .getByText(path.basename(folder), { exact: true })
        .first()
        .waitFor({ timeout: 120000 });
    await page.waitForTimeout(1500);
    const tag = `${theme}-${language}`;
    const check = async (screen) => {
        await page.waitForTimeout(1200);
        await page.screenshot({ path: path.join(out, `${screen}-${tag}.png`) });
        const texts = await visibleTexts(page);
        report.texts[`${screen}|${language}|${theme}`] = texts;
        const overflow = await page.evaluate(
            () =>
                document.documentElement.scrollWidth -
                document.documentElement.clientWidth,
        );
        if (overflow > 0)
            problems.push(`${screen}: horizontal overflow ${overflow} px`);
        for (const t of new Set(texts))
            if (SUSPICIOUS.test(t)) problems.push(`${screen}: suspicious text "${t}"`);
    };
    await check("projects");
    await choosePreferences(page, theme, language);
    await page.getByRole("link").first().click();
    await page.getByText(path.basename(folder), { exact: true }).first().click();
    await page.keyboard.press("Enter");
    await page.waitForTimeout(1500);
    const links = await page.getByRole("link").all();
    const names = [];
    for (const l of links) names.push((await l.textContent())?.trim() ?? "");
    for (let i = 0; i < names.length; i++) {
        await page.getByRole("link").nth(i).click();
        await check(`rail${i}-${names[i].replace(/\W+/g, "_")}`);
    }
    await page.keyboard.press("Control+k");
    await page.keyboard.type("a");
    await check("quicksearch");
    for (const p of problems)
        report.findings.push({ theme, language, problem: p });
    await browser.close();
}

for (const language of LANGUAGES)
    for (const theme of THEMES) await walk(theme, language);

// Strings that are the same in both languages on the same screen: candidates for missing texts.
const same = new Set();
for (const key of Object.keys(report.texts)) {
    const [screen, language, theme] = key.split("|");
    if (language !== "de") continue;
    const en = new Set(report.texts[`${screen}|en|${theme}`] ?? []);
    for (const t of new Set(report.texts[key]))
        if (en.has(t) && /[a-zA-Z]{4,}/.test(t) && !/^[\d\W]+$/.test(t))
            same.add(`${screen}: ${t}`);
}
report.sameInBoth = [...same].sort();
writeFileSync(path.join(out, "report.json"), JSON.stringify(report, null, 2));
console.log(JSON.stringify({ findings: report.findings }, null, 2));
console.log("same in de and en:", report.sameInBoth.length);
server.stop();
