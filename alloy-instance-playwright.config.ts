import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "evals/alloy-instance",
  testMatch: "browser.spec.ts",
  workers: 1,
  use: { browserName: "chromium" },
});
