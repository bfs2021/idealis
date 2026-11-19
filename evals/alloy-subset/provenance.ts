import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
export const digest = (input: Uint8Array | string): string =>
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
    "evals/alloy-subset",
    "evals/plans/0034-alloy-subset.md",
    "alloy-playwright.config.ts",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    "scripts/package.ts",
    "scripts/package-test.ts",
    "scripts/release.ts",
    "justfile",
    "package.json",
    "pnpm-lock.yaml",
    "src/trace.ts",
  ])
    .toString()
    .split("\0")
    .filter(
      (path) =>
        (path && /\.(rs|ts|toml|lock|json|yaml|yml)$/.test(path)) ||
        path === "justfile" ||
        path === "evals/plans/0034-alloy-subset.md",
    );
  const sources = await Promise.all(
    [
      ...new Set([
        ...paths,
        "crates/oracle/target/wasm32-unknown-unknown/release/roracle.wasm",
      ]),
    ]
      .sort()
      .map(async (path) => ({ path, sha256: digest(await readFile(path)) })),
  );
  return { sources, sourceVersion: digest(JSON.stringify(sources)) };
};
