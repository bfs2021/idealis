import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  testMatch: "rtla.browser.spec.ts",
  use: { browserName: "chromium" },
  reporter: "list",
});
