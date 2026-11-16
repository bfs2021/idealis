import { defineConfig } from "vitest/config";
export default defineConfig({
  test: { include: ["evals/controlled-schedules/runner.vitest.ts"] },
});
