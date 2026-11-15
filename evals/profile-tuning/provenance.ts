import { readdir, readFile } from "node:fs/promises";
import { sourceFingerprint } from "../single-model/provenance.js";
import { digest } from "../usable/provenance.js";

export const fingerprint = async () => {
  const base = await sourceFingerprint();
  const paths = [
    "evals/plans/0024-profile-tuning.md",
    ...(await readdir("evals/profile-tuning"))
      .filter((path) => path.endsWith(".ts"))
      .map((path) => `evals/profile-tuning/${path}`),
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
