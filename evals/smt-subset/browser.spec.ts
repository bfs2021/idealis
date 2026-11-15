import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";

test("pure Rust SMT subset runs in Chromium without cross-origin isolation", async ({
  page,
}) => {
  const bundle = await build({
    entryPoints: ["evals/smt-subset/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "smtSubset",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing browser bundle");
  const wasm = await readFile(
    "crates/rsmt/target/wasm32-unknown-unknown/release/rsmt.wasm",
  );
  await page.route("http://smt.test/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/bundle.js")
      await route.fulfill({
        contentType: "application/javascript",
        body: javascript.text,
      });
    else if (path === "/subset.wasm")
      await route.fulfill({ contentType: "application/wasm", body: wasm });
    else
      await route.fulfill({
        contentType: "text/html",
        body: '<!doctype html><script src="/bundle.js"></script>',
      });
  });
  await page.goto("http://smt.test/");
  const result = await page.evaluate(async () => {
    const api = Reflect.get(globalThis, "smtSubset");
    if (typeof api?.run !== "function") throw new Error("Missing API");
    return api.run();
  });
  expect(result).toEqual({
    sat: "sat",
    amount: "7",
    unsat: "unsat",
    unknown: "unknown",
    error: "error",
    repeated: "7",
    isolated: false,
  });
});
