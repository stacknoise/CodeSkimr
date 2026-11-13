// Measures the M4 budgets of blueprint 16.1 through the real server (WP-54):
//   graph layout  = command `graph.layers`  (budget graph_layout_ms, < 50 ms)
//   usages query  = command `usages.query`  at depth 2 (budget usages_query_ms, < 20 ms)
// The server is the end-to-end harness of the app (release build, in-memory keystore); the
// project is a folder you name. Nothing is written into the folder.
//
// Usage, from the CodeSkimr root, after `npm run build` in web/:
//   node tools/acceptance/m4-measure.mjs <project-folder> [runs]
import { readFileSync } from "node:fs";
import path from "node:path";
import { root, startServer } from "./server.mjs";
import { performance } from "node:perf_hooks";

const folder = path.resolve(process.argv[2] ?? root);
const runs = Number(process.argv[3] ?? 50);

const server = await startServer();
const url = server.url;

async function token() {
    const file = await server.launchFile();
    const code = /code=([0-9a-f]+)/.exec(readFileSync(file, "utf8"))[1];
    const reply = await fetch(`${url}/api/v1/session/exchange`, {
        method: "POST",
        headers: { "content-type": "application/json", origin: url },
        body: JSON.stringify({ code }),
    });
    return (await reply.json()).token;
}

const bearer = await token();
async function command(id, body) {
    const started = performance.now();
    const reply = await fetch(`${url}/api/v1/commands/${id}`, {
        method: "POST",
        headers: {
            "content-type": "application/json",
            authorization: `Bearer ${bearer}`,
            origin: url,
        },
        body: JSON.stringify(body),
    });
    const data = await reply.json();
    return { data, ms: performance.now() - started, ok: reply.ok };
}

function stats(values) {
    const sorted = [...values].sort((a, b) => a - b);
    const at = (q) =>
        sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
    return `median ${at(0.5).toFixed(2)} ms, p95 ${at(0.95).toFixed(2)} ms, max ${sorted.at(-1).toFixed(2)} ms (n=${sorted.length})`;
}

const added = await command("projects.add", { source: folder });
console.log(
    "projects.add",
    added.ok,
    `${added.ms.toFixed(0)} ms`,
    JSON.stringify(added.data).slice(0, 300),
);
const list = await command("projects.list", null);
const project = (list.data.projects ?? list.data).at(-1).id;

// Graph layout.
const first = await command("graph.layers", { project });
console.log(
    "graph.layers boxes:",
    first.data.boxes?.length,
    "links:",
    first.data.links?.length,
);
const layout = [];
for (let i = 0; i < runs; i += 1)
    layout.push((await command("graph.layers", { project })).ms);
console.log("graph.layers (request round trip):", stats(layout));

// Usages: look for symbols with many users among the hits of a few searches.
const candidates = new Map();
for (const text of [
    "new",
    "error",
    "path",
    "id",
    "name",
    "config",
    "result",
    "value",
    "string",
    "symbol",
]) {
    const hits =
        (await command("search.query", { project, query: text })).data.hits ??
        [];
    for (const hit of hits) candidates.set(hit.id ?? hit.symbol, hit);
}
const measured = [];
for (const [symbol] of [...candidates].slice(0, 400)) {
    const r = await command("usages.query", {
        project,
        symbol,
        way: "both",
        depth: 2,
        include_tests: true,
    });
    if (r.ok)
        measured.push({
            symbol,
            ms: r.ms,
            hits: r.data.incoming?.total ?? 0,
            summary: r.data.summary,
        });
}
measured.sort((a, b) => b.hits - a.hits);
const buckets = [10, 100, 1000, Infinity].map((max, i, all) => {
    const low = i === 0 ? 0 : all[i - 1];
    const group = measured.filter((m) => m.hits >= low && m.hits < max);
    return `${low}-${max === Infinity ? "+" : max - 1} incoming: ${group.length} symbols, ${group.length ? stats(group.map((m) => m.ms)) : "-"}`;
});
console.log(
    "first call per symbol by number of incoming usages:\n  " +
        buckets.join("\n  "),
);
console.log(
    "symbols probed:",
    measured.length,
    "largest incoming:",
    measured.slice(0, 3).map((m) => m.hits),
);
const usages = [];
for (const { symbol } of measured.slice(0, 5)) {
    const series = [];
    for (let i = 0; i < runs; i += 1) {
        series.push(
            (
                await command("usages.query", {
                    project,
                    symbol,
                    way: "both",
                    depth: 2,
                    include_tests: true,
                })
            ).ms,
        );
    }
    usages.push(series);
    console.log(`usages.query depth 2, symbol ${symbol}:`, stats(series));
}
console.log("usages.query all five symbols:", stats(usages.flat()));
// Where does the time of the largest symbol go? Vary depth and direction.
const hub = measured[0].symbol;
for (const [way, depth] of [
    ["both", 1],
    ["incoming", 1],
    ["incoming", 2],
    ["outgoing", 2],
]) {
    const series = [];
    for (let i = 0; i < runs; i += 1)
        series.push(
            (
                await command("usages.query", {
                    project,
                    symbol: hub,
                    way,
                    depth,
                    include_tests: true,
                })
            ).ms,
        );
    console.log(`largest symbol, way ${way}, depth ${depth}:`, stats(series));
}

server.stop();
