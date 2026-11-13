// Shared helpers of the acceptance scripts: a REST client over the end-to-end server, timing
// statistics, and a browser that opens the UI and adds a project folder.
import { copyFileSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { performance } from "node:perf_hooks";
import { root, startServer } from "./server.mjs";

/** Starts the server, signs in and adds `folder`; returns server, command() and the project id. */
export async function startClient(folder) {
    const server = await startServer();
    const url = server.url;
    const file = await server.launchFile();
    const code = /code=([0-9a-f]+)/.exec(readFileSync(file, "utf8"))[1];
    const reply = await fetch(`${url}/api/v1/session/exchange`, {
        method: "POST",
        headers: { "content-type": "application/json", origin: url },
        body: JSON.stringify({ code }),
    });
    const bearer = (await reply.json()).token;
    const command = async (id, body) => {
        const started = performance.now();
        const answer = await fetch(`${url}/api/v1/commands/${id}`, {
            method: "POST",
            headers: {
                "content-type": "application/json",
                authorization: `Bearer ${bearer}`,
                origin: url,
            },
            body: JSON.stringify(body),
        });
        const data = await answer.json();
        return { data, ms: performance.now() - started, ok: answer.ok };
    };
    const added = await command("projects.add", { source: folder });
    console.log(
        "projects.add",
        added.ok,
        `${added.ms.toFixed(0)} ms`,
        JSON.stringify(added.data).slice(0, 300),
    );
    const list = await command("projects.list", null);
    const project = (list.data.projects ?? list.data).at(-1).id;
    return { server, command, project };
}

/** Median, p95 and max of timings in milliseconds as one line. */
export function stats(values) {
    const sorted = [...values].sort((a, b) => a - b);
    const at = (q) =>
        sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
    return `median ${at(0.5).toFixed(2)} ms, p95 ${at(0.95).toFixed(2)} ms, max ${sorted.at(-1).toFixed(2)} ms (n=${sorted.length})`;
}

/** Opens the UI of a started server in Chromium and adds the folder through "Add and scan". */
export async function openUi(server, folder, prefix, options = {}) {
    const { chromium } = createRequire(path.join(root, "web/package.json"))(
        "@playwright/test",
    );
    const copy = path.join(
        mkdtempSync(path.join(tmpdir(), prefix)),
        "launch.html",
    );
    copyFileSync(await server.launchFile(), copy);
    const browser = await chromium.launch({
        executablePath: process.env.CODESKIMR_CHROMIUM,
    });
    const context = await browser.newContext({
        viewport: { width: 1440, height: 900 },
        deviceScaleFactor: 1,
    });
    if (options.init) await context.addInitScript(...options.init);
    const page = await context.newPage();
    await page.goto(`file://${copy}`);
    await page.getByRole("textbox").first().fill(folder);
    await page
        .getByRole("button")
        .filter({ hasText: /Add and scan|Hinzufügen/ })
        .first()
        .click();
    return { browser, page };
}

export { performance };

/** Folder (default: the CodeSkimr root) and number of runs from the command line. */
export function measureArgs() {
    return {
        folder: path.resolve(process.argv[2] ?? root),
        runs: Number(process.argv[3] ?? 50),
    };
}
