import { readFile } from "node:fs/promises";

// Saved manifests retain the repository paths used when the run was recorded.
export const readSnapshotSource = (path: string) =>
  readFile(path.startsWith("src/") ? `packages/idealis/${path}` : path);
