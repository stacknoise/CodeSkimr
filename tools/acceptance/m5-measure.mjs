// Measuring frame for milestone M5 (WP-67, blueprint 19): runs `tools/accuracy/measure.py` per language
// on the pinned public corpora and prints one table. Level 1 is always measured. Level 2 (ADR 0033,
// decision 14) is measured only for the languages whose SCIP index you name; it counts verified
// references only and is judged against the threshold of the ADR: precision and recall of at least
// 99 % on at least 150 sampled symbols. Level 1 and level 2 are shown in separate rows, never mixed.
//
// This script measures and prints. It does not write into the report: the numbers go into
// `docs/acceptance/m5.md` only when level 2 (#145 and the issues listed in docs/accuracy.md) is done.
// The reference tools run the build tools of the corpus (foreign code, see docs/accuracy.md): run it
// in a container or virtual machine without credentials.
//
// Usage, from the CodeSkimr root after `cargo build --release`:
//   node tools/acceptance/m5-measure.mjs [--languages rust,typescript,python,java,csharp]
//        [--samples 150] [--seed 1] [--binary target/release/codeskimr] [--out file]
//        [--level2 rust=INDEX.scip] [--level2 java=INDEX.scip:INDEXER[:WORKSPACE]] ...
// `--level2 LANG=INDEX[:INDEXER[:WORKSPACE]]` is passed as `--scip INDEX --verified-only
// [--indexer INDEXER] [--workspace WORKSPACE]` (how the index is made: docs/accuracy.md).
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../..");
const THRESHOLD = 0.99;
const MIN_SAMPLES = 150;
const DEFAULT_LANGUAGES = ["rust", "typescript", "python", "java", "csharp"];

function parse(argv) {
    const options = { languages: DEFAULT_LANGUAGES, samples: MIN_SAMPLES, seed: 1, out: null, binary: path.join(root, "target/release/codeskimr"), level2: new Map() };
    for (let i = 0; i < argv.length; i += 2) {
        const [flag, value] = [argv[i], argv[i + 1]];
        if (flag === "--languages") options.languages = value.split(",");
        else if (flag === "--samples") options.samples = Number(value);
        else if (flag === "--seed") options.seed = Number(value);
        else if (flag === "--binary") options.binary = value;
        else if (flag === "--out") options.out = value;
        else if (flag === "--level2") addLevel2(options.level2, value);
        else throw new Error(`unknown option ${flag}`);
    }
    return options;
}

function addLevel2(map, value) {
    const [language, rest] = value.split(/=(.*)/s);
    const [index, indexer, workspace] = rest.split(":");
    map.set(language, { index, indexer, workspace });
}

function measure(language, options, level2) {
    const args = ["tools/accuracy/measure.py", language, "--samples", String(options.samples), "--seed", String(options.seed), "--json"];
    args.push("--codeskimr", options.binary, "--languages", path.join(root, "languages"));
    if (level2) {
        args.push("--scip", level2.index, "--verified-only");
        if (level2.indexer) args.push("--indexer", level2.indexer);
        if (level2.workspace) args.push("--workspace", level2.workspace);
    }
    const result = spawnSync("python3", args, { cwd: root, encoding: "utf8", maxBuffer: 1 << 28 });
    if (result.status !== 0) return { error: (result.stderr || result.stdout).trim().split("\n").pop() };
    return JSON.parse(result.stdout).summary;
}

const percent = (value) => (value == null ? "–" : `${(value * 100).toFixed(1)} %`);

function row(language, level, summary) {
    if (summary.error) return `| ${language} | ${level} | not measured: ${summary.error} | | | | |`;
    const enough = summary.samples >= MIN_SAMPLES;
    const verdict =
        level === 1
            ? "shown"
            : enough && summary.precision >= THRESHOLD && summary.recall >= THRESHOLD
              ? "✅ ≥ 99 %"
              : enough
                ? "❌ < 99 %"
                : `❌ only ${summary.samples} symbols`;
    return `| ${language} | ${level} | ${summary.samples} | ${percent(summary.precision)} | ${percent(summary.recall)} | ${percent(summary.exact_rate)} | ${verdict} |`;
}

const options = parse(process.argv.slice(2));
const lines = [
    `# M5 measurement, ${new Date().toISOString().slice(0, 10)}, seed ${options.seed}, ${options.samples} symbols asked`,
    "",
    "| Language | Level | Symbols | Precision | Recall | Exact | Verdict |",
    "|---|---|---|---|---|---|---|",
];
for (const language of options.languages) {
    lines.push(row(language, 1, measure(language, options, null)));
    const level2 = options.level2.get(language);
    if (level2) lines.push(row(language, 2, measure(language, options, level2)));
}
console.log(lines.join("\n"));
if (options.out) writeFileSync(options.out, `${lines.join("\n")}\n`);
