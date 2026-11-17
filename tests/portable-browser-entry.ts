import { counterModel } from "../examples/counter-model.js";
import { createCounter, isCounterAction } from "../examples/durable-counter.js";
import { createRtlcBackend, runBackend } from "../src/backend.js";
import { checkConformance } from "../src/conformance.js";
import type {
  SafetyBounds,
  SafetyProgram,
  SafetyResult,
  TlaSourceInput,
} from "../src/contracts.js";
import { compileFiniteModel } from "../src/finite.js";
import { createSimulationHost } from "../src/host.js";
import {
  checkFiniteHistories,
  checkPortableConformance,
  checkPortableTrace,
  checkSafety,
  coreDeletionCandidates,
  createHistoryRecorder,
  createPortableWasm,
  fromPortableValue,
  loadFiniteModel,
  toPortableTrace,
  toPortableValue,
  verifyHostTape,
} from "../src/portable.js";
import { PortableCoreError } from "../src/portable-value.js";
import { checkTlaSource, compileTlaSource } from "../src/tla-source.js";

export const run = async (): Promise<Record<string, unknown>> => {
  const core = await createPortableWasm(
    new Uint8Array(await (await fetch("/core.wasm")).arrayBuffer()),
  );
  const source = {
    source:
      "---- MODULE BrowserSource ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nSafe == x < 2\n====",
    config: "INIT Init\nNEXT Next\nINVARIANT Safe",
  };
  const sourceProgram = compileTlaSource(core, source);
  const sourceResult = checkTlaSource(core, source, {
    maxDepth: 10,
    maxStates: 100,
    maxWork: 10000,
  });
  if (
    sourceProgram.name !== "BrowserSource" ||
    sourceResult.status !== "invariant-violation" ||
    sourceResult.violation?.trace.length !== 3
  )
    throw new Error("Rust source frontend differs in browser");
  const finiteInputs: {
    name: string;
    source: string;
    config: string;
    expected: string;
  }[] = await (await fetch("/finite.json")).json();
  for (const input of finiteInputs) {
    const program = compileTlaSource(core, input);
    const bounds = {
      maxDepth: 100,
      maxStates: 1000,
      maxWork: 1_000_000,
      maxTemporalNodes: 10000,
    };
    const observe = (run: () => SafetyResult): string => {
      try {
        return run().status;
      } catch (error) {
        if (!(error instanceof PortableCoreError)) throw error;
        return "model-error";
      }
    };
    if (
      observe(() => checkTlaSource(core, input, bounds)) !== input.expected ||
      observe(() => checkSafety(core, program, bounds)) !== input.expected
    )
      throw new Error(`Finite TLC support differs in browser: ${input.name}`);
  }
  const moduleInputs: (TlaSourceInput & {
    name: string;
    expected: string;
    states?: number;
    action?: string;
  })[] = await (await fetch("/modules.json")).json();
  for (const input of moduleInputs) {
    const bounds = { maxDepth: 100, maxStates: 1000, maxWork: 1_000_000 };
    const observe = (run: () => unknown): unknown => {
      try {
        return run();
      } catch (error) {
        if (!(error instanceof PortableCoreError)) throw error;
        return null;
      }
    };
    const program = observe(() => compileTlaSource(core, input));
    if (input.expected === "unsupported") {
      if (program !== null)
        throw new Error(`Browser accepted unsupported module ${input.name}`);
      continue;
    }
    const result = observe(() => checkTlaSource(core, input, bounds));
    if (input.expected === "model-error") {
      if (program === null || result !== null)
        throw new Error(`Browser module error differs ${input.name}`);
      continue;
    }
    if (program === null || result === null)
      throw new Error(`Browser module failed ${input.name}`);
    const checked = checkTlaSource(core, input, bounds);
    if (input.action && checked.violation?.trace[1]?.action !== input.action)
      throw new Error(`Browser module action differs ${input.name}`);
    if (
      checked.status !== input.expected ||
      (input.states !== undefined && checked.states !== input.states)
    )
      throw new Error(`Browser module result differs ${input.name}`);
  }
  const actions = [
    { type: "add", id: "a", amount: 1 },
    { type: "crash" },
    { type: "restart" },
  ] as const;
  const program: SafetyProgram = await (await fetch("/safety.json")).json();
  const safety = checkSafety(core, program, {
    maxDepth: 4,
    maxStates: 5,
    maxWork: 100,
  });
  const backendResult = await runBackend(
    createRtlcBackend(core, { version: "0.1.0-alpha.0" }),
    {
      format: "modelt.backend-request",
      version: 1,
      scope: {
        modelId: "browser-safety-v1",
        propertyId: "Safe-v1",
        semantics: "tlc-1.7.4-int32",
        assumptions: [],
      },
      query: {
        kind: "explicit",
        bounds: { maxDepth: 4, maxStates: 5, maxWork: 100 },
      },
      input: program,
    },
  );
  if (
    backendResult.outcome.kind !== "counterexample" ||
    backendResult.outcome.witness.kind !== "behavior"
  )
    throw new Error("Browser backend contract differs");
  if (
    safety.status !== "invariant-violation" ||
    safety.violation?.trace.length !== 4
  )
    throw new Error("Browser safety backend differs");
  const complex: {
    program: SafetyProgram;
    bounds?: SafetyBounds;
    expected: SafetyResult["status"];
  }[] = await (await fetch("/complex.json")).json();
  for (const input of complex) {
    const result = checkSafety(
      core,
      input.program,
      input.bounds ?? {
        maxDepth: 100,
        maxStates: 10000,
        maxWork: 2000000,
        maxTemporalNodes: 50000,
      },
    );
    if (result.status !== input.expected)
      throw new Error(`Browser complex backend: ${JSON.stringify(result)}`);
  }
  const reference = await checkConformance(
    counterModel,
    createCounter(),
    actions,
  );
  const normal = await checkPortableConformance(
    core,
    counterModel,
    createCounter(),
    actions,
  );
  const broken = await checkPortableConformance(
    core,
    counterModel,
    createCounter({ persist: false }),
    actions,
  );
  const replay = await checkPortableTrace(
    core,
    toPortableTrace(reference.trace),
    createCounter({ persist: false }),
    (value) => {
      const action = fromPortableValue(value);
      if (!isCounterAction(action)) throw new Error("Invalid portable action");
      return action;
    },
  );
  const value = toPortableValue({
    integer: 9007199254740993123456789n,
    set: new Set([2, 1, 2]),
    map: new Map([["😀", new Uint8Array([255])]]),
  });
  const result = core.request({
    version: 1,
    request: { kind: "canonicalize", value },
  });
  if (
    result.kind !== "value" ||
    JSON.stringify(result.value) !== JSON.stringify(value)
  )
    throw new Error("Browser value semantics differ");
  const finite = compileFiniteModel(counterModel, {
    maxDepth: 20,
    maxStates: 1000,
  });
  if (
    !finite.model ||
    checkFiniteHistories(core, finite.model, [
      toPortableTrace(reference.trace),
    ])[0]?.status !== "matched"
  )
    throw new Error("Browser finite artifact differs");
  if (!coreDeletionCandidates(core, 3).some((range) => range.count === 1))
    throw new Error("Missing browser shrink candidates");
  const loaded = loadFiniteModel(core, finite.model);
  const history = createHistoryRecorder(
    counterModel.name,
    toPortableValue(reference.trace.initial),
  );
  for (const [index, step] of reference.trace.steps.entries()) {
    history.invoke(String(index), toPortableValue(step.action));
    history.returned(String(index), toPortableValue(step.observation));
  }
  if (
    loaded.checkHistory(history.snapshot(), 100).status !== "linearizable" ||
    loaded.checkHistories([toPortableTrace(reference.trace)])[0]?.status !==
      "matched"
  )
    throw new Error("Browser loaded history differs");
  loaded.release();
  const host = createSimulationHost({ seed: 3 });
  host.write("value", 1);
  host.persist();
  host.write("value", 2);
  host.crash();
  host.restart();
  if (host.read("value") !== 1)
    throw new Error("Browser storage recovery differs");
  if (
    JSON.stringify(verifyHostTape(core, host.trace())) !==
    JSON.stringify(
      toPortableValue({
        clock: 0,
        running: true,
        storage: { value: 1 },
        pending: [],
      }),
    )
  )
    throw new Error("Rust browser host replay differs");
  return {
    source: sourceResult.status,
    backend: backendResult.outcome.kind,
    normal: normal.status,
    broken: broken.status,
    replay: replay.status,
    steps: broken.checkedSteps,
    isolated: globalThis.crossOriginIsolated,
  };
};
