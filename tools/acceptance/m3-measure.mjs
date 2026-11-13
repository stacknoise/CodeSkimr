// Measures the M3 budgets of blueprint 16.1 through the real server (WP-49):
//   quick search per keystroke = command `search.query`   (budget search_keystroke_ms, < 10 ms)
//   switch selected type       = command `explorer.symbol` (budget type_switch_ms, < 16 ms)
// Only the server side and the HTTP round trip are measured; rendering in the browser is not.
// The server is the end-to-end harness of the app (release build, in-memory keystore); the
// project is a folder you name. Nothing is written into the folder.
//
// Usage, from the CodeSkimr root, after `npm run build` in web/:
//   node tools/acceptance/m3-measure.mjs <project-folder> [runs]
import { measureArgs, performance, startClient, stats } from "./client.mjs";

const { folder, runs } = measureArgs();
const { server, command, project } = await startClient(folder);

// Quick search: every prefix of a few names is one keystroke, as typed in the overlay.
const words = [
    "ReverseIndex",
    "Scanner",
    "Config",
    "storage",
    "resolve",
    "Symbol",
];
const keystrokes = [];
const perWord = [];
for (let round = 0; round < Math.max(1, Math.round(runs / 10)); round += 1) {
    for (const word of words) {
        for (let n = 1; n <= word.length; n += 1) {
            const r = await command("search.query", {
                project,
                query: word.slice(0, n),
            });
            keystrokes.push(r.ms);
            if (round === 0)
                perWord.push({
                    text: word.slice(0, n),
                    ms: r.ms,
                    hits: r.data.hits?.length,
                });
        }
    }
}
console.log("search.query, one call per typed prefix:", stats(keystrokes));
console.log(
    "slowest first-round prefixes:",
    perWord
        .sort((a, b) => b.ms - a.ms)
        .slice(0, 5)
        .map((p) => `${p.text}=${p.ms.toFixed(1)}ms/${p.hits}`)
        .join(", "),
);

// Switching the type: explorer.symbol for many types, first call and repeated.
const ids = new Map();
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
    for (const hit of (await command("search.query", { project, query: text }))
        .data.hits ?? [])
        if (
            hit.kind &&
            /class|struct|interface|enum|trait|record|union|type/.test(hit.kind)
        )
            ids.set(hit.id ?? hit.symbol, hit);
}
const first = [];
for (const [symbol] of [...ids].slice(0, 200)) {
    const r = await command("explorer.symbol", { project, symbol });
    if (r.ok)
        first.push({
            symbol,
            ms: r.ms,
            members: r.data.members?.length ?? r.data.rows?.length ?? 0,
        });
}
console.log("types probed:", first.length);
console.log(
    "explorer.symbol, first call per type:",
    stats(first.map((f) => f.ms)),
);
const biggest = [...first].sort((a, b) => b.members - a.members).slice(0, 5);
console.log(
    "largest types (members):",
    biggest.map((b) => b.members),
);
const repeated = [];
for (const { symbol } of biggest) {
    const series = [];
    for (let i = 0; i < runs; i += 1)
        series.push((await command("explorer.symbol", { project, symbol })).ms);
    repeated.push(series);
    console.log(`explorer.symbol, type ${symbol}:`, stats(series));
}
console.log("explorer.symbol, five largest types:", stats(repeated.flat()));
const outline = [];
for (let i = 0; i < runs; i += 1)
    outline.push((await command("explorer.outline", { project })).ms);
console.log("explorer.outline (not a budget, for reference):", stats(outline));

server.stop();
