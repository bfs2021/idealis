import { test } from "vite-plus/test";
import { checks } from "./checks.js";

test(
  "benchmark fixtures and immutable query snapshots retain Z3 semantics",
  checks,
);
