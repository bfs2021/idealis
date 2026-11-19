import { defineConfig } from "vitest/config";

export default defineConfig({
  test: { include: ["evals/verified-cache/cache.vitest.test.ts"] },
});
