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
  "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
  "assets/idealis-core.wasm",
);
await copyFile(
  "crates/rtla/target/wasm32-unknown-unknown/release/rtla.wasm",
  "assets/rtla.wasm",
);
const wasm = await readFile("assets/idealis-core.wasm");
const pkg = JSON.parse(await readFile("package.json", "utf8"));
await writeFile(
  "assets/manifest.json",
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(wasm).digest("hex"), bytes: wasm.length }, null, 2)}\n`,
);
const parser = await readFile("assets/rtla.wasm");
await writeFile(
  "assets/rtla-manifest.json",
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(parser).digest("hex"), bytes: parser.length }, null, 2)}\n`,
);
execFileSync("pnpm", ["pack", "--pack-destination", target], { stdio: "pipe" });
const workspace = await mkdtemp(resolve(target, "cargo-workspace-"));
for (const name of [
  "rtla",
  "rtlc",
  "rtla-compiler",
  "idealis-core",
  "idealis-smt",
]) {
  await mkdir(resolve(workspace, name));
  for (const file of [
    "src",
    "examples",
    "tests",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE",
    ...(name === "rtla" ? ["NOTICE"] : []),
    ...(name === "idealis-smt" ? ["benches"] : []),
  ])
    await cp(`crates/${name}/${file}`, resolve(workspace, name, file), {
      recursive: true,
    });
}
await writeFile(
  resolve(workspace, "Cargo.toml"),
  '[workspace]\nmembers=["rtla","rtlc","rtla-compiler","idealis-core","idealis-smt"]\nresolver="3"\n',
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
for (const name of [
  "rtla",
  "rtlc",
  "rtla-compiler",
  "idealis-core",
  "idealis-smt",
]) {
  const crate = `${name}-${pkg.version}.crate`;
  await copyFile(
    resolve(workspace, "target/package", crate),
    resolve(target, crate),
  );
}
await copyFile(
  "assets/idealis-core.wasm",
  resolve(target, "idealis-core.wasm"),
);
await copyFile("assets/manifest.json", resolve(target, "wasm-manifest.json"));
await copyFile("assets/rtla.wasm", resolve(target, "rtla.wasm"));
await copyFile(
  "assets/rtla-manifest.json",
  resolve(target, "rtla-manifest.json"),
);
const smt = await readFile(
  "crates/idealis-smt/target/wasm32-unknown-unknown/release/idealis_smt.wasm",
);
await writeFile(resolve(target, "idealis-smt.wasm"), smt);
await writeFile(
  resolve(target, "idealis-smt-manifest.json"),
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(smt).digest("hex"), bytes: smt.length }, null, 2)}\n`,
);
console.log(
  `Verified npm, rtla/rtla-compiler/rtlc/idealis-core/idealis-smt Cargo and Wasm artifacts: ${target}`,
);
