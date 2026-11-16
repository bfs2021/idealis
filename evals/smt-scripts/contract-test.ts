import { test } from "node:test";
import { checks } from "./checks.js";

test(
  "SMT scripts agree with Z3, independent snapshots and shared work budgets",
  checks,
);
