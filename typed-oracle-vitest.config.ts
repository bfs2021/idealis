import { defineConfig } from "vitest/config";
export default defineConfig({
  test: { include: ["evals/roracle-wit/runner.vitest.ts"] },
});
