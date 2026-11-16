import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";

test("expanded arithmetic and contexts run without host imports in Chromium", async ({
  page,
}) => {
  const bundle = await build({
    entryPoints: ["evals/smt-expansion/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "smtExpansion",
    write: false,
  });
  const javascript = bundle.outputFiles[0];
  if (!javascript) throw new Error("Missing bundle");
  const wasm = await readFile(
    "crates/idealis-smt/target/wasm32-unknown-unknown/release/idealis_smt.wasm",
  );
  await page.route("http://smt-expansion.test/**", async (route) => {
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
  await page.goto("http://smt-expansion.test/");
  const result = await page.evaluate(async () => {
    const api = Reflect.get(globalThis, "smtExpansion");
    if (typeof api?.run !== "function") throw new Error("Missing API");
    return api.run();
  });
  expect(result).toEqual({
    checked: 36,
    positive: "3",
    negative: "4",
    error: "error",
    afterError: "3",
    unknown: "unknown",
    restored: "9",
    isolated: false,
  });
});
