import { defineConfig } from "vitest/config";
export default defineConfig({
  test: {
    include: [
      "evals/smt-expansion/runner.vitest.ts",
      "evals/smt-scripts/runner.vitest.ts",
      "evals/smt-tuning/runner.vitest.ts",
    ],
  },
});
