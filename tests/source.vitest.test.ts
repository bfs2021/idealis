import { test } from "vitest";
import { registerSourceContract } from "./source-contract.js";

registerSourceContract((name, run) => {
  test(name, run);
});
