import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./evals/roracle-wit",
  testMatch: "browser.spec.ts",
  use: { browserName: "chromium" },
  reporter: "list",
});
