import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./evals/smt-subset",
  testMatch: [
    "browser.spec.ts",
    "expansion-browser.spec.ts",
    "script-browser.spec.ts",
  ],
  workers: 1,
  use: { browserName: "chromium", headless: true },
});
