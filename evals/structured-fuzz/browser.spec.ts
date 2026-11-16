import { readFile, writeFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";
import type { run } from "./browser-entry.js";
import { type AuditRecord, createAudit } from "./clients.js";
import { regression } from "./corpus.js";
import { wasmPath } from "./paths.js";

test("replays rejected requests, saved cases and feedback phases in Chromium", async ({
  page,
}, info) => {
  const audit = await createAudit();
  const records: AuditRecord[] = [];
  try {
    for (const input of regression) records.push(await audit.check(input));
  } finally {
    await audit.close();
  }
  const bundle = await build({
    entryPoints: ["evals/structured-fuzz/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "fuzzReplay",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing fuzz replay bundle");
  const wasm = await readFile(wasmPath);
  await page.route("http://fuzz.test/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    await route.fulfill(
      path === "/bundle.js"
        ? { contentType: "application/javascript", body: javascript.text }
        : path === "/testing.wasm"
          ? { contentType: "application/wasm", body: wasm }
          : path === "/cases.json"
            ? { contentType: "application/json", body: JSON.stringify(records) }
            : {
                contentType: "text/html",
                body: '<!doctype html><script src="/bundle.js"></script>',
              },
    );
  });
  await page.goto("http://fuzz.test/");
  const result: Awaited<ReturnType<typeof run>> = await page.evaluate(
    async () => {
      const api = Reflect.get(globalThis, "fuzzReplay");
      if (typeof api?.run !== "function")
        throw new Error("Missing browser replay API");
      return api.run();
    },
  );
  expect(result.records).toBe(regression.length);
  expect(result.requests).toBeGreaterThan(10);
  expect(result.imports).toEqual([]);
  expect(result.isolated).toBe(false);
  const path = info.outputPath("records.json");
  await writeFile(path, JSON.stringify({ result, native: records }));
  await info.attach("fuzz-browser-records", {
    path,
    contentType: "application/json",
  });
});
