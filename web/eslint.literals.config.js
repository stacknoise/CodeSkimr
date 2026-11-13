// No-literals rules for the frontend (blueprint R-06, section 3.2).
// Mirrors the Dylint lint `no_literals` in tools/lints: same exceptions, same intent.
// The CI job `literals` runs this file as soon as web/package.json exists.
// Needs the dev dependencies `eslint`, `typescript-eslint` and `eslint-plugin-i18next`.
import i18next from "eslint-plugin-i18next";
import tseslint from "typescript-eslint";

/** Files where literals are allowed: tests and the `consts.ts` of a feature. */
const TEST_FILES = ["**/*.test.ts", "**/*.test.tsx", "**/*.spec.ts", "**/*.spec.tsx", "**/tests/**"];
const CONST_FILES = ["**/consts.ts"];
/** Generated files are checked at their source, see tools/codegen. */
const GENERATED_FILES = ["**/*.generated.ts", "**/shared/i18n/keys.ts"];

export default tseslint.config(
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: [...TEST_FILES, ...CONST_FILES, ...GENERATED_FILES],
    languageOptions: { parser: tseslint.parser },
    plugins: { i18next },
    rules: {
      // Only 0 and 1 are allowed, exactly as in the Rust lint.
      "no-magic-numbers": ["error", { ignore: [0, 1], ignoreArrayIndexes: false, enforceConst: true }],
      // User-visible text lives in locales/, everything else in consts.ts.
      "i18next/no-literal-string": ["error", { mode: "all" }],
    },
  },
);
