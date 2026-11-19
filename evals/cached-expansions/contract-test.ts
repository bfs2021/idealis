import { test } from "node:test";
import { registerExpansionContract } from "../../tests/expansions-contract.js";

registerExpansionContract((name, run) => test(name, run));
