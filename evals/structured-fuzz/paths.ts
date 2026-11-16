export const nativePath =
  "evals/structured-fuzz/rust/target/release/idealis-fuzz-eval";
export const wasmPath =
  "crates/idealis-testing/target/wasm32-unknown-unknown/release/idealis_testing.wasm";
export const fuzzRoot = "evals/structured-fuzz/fuzz";
export const nightly = "nightly-2026-08-27";
export const fuzzBinary = (target: "raw" | "structured"): string =>
  `${fuzzRoot}/target/${process.arch === "arm64" && process.platform === "darwin" ? "aarch64-apple-darwin" : "x86_64-unknown-linux-gnu"}/release/${target}`;
