import { readFile } from "node:fs/promises";
import { join } from "node:path";

export const readOracleWasm = async (
  root: string,
): Promise<Buffer<ArrayBuffer>> => {
  const directory = join(
    root,
    "crates/oracle/target/wasm32-unknown-unknown/release",
  );
  try {
    return await readFile(join(directory, "roracle.wasm"));
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT")
      return readFile(join(directory, "idealis_oracle.wasm"));
    throw error;
  }
};
