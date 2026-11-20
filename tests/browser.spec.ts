import { readFile } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { expect, test } from "@playwright/test";
import { build } from "esbuild";
import { config, names } from "../evals/experiments/quint-generated/cases.js";

let server: Server | undefined;
let origin: string;

test.beforeAll(async () => {
  const core = await build({
    entryPoints: ["packages/idealis/src/index.ts"],
    bundle: true,
    platform: "browser",
    format: "esm",
    write: false,
    metafile: true,
  });
  expect(
    Object.keys(core.metafile.inputs).every((name) =>
      name.startsWith("packages/idealis/src/"),
    ),
  ).toBe(true);
  const bundle = await build({
    entryPoints: ["tests/browser-entry.ts"],
    bundle: true,
    platform: "browser",
    format: "iife",
    globalName: "idealisBrowser",
    define: { global: "globalThis" },
    alias: { "node:zlib": "fflate" },
    external: ["bun:jsc"],
    write: false,
  });
  const output = bundle.outputFiles[0];
  if (!output) throw new Error("Browser bundle was empty");
  const z3 = dirname(createRequire(import.meta.url).resolve("z3-solver"));
  const assets = new Map([
    [
      "/core.wasm",
      {
        body: await readFile(
          "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
        ),
        type: "application/wasm",
      },
    ],
    ["/bundle.js", { body: output.contents, type: "text/javascript" }],
    [
      "/z3-built.js",
      {
        body: await readFile(join(z3, "z3-built.js")),
        type: "text/javascript",
      },
    ],
    [
      "/z3-built.wasm",
      {
        body: await readFile(join(z3, "z3-built.wasm")),
        type: "application/wasm",
      },
    ],
  ]);
  server = createServer((request, response) => {
    response.setHeader("Cross-Origin-Opener-Policy", "same-origin");
    response.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
    const asset = assets.get(request.url ?? "/");
    response.setHeader("Content-Type", asset?.type ?? "text/html");
    response.end(
      asset?.body ??
        '<!doctype html><title>idealis browser test</title><script src="/z3-built.js"></script><script src="/bundle.js"></script>',
    );
  });
  await new Promise<void>((resolve) => server?.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string")
    throw new Error("No browser test address");
  origin = `http://127.0.0.1:${address.port}`;
});

test.afterAll(async () => {
  if (server)
    await new Promise<void>((resolve, reject) =>
      server?.close((error) => (error ? reject(error) : resolve())),
    );
});

test("checks unchanged Quint output in Chromium", async ({ page }) => {
  const inputs = (
    await Promise.all(
      names.map(async (name) => {
        const source = await readFile(
          `fixtures/quint-generated/generated/${name}.tla`,
          "utf8",
        );
        return ["safety", "broken"].map((property) => ({
          source,
          config: config(property === "safety" ? "safety" : "broken"),
        }));
      }),
    )
  ).flat();
  await page.goto(origin);
  const result = await page.evaluate(async (cases) => {
    const api: unknown = Reflect.get(globalThis, "idealisBrowser");
    if (
      typeof api !== "object" ||
      api === null ||
      !("runGenerated" in api) ||
      typeof api.runGenerated !== "function"
    )
      throw new Error("Browser generated-model API missing");
    return api.runGenerated(cases);
  }, inputs);
  expect(result).toMatchObject(
    inputs.map((_, index) => ({
      status: index % 2 === 0 ? "complete" : "invariant-violation",
      reason: null,
    })),
  );
});

test("runs core, shrinking, TLA+ and Z3 Wasm in a real browser", async ({
  page,
}) => {
  await page.goto(origin);
  const result = await page.evaluate(async () => {
    const api: unknown = Reflect.get(globalThis, "idealisBrowser");
    if (
      typeof api !== "object" ||
      api === null ||
      !("run" in api) ||
      typeof api.run !== "function"
    )
      throw new Error("Browser API missing");
    return api.run();
  });
  expect(result).toEqual({
    model: "bounded",
    replay: "matched",
    property: "failed",
    shrunk: 3,
    tla: "invariant-violation",
    tlaSteps: 2,
    smt: { status: "sat", values: { value: "7" } },
    conformance: "matched",
    isolated: true,
  });
});
