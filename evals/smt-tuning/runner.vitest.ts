import { test } from "vitest";
import { checks } from "./checks.js";

test(
  "benchmark fixtures and immutable query snapshots retain Z3 semantics",
  checks,
);
