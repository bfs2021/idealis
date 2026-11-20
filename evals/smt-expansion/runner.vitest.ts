import { test } from "vite-plus/test";
import { checks } from "./checks.js";

test(
  "SMT expansion agrees with Z3 and preserves independent assertion scopes",
  checks,
);
