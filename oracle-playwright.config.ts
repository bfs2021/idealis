import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals/oracle-comparison",
  testMatch: "browser.spec.ts",
  workers: 1,
  outputDir: ".artifacts/oracle/browser",
  use: { browserName: "chromium" },
});
