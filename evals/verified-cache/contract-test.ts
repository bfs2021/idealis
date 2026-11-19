import { test } from "node:test";
import { registerCheckCacheContract } from "../../tests/check-cache-contract.js";

registerCheckCacheContract((name, run) => test(name, run));
