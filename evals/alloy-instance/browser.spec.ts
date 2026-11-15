import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";

test("public Alloy adapter checks an observed instance in Chromium", async ({
  page,
}) => {
  const bundle = await build({
    entryPoints: ["evals/alloy-instance/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "ralloy",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing browser bundle");
  const wasm = await readFile(
    "crates/ralloy/target/wasm32-unknown-unknown/release/ralloy.wasm",
  );
  await page.route("http://alloy.test/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    await route.fulfill(
      path === "/bundle.js"
        ? { contentType: "application/javascript", body: javascript.text }
        : path === "/ralloy.wasm"
          ? { contentType: "application/wasm", body: wasm }
          : {
              contentType: "text/html",
              body: '<!doctype html><script src="/bundle.js"></script>',
            },
    );
  });
  await page.goto("http://alloy.test/");
  expect(
    await page.evaluate(async () => {
      const api = Reflect.get(globalThis, "ralloy");
      if (typeof api?.run !== "function") throw new Error("Missing ralloy API");
      return api.run();
    }),
  ).toEqual({
    status: "counterexample",
    check: "mismatch",
    dag: "matched",
    unknown: "unknown",
    invalid: "invalid",
    isolated: false,
  });
});
