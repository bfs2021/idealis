import {
  type CacheModelInput,
  runCachedModels,
} from "../evals/verified-cache/scenarios.js";
import { createCheckCache } from "../src/check-cache.js";
import { createHttpCheckCacheStore } from "../src/check-cache-http.js";

export const run = async (
  endpoint: string,
  models: readonly CacheModelInput[],
  runId: string,
) => {
  const bytes = new Uint8Array(await (await fetch("/core.wasm")).arrayBuffer());
  return runCachedModels(
    bytes,
    models,
    createCheckCache(createHttpCheckCacheStore(endpoint)),
    runId,
  );
};
