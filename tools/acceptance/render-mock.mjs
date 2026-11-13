// Renders one UI mock (docs/mocks/*.dc.html) to a PNG for the mock comparison of WP-54. The mocks
// are written for a design canvas whose runtime is not in the repository, so this fills the small
// template language they use (`{{name}}`, `<sc-for>`) from the data function of the mock itself.
//
// Usage: node tools/acceptance/render-mock.mjs <Name> <out.png> [width] [height]
//   Name is the file name without `.dc.html`, e.g. Graph, Scan, Usages.
import { readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(
    path.resolve(import.meta.dirname, "../../web/package.json"),
);
const { chromium } = require("@playwright/test");

const [name, out, width = "1440", height = "900"] = process.argv.slice(2);
const root = path.resolve(import.meta.dirname, "../..");
const source = readFileSync(
    path.join(root, "docs/mocks", `${name}.dc.html`),
    "utf8",
);

const script = /<script type="text\/x-dc"[^>]*>([\s\S]*?)<\/script>/.exec(
    source,
)[1];
class DCLogic {
    constructor() {
        this.props = {};
    }
}
const Component = new Function("DCLogic", `${script}; return Component;`)(
    DCLogic,
);
const values = new Component().renderVals();

const lookup = (scope, expr) => expr.split(".").reduce((o, k) => o?.[k], scope);
const fill = (text, scope) =>
    text.replace(/\{\{\s*([\w.]+)\s*\}\}/g, (_, e) =>
        String(lookup(scope, e) ?? ""),
    );
let body = /<x-dc>([\s\S]*)<\/x-dc>/.exec(source)[1];
const loop =
    /<sc-for list="\{\{(\w+)\}\}" as="(\w+)"[^>]*>([\s\S]*?)<\/sc-for>/;
for (let m = loop.exec(body); m; m = loop.exec(body)) {
    const items = values[m[1]] ?? [];
    const rows = items
        .map((item) => fill(m[3], { ...values, [m[2]]: item }))
        .join("");
    body = body.replace(m[0], rows);
}
body = fill(body, values).replace(/<\/?helmet>/g, "");

const browser = await chromium.launch({
    executablePath: process.env.CODESKIMR_CHROMIUM,
});
const page = await browser.newPage({
    viewport: { width: Number(width), height: Number(height) },
});
await page.setContent(
    `<!doctype html><html><head><meta charset="utf-8"></head><body>${body}</body></html>`,
    { waitUntil: "load" },
);
await page.screenshot({ path: out });
await browser.close();
