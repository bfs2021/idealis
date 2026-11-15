import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { compileTlaSafety } from "../../src/tla-safety.js";

const input = JSON.parse(await readFile(process.argv[2] ?? "", "utf8"));
const program = compileTlaSafety(input);
process.stdout.write(
  execFileSync("crates/modelt-core/target/release/modelt-core", [], {
    input: `${JSON.stringify({ version: 1, request: { kind: "check-safety", program, bounds: input.bounds } })}\n`,
  }),
);
