import { execFileSync } from "node:child_process";

for (const manifest of [
  "crates/oracle/Cargo.toml",
  "crates/idealis-core/Cargo.toml",
  "crates/rtla/Cargo.toml",
  "crates/idealis-smt/Cargo.toml",
  "crates/ralloy/Cargo.toml",
  "crates/idealis-testing/Cargo.toml",
  "evals/rust-probe/Cargo.toml",
  "evals/sat-foundation/rust/Cargo.toml",
  "evals/oracle-comparison/rust/Cargo.toml",
]) {
  const args = ["build", "--release", "--locked", "--manifest-path", manifest];
  execFileSync("cargo", args, { stdio: "inherit" });
  execFileSync(
    "cargo",
    [...args, "--lib", "--target", "wasm32-unknown-unknown"],
    { stdio: "inherit" },
  );
}
for (const manifest of [
  "evals/controlled-schedules/rust/Cargo.toml",
  "evals/structured-fuzz/rust/Cargo.toml",
])
  execFileSync(
    "cargo",
    ["build", "--release", "--locked", "--manifest-path", manifest],
    { stdio: "inherit" },
  );
