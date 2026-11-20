import { readFile, writeFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";
import { createSut } from "../../evals/experiments/controlled-schedules/clients.js";
import { wasmPath } from "../../evals/experiments/testing-core/clients.js";
import type { run } from "./browser-entry.js";

test("controls, shrinks and replays async actors without browser isolation", async ({
  page,
}, info) => {
  const bundle = await build({
    entryPoints: ["tests/controlled-schedules/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "schedules",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing schedule bundle");
  const wasm = await readFile(wasmPath);
  await page.route("http://schedules.test/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    await route.fulfill(
      path === "/bundle.js"
        ? { contentType: "application/javascript", body: javascript.text }
        : path === "/testing.wasm"
          ? { contentType: "application/wasm", body: wasm }
          : {
              contentType: "text/html",
              body: '<!doctype html><script src="/bundle.js"></script>',
            },
    );
  });
  await page.goto("http://schedules.test/");
  const result: Awaited<ReturnType<typeof run>> = await page.evaluate(
    async () => {
      const api = Reflect.get(globalThis, "schedules");
      if (typeof api?.run !== "function")
        throw new Error("Missing browser schedule API");
      return api.run();
    },
  );
  expect(result.normal.status).toBe("passed");
  expect(result.broken.status).toBe("failed");
  expect(result.broken.shrink?.status).toBe("minimal");
  expect(result.fault.status).toBe("failed");
  expect(result.fixed.status).toBe("passed");
  expect(result.emptyBudget.status).toBe("inconclusive");
  expect(result.called).toBe(false);
  expect(result.imports).toEqual([]);
  expect(result.isolated).toBe(false);
  const sut = createSut();
  try {
    const testCase = result.broken.counterexample;
    if (!testCase) throw new Error("Missing browser witness");
    expect((await sut.evaluate(3, testCase, true)).verdict.status).toBe(
      "mismatch",
    );
    expect((await sut.evaluate(3, testCase, false)).verdict.status).toBe(
      "matched",
    );
    const path = info.outputPath("records.json");
    await writeFile(path, JSON.stringify({ ...result, native: sut.exchanges }));
    await info.attach("schedule-browser-records", {
      path,
      contentType: "application/json",
    });
  } finally {
    await sut.close();
  }
});
