import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
export const digest = (value: string | Uint8Array): string =>
  createHash("sha256").update(value).digest("hex");
export const fingerprint = async () => {
  const paths = execFileSync("git", [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-standard",
    "--",
    "crates/oracle",
    "evals/workspace-oracle",
    "crates/idealis-testing",
    "src/testing.ts",
    "src/testing-wire.ts",
    "src/contracts.ts",
    "src/conformance.ts",
    "src/shrink.ts",
    "src/trace.ts",
    "examples/durable-counter.ts",
    "examples/counter-model.ts",
    "evals/testing-core",
    "evals/smt-subset/clients.ts",
    "evals/plans/0036-testing-core.md",
    "testing-playwright.config.ts",
    "testing-vitest.config.ts",
    "package.json",
    "pnpm-lock.yaml",
    "scripts/package.ts",
    "scripts/package-test.ts",
    "scripts/release.ts",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    "justfile",
  ])
    .toString()
    .split("\0")
    .filter(
      (path) =>
        path &&
        (/\.(rs|ts|toml|lock|json|yaml|yml)$/.test(path) ||
          path === "justfile" ||
          path.endsWith("0036-testing-core.md")),
    );
  const sources = await Promise.all(
    [
      ...new Set([
        ...paths,
        "crates/oracle/target/wasm32-unknown-unknown/release/idealis_oracle.wasm",
      ]),
    ]
      .sort()
      .map(async (path) => ({ path, sha256: digest(await readFile(path)) })),
  );
  return { sources, sourceVersion: digest(JSON.stringify(sources)) };
};
