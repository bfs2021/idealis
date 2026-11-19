import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { expect, test } from "@playwright/test";

test("pure WIT binding runs the same full reports in Chromium", async ({
  page,
}) => {
  execFileSync(
    "pnpm",
    [
      "exec",
      "esbuild",
      "evals/roracle-wit/browser-entry.ts",
      "--bundle",
      "--platform=browser",
      "--format=esm",
      "--outfile=.artifacts/typed-oracle-browser.js",
    ],
    { stdio: "pipe" },
  );
  const files = new Map([
    [
      "/entry.js",
      {
        type: "text/javascript",
        bytes: await readFile(".artifacts/typed-oracle-browser.js"),
      },
    ],
    [
      "/oracle.core.wasm",
      {
        type: "application/wasm",
        bytes: await readFile("evals/roracle-wit/generated/oracle.core.wasm"),
      },
    ],
    [
      "/roracle.wasm",
      {
        type: "application/wasm",
        bytes: await readFile(
          "crates/oracle/target/wasm32-unknown-unknown/release/roracle.wasm",
        ),
      },
    ],
  ]);
  const server = createServer((request, response) => {
    const file = files.get(request.url ?? "");
    if (file) {
      response.setHeader("content-type", file.type);
      response.end(file.bytes);
    } else response.end("<!doctype html><title>Typed oracle</title>");
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string")
    throw new Error("Missing server address");
  try {
    await page.goto(`http://127.0.0.1:${address.port}`);
    const result = await page.evaluate(async () => {
      const path = "/entry.js";
      const entry = await import(path);
      return entry.run();
    });
    expect(result).toEqual({ status: "matched", reports: 140 });
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
