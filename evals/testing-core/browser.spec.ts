import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";

test("public testing runtime generates, shrinks and replays real counter failures", async ({
  page,
}) => {
  const bundle = await build({
    entryPoints: ["evals/testing-core/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "testing",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing browser bundle");
  const wasm = await readFile(
    "crates/idealis-testing/target/wasm32-unknown-unknown/release/idealis_testing.wasm",
  );
  await page.route("http://testing.test/**", async (route) => {
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
  await page.goto("http://testing.test/");
  expect(
    await page.evaluate(async () => {
      const api = Reflect.get(globalThis, "testing");
      if (typeof api?.run !== "function")
        throw new Error("Missing testing API");
      return api.run();
    }),
  ).toEqual({
    normal: "passed",
    broken: "failed",
    actions: ["add", "crash", "restart"],
    shrink: "minimal",
    replay: "failed",
    fixed: "passed",
    unknown: "inconclusive",
    isolated: false,
  });
});
