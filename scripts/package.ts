import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFile,
  cp,
  mkdir,
  mkdtemp,
  readFile,
  writeFile,
} from "node:fs/promises";
import { resolve } from "node:path";

const target = resolve(process.argv[2] ?? ".artifacts/packages");
await mkdir(target, { recursive: true });
await mkdir("assets", { recursive: true });
await copyFile(
  "crates/modelt-core/target/wasm32-unknown-unknown/release/modelt_core.wasm",
  "assets/modelt-core.wasm",
);
const wasm = await readFile("assets/modelt-core.wasm");
const pkg = JSON.parse(await readFile("package.json", "utf8"));
await writeFile(
  "assets/manifest.json",
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(wasm).digest("hex"), bytes: wasm.length }, null, 2)}\n`,
);
execFileSync("pnpm", ["pack", "--pack-destination", target], { stdio: "pipe" });
const workspace = await mkdtemp(resolve(target, "cargo-workspace-"));
for (const name of ["rtlc", "modelt-core"]) {
  await mkdir(resolve(workspace, name));
  for (const file of [
    "src",
    "examples",
    "tests",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE",
  ])
    await cp(`crates/${name}/${file}`, resolve(workspace, name, file), {
      recursive: true,
    });
}
await writeFile(
  resolve(workspace, "Cargo.toml"),
  '[workspace]\nmembers=["rtlc","modelt-core"]\nresolver="3"\n',
);
await copyFile("crates/rtlc/Cargo.lock", resolve(workspace, "Cargo.lock"));
execFileSync(
  "cargo",
  [
    "metadata",
    "--offline",
    "--format-version",
    "1",
    "--manifest-path",
    resolve(workspace, "Cargo.toml"),
  ],
  { stdio: "pipe" },
);
execFileSync(
  "cargo",
  [
    "package",
    "--workspace",
    "--allow-dirty",
    "--offline",
    "--locked",
    "--manifest-path",
    resolve(workspace, "Cargo.toml"),
  ],
  { stdio: "pipe" },
);
for (const name of ["rtlc", "modelt-core"]) {
  const crate = `${name}-${pkg.version}.crate`;
  await copyFile(
    resolve(workspace, "target/package", crate),
    resolve(target, crate),
  );
}
await copyFile("assets/modelt-core.wasm", resolve(target, "modelt-core.wasm"));
await copyFile("assets/manifest.json", resolve(target, "wasm-manifest.json"));
console.log(
  `Verified npm, rtlc/modelt-core Cargo and Wasm artifacts: ${target}`,
);
