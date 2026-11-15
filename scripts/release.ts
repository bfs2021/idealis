import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const pkg = JSON.parse(await readFile("package.json", "utf8"));
const tag = process.argv[2];
if (tag !== `v${pkg.version}`)
  throw new Error("Release tag and package version differ");
for (const name of [
  "idealis-core",
  "rtlc",
  "rtla",
  "rtla-compiler",
  "idealis-smt",
]) {
  const cargo = await readFile(`crates/${name}/Cargo.toml`, "utf8");
  if (!cargo.includes(`version = "${pkg.version}"`))
    throw new Error(`${name} and npm versions differ`);
}
if (process.argv[3] === "--publish") {
  const directory = resolve(".artifacts/packages");
  const tarball = resolve(directory, `mizchi-idealis-${pkg.version}.tgz`);
  const changelog = await readFile("CHANGELOG.md", "utf8");
  const section = changelog
    .split(`## ${pkg.version}\n`)[1]
    ?.split("\n## ")[0]
    ?.trim();
  if (!section) throw new Error("Missing release notes");
  const notes = resolve(directory, "release-notes.md");
  await writeFile(notes, section);
  await Promise.all(
    [
      tarball,
      resolve(directory, `idealis-core-${pkg.version}.crate`),
      resolve(directory, `rtlc-${pkg.version}.crate`),
      resolve(directory, `rtla-${pkg.version}.crate`),
      resolve(directory, `rtla-compiler-${pkg.version}.crate`),
      resolve(directory, `idealis-smt-${pkg.version}.crate`),
      resolve(directory, "idealis-smt.wasm"),
      resolve(directory, "idealis-smt-manifest.json"),
      resolve(directory, "rtla.wasm"),
      resolve(directory, "rtla-manifest.json"),
      resolve(directory, "idealis-core.wasm"),
      resolve(directory, "wasm-manifest.json"),
    ].map((path) => readFile(path)),
  );
  execFileSync(
    "npm",
    [
      "publish",
      tarball,
      "--access",
      "public",
      "--tag",
      pkg.version.includes("-") ? "next" : "latest",
    ],
    { stdio: "inherit" },
  );
  execFileSync(
    "gh",
    [
      "release",
      "create",
      tag,
      tarball,
      resolve(directory, `idealis-core-${pkg.version}.crate`),
      resolve(directory, `rtlc-${pkg.version}.crate`),
      resolve(directory, `rtla-${pkg.version}.crate`),
      resolve(directory, `rtla-compiler-${pkg.version}.crate`),
      resolve(directory, `idealis-smt-${pkg.version}.crate`),
      resolve(directory, "idealis-smt.wasm"),
      resolve(directory, "idealis-smt-manifest.json"),
      resolve(directory, "rtla.wasm"),
      resolve(directory, "rtla-manifest.json"),
      resolve(directory, "idealis-core.wasm"),
      resolve(directory, "wasm-manifest.json"),
      "--title",
      `idealis ${pkg.version}`,
      "--notes-file",
      notes,
      ...(pkg.version.includes("-") ? ["--prerelease"] : []),
    ],
    { stdio: "inherit" },
  );
} else console.log(`Release metadata verified: ${tag}`);
