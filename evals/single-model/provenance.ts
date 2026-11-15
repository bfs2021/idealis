import { readdir, readFile } from "node:fs/promises";
import { fingerprint } from "../parallel-safety/provenance.js";
import { digest } from "../usable/provenance.js";

export const sourceFingerprint = async () => {
  const base = await fingerprint();
  const paths = [
    "evals/plans/0019-single-model-scaling.md",
    "evals/tlc-throughput/protocol.ts",
    ...(await readdir("evals/single-model"))
      .filter((path) => path.endsWith(".ts"))
      .map((path) => `evals/single-model/${path}`),
  ];
  const sources = [
    ...base.sources,
    ...(await Promise.all(
      paths.map(async (path) => ({
        path,
        sha256: digest(await readFile(path)),
      })),
    )),
  ].sort((left, right) => left.path.localeCompare(right.path));
  return { sources, sourceVersion: digest(JSON.stringify(sources)) };
};
