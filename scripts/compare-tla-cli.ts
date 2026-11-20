import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cp, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const [java, jar] = process.argv.slice(2);
if (!java || !jar)
  throw new Error("Usage: compare-tla-cli <java> <tla2tools.jar v1.7.4>");
const fixtures = resolve("crates/idealis-cli/tests/specs");
const native = resolve(
  "crates/idealis-cli/target/release",
  process.platform === "win32" ? "idealis.exe" : "idealis",
);
const npm = resolve("packages/idealis/bin/idealis.js");
const javaBin =
  java.includes("/") || java.includes("\\") ? resolve(java) : java;
const classpath = resolve(jar);
const results = [
  ["tlc", ["-workers", "1", "Counter"], 0, "No error has been found"],
  ["tlc", ["-config", "Bad", "Counter.tla"], 12, "Broken"],
  ["tlc", ["Stop"], 11, "Deadlock reached"],
  ["tlc", ["-deadlock", "Stop"], 0, "No error has been found"],
  ["tla", ["Stop"], 0, "Semantic processing of module Stop"],
  ["tla", ["Unknown.tla"], 0, "Semantic errors:"],
  ["tla", ["-s", "Unknown.tla"], 0, "Parsing file"],
  ["tla", ["SyntaxError.tla"], 255, ""],
  ["tlc", ["-xx", "Counter"], 1, ""],
  ["tlc", ["MissingModel"], 255, ""],
] as const;
const root = await mkdtemp(join(tmpdir(), "idealis-tla-cli-"));
try {
  for (const [index, [command, args, code, diagnostic]] of results.entries()) {
    const routes = [
      [
        javaBin,
        "-cp",
        classpath,
        command === "tlc" ? "tlc2.TLC" : "tla2sany.SANY",
      ],
      [native, command],
      [process.execPath, npm, command],
    ] as const;
    for (const [route, [bin, ...prefix]] of routes.entries()) {
      const cwd = join(root, String(index), String(route));
      await cp(fixtures, cwd, { recursive: true });
      await writeFile(
        join(cwd, "SyntaxError.tla"),
        "---- MODULE SyntaxError ----\nBad == (\n====\n",
      );
      const output = spawnSync(bin, [...prefix, ...args], {
        cwd,
        encoding: "utf8",
        timeout: 30_000,
      });
      if (output.error) throw output.error;
      const message = `${bin} ${command} ${args.join(" ")}\n${output.stdout}\n${output.stderr}`;
      assert.equal(output.status, code, message);
      assert.ok(output.stdout.includes(diagnostic), message);
      if (command === "tla" && args[0] === "-s")
        assert.ok(!output.stdout.includes("Semantic errors:"), message);
    }
  }
  console.log(
    `${results.length} CLI cases matched across TLC/SANY, Rust and npm.`,
  );
} finally {
  await rm(root, { recursive: true, force: true });
}
