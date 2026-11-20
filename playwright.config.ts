import { defineConfig } from "@playwright/test";

export default defineConfig({
  use: { browserName: "chromium" },
  reporter: "list",
  testMatch: "browser.spec.ts",
  projects: [
    {
      name: "core",
      testDir: "tests",
      testMatch: ["browser.spec.ts", "scheduling-browser.spec.ts"],
    },
    {
      name: "portable",
      testDir: "tests",
      testMatch: "portable.browser.spec.ts",
    },
    { name: "rtla", testDir: "tests", testMatch: "rtla.browser.spec.ts" },
    { name: "runtime", testDir: "tests", testMatch: "runtime.browser.spec.ts" },
    {
      name: "oracle",
      testDir: "evals",
      testMatch: [
        "oracle-comparison/browser.spec.ts",
        "workspace-oracle/browser.spec.ts",
      ],
      outputDir: ".artifacts/oracle/browser",
      workers: 1,
    },
    { name: "typed-oracle", testDir: "evals/roracle-wit" },
    {
      name: "sat",
      testDir: "evals/sat-foundation",
      outputDir: ".artifacts/sat-foundation/browser",
      workers: 1,
    },
    { name: "testing", testDir: "evals/testing-core", workers: 1 },
    {
      name: "schedules",
      testDir: "evals/controlled-schedules",
      outputDir: ".artifacts/controlled-schedules/browser",
      workers: 1,
    },
    {
      name: "fuzz",
      testDir: "evals/structured-fuzz",
      outputDir: ".artifacts/structured-fuzz/browser",
      workers: 1,
    },
    {
      name: "smt",
      testDir: "evals/smt-subset",
      testMatch: [
        "browser.spec.ts",
        "expansion-browser.spec.ts",
        "script-browser.spec.ts",
      ],
      workers: 1,
    },
    { name: "alloy", testDir: "evals/alloy-subset", workers: 1 },
    { name: "alloy-instance", testDir: "evals/alloy-instance", workers: 1 },
  ],
});
