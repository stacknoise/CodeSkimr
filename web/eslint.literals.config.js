// No-literals rules for the frontend (blueprint R-06, section 3.2).
// Mirrors the Dylint lint `no_literals` in tools/lints: same exceptions, same intent.
// The CI job `literals` runs this file as soon as web/package.json exists.
// Needs the dev dependencies `eslint`, `typescript-eslint` and `eslint-plugin-i18next`.
import i18next from "eslint-plugin-i18next";
import tseslint from "typescript-eslint";

/** Files where literals are allowed: tests and the `consts.ts` of a feature. */
const TEST_FILES = [
  "**/*.test.ts",
  "**/*.test.tsx",
  "**/*.spec.ts",
  "**/*.spec.tsx",
  "**/tests/**",
];
const CONST_FILES = ["**/consts.ts"];
/** Generated files are checked at their source, see tools/codegen. */
const GENERATED_FILES = ["**/*.generated.ts", "**/shared/i18n/keys.ts", "**/theme/tokens.ts"];

/**
 * `consts.ts` may hold literals only as the value of a `const`. Everything else is refused there:
 * other statements, `let` and `var`, and function bodies (where a literal would hide).
 */
const CONST_FILE_SHAPE = [
  {
    selector:
      "Program > *:not(ImportDeclaration, ExportNamedDeclaration, VariableDeclaration, TSTypeAliasDeclaration, TSInterfaceDeclaration)",
    message: "consts.ts holds only constants, types and imports.",
  },
  {
    selector: "VariableDeclaration[kind!='const']",
    message: "consts.ts holds only const declarations.",
  },
  {
    selector: ":function",
    message: "consts.ts holds values, not functions; a literal in a function is not a constant.",
  },
];

export default tseslint.config(
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: [...TEST_FILES, ...CONST_FILES, ...GENERATED_FILES],
    languageOptions: { parser: tseslint.parser },
    plugins: { i18next },
    rules: {
      // Only 0 and 1 are allowed, exactly as in the Rust lint.
      "no-magic-numbers": [
        "error",
        { ignore: [0, 1], ignoreArrayIndexes: false, enforceConst: true },
      ],
      // `no-magic-numbers` lets `const limit = 4096` pass (a number "defined as a constant"), the
      // Rust lint does not: a number above 1 belongs in consts.ts, whatever it is assigned to.
      "no-restricted-syntax": [
        "error",
        {
          selector: "Literal[value>1]",
          message: "A number other than 0 and 1 belongs in consts.ts.",
        },
      ],
      // User-visible text lives in locales/, everything else in consts.ts.
      "i18next/no-literal-string": ["error", { mode: "all" }],
    },
  },
  {
    files: CONST_FILES.map((file) => file.replace("**/", "src/**/")),
    languageOptions: { parser: tseslint.parser },
    rules: { "no-restricted-syntax": ["error", ...CONST_FILE_SHAPE] },
  },
);
