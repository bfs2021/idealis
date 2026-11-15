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
    "evals/smt-subset",
    "evals/smt-subset-rust",
    "evals/plans/0030-smt-subset.md",
    "evals/plans/0031-smt-reuse.md",
    "evals/harness/ttl.ts",
    "src/conformance.ts",
    "src/contracts.ts",
    "src/trace.ts",
    "examples/counter-model.ts",
    "examples/durable-counter.ts",
    "smt-playwright.config.ts",
    "justfile",
    "package.json",
    "pnpm-lock.yaml",
    ".github/workflows/ci.yml",
  ])
    .toString()
    .split("\0")
    .filter(
      (path) =>
        path &&
        (path.endsWith(".ts") ||
          path.endsWith(".rs") ||
          path.endsWith(".toml") ||
          path.endsWith(".lock") ||
          path.endsWith(".yaml") ||
          path.endsWith(".yml") ||
          path.endsWith(".json") ||
          path.endsWith("0030-smt-subset.md") ||
          path.endsWith("0031-smt-reuse.md") ||
          path === "justfile"),
    );
  const sources = await Promise.all(
    [...new Set(paths)]
      .sort()
      .map(async (path) => ({ path, sha256: digest(await readFile(path)) })),
  );
  return { sources, sourceVersion: digest(JSON.stringify(sources)) };
};
