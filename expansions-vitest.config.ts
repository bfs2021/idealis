import { defineConfig } from "vitest/config";

export default defineConfig({
  test: { include: ["evals/cached-expansions/contract.vitest.test.ts"] },
});
