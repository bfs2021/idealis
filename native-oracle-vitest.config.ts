import { defineConfig } from "vitest/config";

export default defineConfig({
  test: { include: ["evals/roracle-native/runner.vitest.ts"] },
});
