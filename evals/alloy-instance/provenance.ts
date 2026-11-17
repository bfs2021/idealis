import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

export const digest = (input: string | Uint8Array): string =>
  createHash("sha256").update(input).digest("hex");
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
    "crates/ralloy",
    "src/alloy.ts",
    "src/contracts.ts",
    "src/trace.ts",
    "evals/alloy-instance",
    "evals/alloy-subset/corpus.ts",
    "evals/smt-subset/clients.ts",
    "evals/plans/0035-alloy-instance.md",
    "alloy-vitest.config.ts",
    "alloy-instance-playwright.config.ts",
    "package.json",
    "pnpm-lock.yaml",
    "scripts/package.ts",
    "scripts/package-test.ts",
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
          path.endsWith("0035-alloy-instance.md")),
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
export const nativePath = "crates/ralloy/target/release/ralloy";
export const wasmPath =
  "crates/ralloy/target/wasm32-unknown-unknown/release/ralloy.wasm";
