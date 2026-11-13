// @vitest-environment node
import { ESLint } from "eslint";
import { describe, expect, it } from "vitest";

/** Lints `code` as if it were the file `file` of the UI, with the config that CI uses. */
async function rules(file: string, code: string): Promise<string[]> {
  const eslint = new ESLint({ overrideConfigFile: "eslint.literals.config.js" });
  const [result] = await eslint.lintText(code, { filePath: file });
  // A file that the config ignores comes back with one warning and no rule.
  return (result?.messages ?? [])
    .filter((message) => message.ruleId !== null || !/ignored/i.test(message.message))
    .map((message) => message.ruleId ?? "parse-error");
}

const FEATURE = "src/features/demo/view.ts";
const CONSTS = "src/features/demo/consts.ts";

describe("no literals in the frontend (R-06)", () => {
  it("refuses a number other than 0 and 1", async () => {
    expect(await rules(FEATURE, "export const limit = 4096;\n")).toContain("no-restricted-syntax");
    expect(await rules(FEATURE, "export const next = 1 + 2;\n")).toContain("no-magic-numbers");
  });

  it("allows 0 and 1", async () => {
    expect(await rules(FEATURE, "export const first = [].length + 1;\n")).toEqual([]);
  });

  it("refuses a text in the code", async () => {
    expect(await rules(FEATURE, 'export const title = "Hello world";\n')).toContain(
      "i18next/no-literal-string",
    );
  });

  it("does not check tests and generated files", async () => {
    expect(await rules("src/features/demo/view.test.ts", "export const n = 4096;\n")).toEqual([]);
    expect(await rules("src/tests/helper.ts", "export const n = 4096;\n")).toEqual([]);
    expect(await rules("src/shared/i18n/keys.ts", "export const n = 4096;\n")).toEqual([]);
  });
});

describe("consts.ts is free of the rules on the value side of a const only", () => {
  it("allows literals as the value of a const, also nested in objects and arrays", async () => {
    const code = 'export const A = 4096;\nexport const B = { c: "text", d: [10, 20] } as const;\n';
    expect(await rules(CONSTS, code)).toEqual([]);
  });

  it("allows imports and types", async () => {
    const code =
      'import { X } from "./x";\nexport type Y = "a" | "b";\nexport interface Z { n: number }\nexport const W = X;\n';
    expect(await rules(CONSTS, code)).toEqual([]);
  });

  it("refuses let and var", async () => {
    expect(await rules(CONSTS, "export let limit = 4096;\n")).toContain("no-restricted-syntax");
    expect(await rules(CONSTS, "export var limit = 4096;\n")).toContain("no-restricted-syntax");
  });

  it("refuses a literal in a function body, also one that is a const value", async () => {
    expect(await rules(CONSTS, "export function f() { return 4096; }\n")).toContain(
      "no-restricted-syntax",
    );
    expect(await rules(CONSTS, "export const f = () => 4096;\n")).toContain("no-restricted-syntax");
  });

  it("refuses statements that are not declarations", async () => {
    expect(await rules(CONSTS, 'console.log("x");\n')).toContain("no-restricted-syntax");
    expect(await rules(CONSTS, "if (true) {}\n")).toContain("no-restricted-syntax");
  });
});
