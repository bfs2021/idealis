import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals",
  testMatch: [
    "oracle-comparison/browser.spec.ts",
    "workspace-oracle/browser.spec.ts",
  ],
  workers: 1,
  outputDir: ".artifacts/oracle/browser",
  use: { browserName: "chromium" },
});
