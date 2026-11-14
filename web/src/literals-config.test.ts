// @vitest-environment node
import { resolve } from "node:path";
import { ESLint } from "eslint";
import { describe, expect, it } from "vitest";

const WEB_DIR = resolve(import.meta.dirname, "..");
const CONFIG = resolve(WEB_DIR, "eslint.literals.config.js");
const FEATURE_FILE = resolve(WEB_DIR, "src/feature/view.ts");
const CONSTS_FILE = resolve(WEB_DIR, "src/feature/consts.ts");

const eslint = new ESLint({ overrideConfigFile: CONFIG, cwd: WEB_DIR });

async function rulesFlagged(code: string, filePath: string): Promise<string[]> {
  const results = await eslint.lintText(code, { filePath });
  return results.flatMap((result) =>
    result.messages.map((message) => message.ruleId ?? "parse error"),
  );
}

describe("the literal rules of the frontend (R-06)", () => {
  it("lets 0, 1 and identifiers pass in feature code", async () => {
    expect(await rulesFlagged("export const first = items[0] + 1;", FEATURE_FILE)).toEqual([]);
  });

  it("refuses a number above 1, also as the value of a const", async () => {
    const flagged = await rulesFlagged("export const limit = 4096;", FEATURE_FILE);
    expect(flagged).toContain("no-restricted-syntax");
  });

  it("refuses a string in feature code", async () => {
    const flagged = await rulesFlagged('export const label = "Save";', FEATURE_FILE);
    expect(flagged).toContain("i18next/no-literal-string");
  });

  it("allows the value side of a const in consts.ts", async () => {
    const code = 'export const LIMIT = 4096;\nexport const LABEL = "save";';
    expect(await rulesFlagged(code, CONSTS_FILE)).toEqual([]);
  });

  it("refuses let, functions and other statements in consts.ts", async () => {
    for (const code of [
      "export let LIMIT = 4096;",
      "export function limit() { return 4096; }",
      "export const LIMIT = () => 4096;",
      "console.log(4096);",
    ]) {
      expect(await rulesFlagged(code, CONSTS_FILE), code).toContain("no-restricted-syntax");
    }
  });

  it("does not check tests and generated files", async () => {
    for (const name of [
      "src/feature/view.test.ts",
      "src/feature/view.spec.tsx",
      "src/tests/helper.ts",
      "src/shared/api/error-codes.generated.ts",
    ]) {
      expect(await eslint.isPathIgnored(resolve(WEB_DIR, name)), name).toBe(true);
    }
  });
});
