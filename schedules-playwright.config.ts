import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals/controlled-schedules",
  testMatch: "browser.spec.ts",
  workers: 1,
  outputDir: ".artifacts/controlled-schedules/browser",
  use: { browserName: "chromium" },
});
