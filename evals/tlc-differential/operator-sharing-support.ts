import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import type { PortableCommand } from "../../src/contracts.js";
import { PortableCoreError } from "../../src/portable-value.js";
import { createPortableWasm } from "../../src/portable-wasm.js";
import { isRecord } from "../../src/trace.js";

export const directory = "evals/fixtures/tlc-operator-sharing";
export const wasmPath =
  "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm";
export const nativePath = "crates/idealis-core/target/release/idealis-core";
export const core = await createPortableWasm(await readFile(wasmPath));

export const capture = <Value>(run: () => Value) => {
  try {
    return run();
  } catch (error) {
    assert.ok(error instanceof PortableCoreError);
    return { kind: "error" as const, code: error.code, message: error.message };
  }
};

export const native = (request: PortableCommand["request"]): unknown => {
  const result = spawnSync(nativePath, [], {
    input: `${JSON.stringify({ version: 1, request })}\n`,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
};

export const normalize = (value: unknown): unknown => {
  if (Array.isArray(value)) return value.map(normalize);
  if (!isRecord(value)) return value;
  return Object.fromEntries(
    Object.entries(value)
      .filter(
        ([key, entry]) =>
          !(key === "subscript" && entry === null) &&
          !(
            [
              "fairness",
              "initialProperties",
              "actionProperties",
              "properties",
            ].includes(key) &&
            Array.isArray(entry) &&
            !entry.length
          ),
      )
      .map(([key, entry]) => [key, normalize(entry)]),
  );
};

export interface Limits {
  readonly milliseconds: number;
  readonly bytes: number;
}
export interface BoundedRun {
  readonly stdout: string;
  readonly stderr: string;
  readonly exit: number | null;
  readonly signal: NodeJS.Signals | null;
  readonly elapsedMs: number;
  readonly peakRssBytes: number;
  readonly stopped: "time" | "memory" | undefined;
}
export const mebibytes = (count: number) => count * 1024 * 1024;

export const runBounded = (
  command: string,
  args: readonly string[],
  input: string,
  limits: Limits,
  env: NodeJS.ProcessEnv = process.env,
): Promise<BoundedRun> =>
  new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      env,
      stdio: ["pipe", "pipe", "pipe"],
    });
    const started = performance.now();
    let stdout = "";
    let stderr = "";
    let peakRssBytes = 0;
    let stopped: BoundedRun["stopped"];
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.stdin.on("error", () => undefined);
    child.stdin.end(input);
    const sample = setInterval(() => {
      if (child.pid === undefined || stopped !== undefined) return;
      const reading = spawnSync("ps", ["-o", "rss=", "-p", String(child.pid)], {
        encoding: "utf8",
      });
      const kilobytes = Number.parseInt(reading.stdout.trim(), 10);
      if (Number.isFinite(kilobytes))
        peakRssBytes = Math.max(peakRssBytes, kilobytes * 1024);
      if (peakRssBytes > limits.bytes) stopped = "memory";
      else if (performance.now() - started > limits.milliseconds)
        stopped = "time";
      if (stopped !== undefined) child.kill("SIGKILL");
    }, 10);
    child.once("error", (error) => {
      clearInterval(sample);
      reject(error);
    });
    child.once("close", (exit, signal) => {
      clearInterval(sample);
      resolve({
        stdout,
        stderr,
        exit,
        signal,
        elapsedMs: performance.now() - started,
        peakRssBytes,
        stopped,
      });
    });
  });

export type Outcome =
  | { readonly kind: "safety"; readonly result: Record<string, unknown> }
  | { readonly kind: "error"; readonly code: string; readonly message: string };

export interface Measured {
  readonly outcome: unknown;
  readonly elapsedMs: number;
  readonly run: BoundedRun;
}

export const runNative = async (
  request: PortableCommand["request"],
  limits: Limits,
): Promise<Measured> => {
  const run = await runBounded(
    nativePath,
    [],
    `${JSON.stringify({ version: 1, request })}\n`,
    limits,
  );
  assertAlive(run);
  return { outcome: JSON.parse(run.stdout), elapsedMs: run.elapsedMs, run };
};

const wasmModule = new URL("../../src/portable-wasm.js", import.meta.url).href;
const frontendModule = new URL("../../src/tla-safety.js", import.meta.url).href;
const childSource = `
import { readFileSync } from "node:fs";
const { createPortableWasm } = await import(process.env.IDEALIS_WASM_MODULE);
const wasm = await createPortableWasm(readFileSync(process.env.IDEALIS_WASM_BINARY));
const job = JSON.parse(readFileSync(0, "utf8"));
const started = performance.now();
const failure = (error) => ({
  kind: "error",
  code: String(error?.code ?? ""),
  message: error?.code ? String(error.message) : String(error?.name ?? "") + ": " + String(error?.message ?? error),
});
let outcome;
try {
  let request = job.request;
  if (job.frontend === "ts") {
    const { compileTlaSafety } = await import(process.env.IDEALIS_FRONTEND_MODULE);
    const program = compileTlaSafety(job.input);
    request = { kind: "check-safety", program, bounds: job.bounds };
  }
  outcome = wasm.request({ version: 1, request });
} catch (error) {
  outcome = failure(error);
}
process.stdout.write(JSON.stringify({ elapsedMs: performance.now() - started, outcome }));
`;

const runChild = async (job: unknown, limits: Limits): Promise<Measured> => {
  const run = await runBounded(
    process.execPath,
    ["--input-type=module", "-e", childSource],
    JSON.stringify(job),
    limits,
    {
      ...process.env,
      IDEALIS_WASM_MODULE: wasmModule,
      IDEALIS_WASM_BINARY: wasmPath,
      IDEALIS_FRONTEND_MODULE: frontendModule,
    },
  );
  assertAlive(run);
  const reply = JSON.parse(run.stdout);
  assert.ok(isRecord(reply));
  assert.equal(typeof reply.elapsedMs, "number");
  return { outcome: reply.outcome, elapsedMs: reply.elapsedMs as number, run };
};

export const runWasm = (
  request: PortableCommand["request"],
  limits: Limits,
): Promise<Measured> => runChild({ frontend: "wasm", request }, limits);

export const runTypeScript = (
  input: {
    readonly source: string;
    readonly config: string;
    readonly irVersion: 1 | 2;
  },
  bounds: unknown,
  limits: Limits,
): Promise<Measured> => runChild({ frontend: "ts", input, bounds }, limits);

export function assertAlive(run: BoundedRun): void {
  assert.equal(
    run.stopped,
    undefined,
    `stopped on ${run.stopped}: peak ${Math.round(run.peakRssBytes / 1048576)} MiB after ${Math.round(run.elapsedMs)} ms`,
  );
  assert.equal(
    run.signal,
    null,
    `terminated by ${run.signal} at peak ${Math.round(run.peakRssBytes / 1048576)} MiB: ${run.stderr.slice(0, 200)}`,
  );
  assert.equal(run.exit, 0, run.stderr.slice(0, 300));
}
