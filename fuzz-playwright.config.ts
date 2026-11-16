import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals/structured-fuzz",
  testMatch: "browser.spec.ts",
  workers: 1,
  outputDir: ".artifacts/structured-fuzz/browser",
  use: { browserName: "chromium" },
});
