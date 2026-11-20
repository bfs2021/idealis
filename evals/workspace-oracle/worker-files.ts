// The cost runner fingerprints these same files when it runs in the checkout.
export const costWorkerFiles: readonly string[] = [
  "dist/evals/workspace-oracle/cost-worker.js",
  "dist/evals/workspace-oracle/node.js",
  "dist/evals/workspace-oracle/snapshot.js",
  "dist/evals/workspace-oracle/wasm.js",
  "dist/evals/workspace-oracle/protocol.js",
  "dist/evals/roracle-native/client.js",
  "dist/evals/roracle-native/canonical.js",
  "dist/packages/idealis/src/trace.js",
  "crates/roracle/target/wasm32-unknown-unknown/release/roracle.wasm",
];
