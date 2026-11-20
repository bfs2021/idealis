import { defineConfig } from "vitest/config";

const project = (name: string, ...include: string[]) => ({
  test: { name, include },
});

export default defineConfig({
  test: {
    projects: [
      project("core", "tests/**/*.vitest.test.ts"),
      project("wasm", "evals/wasm-host/runner.vitest.ts"),
      project("native-oracle", "evals/roracle-native/runner.vitest.ts"),
      project("typed-oracle", "evals/roracle-wit/runner.vitest.ts"),
      project("testing", "evals/testing-core/runner.vitest.ts"),
      project("schedules", "evals/controlled-schedules/runner.vitest.ts"),
      project("fuzz", "evals/structured-fuzz/runner.vitest.ts"),
      project("alloy", "evals/alloy-instance/runner.vitest.ts"),
      project(
        "smt",
        "evals/smt-expansion/runner.vitest.ts",
        "evals/smt-scripts/runner.vitest.ts",
        "evals/smt-tuning/runner.vitest.ts",
      ),
    ],
  },
});
