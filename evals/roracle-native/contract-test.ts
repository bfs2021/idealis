import assert from "node:assert/strict";
import { test } from "node:test";
import { decodeOracleReply } from "../workspace-oracle/protocol.js";
import { loadNativeBinding } from "./client.js";
import { registerNativeOracleContract } from "./contract.js";
import { fixtures } from "./fixtures.js";

registerNativeOracleContract(test);

test("the decoder returns the validated report from JSON.parse", (t) => {
  const fixture = fixtures[0];
  assert.ok(fixture);
  const json = loadNativeBinding().dispatchJson(JSON.stringify(fixture.input));
  const parse = t.mock.method(JSON, "parse");
  const reply = decodeOracleReply(json);
  assert.strictEqual(reply.report, parse.mock.calls[0]?.result.report);
});
