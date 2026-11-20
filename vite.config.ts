import { defineConfig } from "vite-plus";

const ignorePatterns = [
  "**/target/**",
  ".artifacts/**",
  ".mutator/**",
  "assets/**",
  "dist/**",
  "test-results/**",
  "playwright-report/**",
  "evals/runs/**",
  "evals/fixtures/**",
  "evals/structured-fuzz/fuzz/artifacts/**",
  "evals/structured-fuzz/fuzz/corpus/**",
];

export default defineConfig({
  fmt: {
    printWidth: 80,
    sortPackageJson: false,
    sortImports: { newlinesBetween: false },
    embeddedLanguageFormatting: "off",
    ignorePatterns: [
      ...ignorePatterns,
      "**/pnpm-lock.yaml",
      "**/Cargo.toml",
      "crates/rtla/tests/fixtures/LICENSE.md",
      "evals/roracle-wit/generated/*.js",
    ],
  },
  lint: {
    categories: { correctness: "error" },
    rules: { "no-unused-vars": "warn", "unicorn/no-new-array": "warn" },
    ignorePatterns: [...ignorePatterns, "evals/roracle-wit/generated/**"],
  },
});
