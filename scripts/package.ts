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
await copyFile(
  "crates/ralloy/target/wasm32-unknown-unknown/release/ralloy.wasm",
  "assets/ralloy.wasm",
);
const alloy = await readFile("assets/ralloy.wasm");
await writeFile(
  "assets/ralloy-manifest.json",
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(alloy).digest("hex"), bytes: alloy.length }, null, 2)}\n`,
);
await copyFile(
  "crates/idealis-testing/target/wasm32-unknown-unknown/release/idealis_testing.wasm",
  "assets/idealis-testing.wasm",
);
const testing = await readFile("assets/idealis-testing.wasm");
await writeFile(
  "assets/idealis-testing-manifest.json",
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(testing).digest("hex"), bytes: testing.length }, null, 2)}\n`,
);
execFileSync("pnpm", ["pack", "--pack-destination", target], { stdio: "pipe" });
const workspace = await mkdtemp(resolve(target, "cargo-workspace-"));
for (const name of [
  "rtla",
  "rtlc",
  "rtla-compiler",
  "idealis-core",
  "idealis-smt",
  "ralloy",
  "idealis-testing",
  "oracle",
]) {
  await mkdir(resolve(workspace, name));
  for (const file of [
    "src",
    "examples",
    "tests",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "SUPPORT.md",
    "LICENSE",
    ...(["rtla", "ralloy"].includes(name) ? ["NOTICE"] : []),
    ...(name === "idealis-smt" ? ["benches"] : []),
  ])
    await cp(`crates/${name}/${file}`, resolve(workspace, name, file), {
      recursive: true,
    });
}
await writeFile(
  resolve(workspace, "Cargo.toml"),
  '[workspace]\nmembers=["rtla","rtlc","rtla-compiler","idealis-core","idealis-smt","ralloy","idealis-testing","oracle"]\nresolver="3"\n',
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
  "ralloy",
  "idealis-testing",
  "idealis-oracle",
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
await writeFile(resolve(target, "ralloy.wasm"), alloy);
await writeFile(
  resolve(target, "ralloy-manifest.json"),
  `${JSON.stringify({ version: pkg.version, abi: 1, sha256: createHash("sha256").update(alloy).digest("hex"), bytes: alloy.length }, null, 2)}\n`,
);
await copyFile(
  "assets/idealis-testing.wasm",
  resolve(target, "idealis-testing.wasm"),
);
await copyFile(
  "assets/idealis-testing-manifest.json",
  resolve(target, "idealis-testing-manifest.json"),
);
console.log(
  `Verified npm, rtla/rtla-compiler/rtlc/idealis-core/idealis-smt/ralloy/idealis-testing/idealis-oracle Cargo and Wasm artifacts: ${target}`,
);
