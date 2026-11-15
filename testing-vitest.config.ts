import { defineConfig } from "vitest/config";
export default defineConfig({
  test: { include: ["evals/testing-core/runner.vitest.ts"] },
});
