import { readdir, readFile } from "node:fs/promises";
import { sourceFingerprint as singleModelFingerprint } from "../single-model/provenance.js";
import { digest } from "../usable/provenance.js";

export const sourceFingerprint = async () => {
  const base = await singleModelFingerprint();
  const paths = [
    "evals/plans/0020-frontier-safety.md",
    "evals/plans/0021-worker-budget.md",
    ...(await readdir("evals/frontier-safety"))
      .filter((path) => path.endsWith(".ts"))
      .map((path) => `evals/frontier-safety/${path}`),
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
