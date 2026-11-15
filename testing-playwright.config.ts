import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals/testing-core",
  testMatch: "browser.spec.ts",
  workers: 1,
  use: { browserName: "chromium" },
});
