import { test } from "vitest";
import { registerBackendContract } from "./backend-contract.js";

registerBackendContract((name, run) => {
  test(name, run);
});
