import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { init, killThreads } from "z3-solver";
import type { Input } from "./corpus.js";
import type { Answer, Reply, Request } from "./wire.js";
import { decode } from "./wire.js";

export { maxWork, nativePath, wasmPath } from "./paths.js";

import { nativePath } from "./paths.js";
export const processClient = (
  path: string,
  args: readonly string[],
  json: boolean,
): {
  request: (input: string) => Promise<string>;
  close: () => Promise<void>;
} => {
  const child = spawn(path, [...args], { stdio: ["pipe", "pipe", "pipe"] });
  let stopped = false;
  let counter = 0;
  let stderr = "";
  const pending: {
    marker: string;
    lines: string[];
    resolve: (value: string) => void;
    reject: (error: Error) => void;
  }[] = [];
  const completion = new Promise<number | null>((resolve) =>
    child.once("close", (code) => {
      stopped = true;
      for (const request of pending.splice(0))
        request.reject(new Error(`process closed ${code}: ${stderr}`));
      resolve(code);
    }),
  );
  child.on("error", (error) => {
    stopped = true;
    for (const request of pending.splice(0)) request.reject(error);
  });
  child.stderr.on("data", (chunk) => {
    stderr += String(chunk);
  });
  const lines = createInterface({ input: child.stdout });
  lines.on("line", (line) => {
    const request = pending[0];
    if (!request) return;
    if (json) {
      pending.shift();
      request.resolve(line);
    } else if (line === request.marker || line === `"${request.marker}"`) {
      pending.shift();
      request.resolve(request.lines.join("\n"));
    } else request.lines.push(line);
  });
  return {
    request: (input) =>
      new Promise((resolve, reject) => {
        if (stopped) {
          reject(new Error("process stopped"));
          return;
        }
        const marker = `SMT-END-${++counter}`;
        pending.push({ marker, lines: [], resolve, reject });
        child.stdin.write(`${input}\n${json ? "" : `(echo "${marker}")\n`}`);
      }),
    close: async () => {
      child.stdin.end();
      const code = await completion;
      lines.close();
      assert.equal(code, 0, stderr);
    },
  };
};
export const createNative = (): {
  request: (request: Request) => Promise<Reply>;
  close: () => Promise<void>;
} => {
  const client = processClient(nativePath, [], true);
  return {
    request: async (request) =>
      decode(
        JSON.parse(
          await client.request(JSON.stringify({ version: 1, request })),
        ),
      ),
    close: client.close,
  };
};
export const quote = (name: string): string => `|${name}|`;
export const integer = (value: string): string =>
  value.startsWith("-") ? `(- ${value.slice(1)})` : value;
export const bindings = (answer: Answer): string =>
  [
    ...Object.entries(answer.values).map(
      ([name, value]) => `(assert (= ${quote(name)} ${integer(value)}))`,
    ),
    ...Object.entries(answer.booleans).map(
      ([name, value]) => `(assert (= ${quote(name)} ${String(value)}))`,
    ),
  ].join("");

export const createZ3 = async () => {
  const api = await init();
  const context = new api.Context("smt-subset-eval");
  const prepare = (input: Input) => {
    const solver = new context.Solver();
    try {
      solver.fromString(input.source);
    } catch (error) {
      solver.release();
      throw error;
    }
    return {
      check: async (): Promise<Answer> => {
        const status = await solver.check();
        if (status !== "sat") return { status, values: {}, booleans: {} };
        const model = solver.model();
        try {
          const values = Object.fromEntries(
            input.integers.map((name) => {
              const value = model.eval(context.Int.const(name), true);
              assert.ok(context.isIntVal(value));
              return [name, value.value().toString()];
            }),
          );
          const booleans = Object.fromEntries(
            input.booleans.map((name) => {
              const value = model
                .eval(context.Bool.const(name), true)
                .toString();
              assert.ok(value === "true" || value === "false");
              return [name, value === "true"];
            }),
          );
          return { status, values, booleans };
        } finally {
          model.release();
        }
      },
      close: () => solver.release(),
    };
  };
  return {
    version: api.getFullVersion(),
    prepare,
    solve: async (input: Input): Promise<Answer> => {
      const problem = prepare(input);
      try {
        return await problem.check();
      } finally {
        problem.close();
      }
    },
    validate: async (input: Input, answer: Answer): Promise<void> => {
      assert.equal(answer.status, "sat");
      const result = await (async () => {
        const solver = new context.Solver();
        try {
          solver.fromString(input.source + bindings(answer));
          return await solver.check();
        } finally {
          solver.release();
        }
      })();
      assert.equal(result, "sat", input.name);
    },
    close: () => killThreads(api.em),
  };
};

type Sexp = string | readonly Sexp[];
export const parseZ3 = (output: string, input: Input): Answer => {
  const tokens = output.match(/\(|\)|\|[^|]*\||[^\s()]+/g) ?? [];
  let position = 0;
  const read = (): Sexp => {
    const value = tokens[position++];
    assert.ok(value);
    if (value !== "(") return value;
    const items: Sexp[] = [];
    while (tokens[position] !== ")") {
      assert.ok(position < tokens.length);
      items.push(read());
    }
    position++;
    return items;
  };
  const status = read();
  assert.ok(
    status === "sat" || status === "unsat" || status === "unknown",
    output,
  );
  if (status !== "sat") return { status, values: {}, booleans: {} };
  const values: Record<string, string> = {};
  const booleans: Record<string, boolean> = {};
  if (input.integers.length + input.booleans.length) {
    const model = read();
    assert.ok(Array.isArray(model), output);
    for (const pair of model) {
      assert.ok(
        Array.isArray(pair) && pair.length === 2 && typeof pair[0] === "string",
      );
      const name = pair[0].replace(/^\||\|$/g, "");
      const value = pair[1];
      if (input.booleans.includes(name)) {
        assert.ok(value === "true" || value === "false");
        booleans[name] = value === "true";
      } else {
        assert.ok(input.integers.includes(name));
        assert.ok(
          typeof value === "string" ||
            (Array.isArray(value) &&
              value.length === 2 &&
              value[0] === "-" &&
              typeof value[1] === "string"),
        );
        values[name] =
          typeof value === "string"
            ? BigInt(value).toString()
            : (-BigInt(String(value[1]))).toString();
      }
    }
  }
  assert.equal(Object.keys(values).length, input.integers.length);
  assert.equal(Object.keys(booleans).length, input.booleans.length);
  assert.equal(position, tokens.length, output);
  return { status, values, booleans };
};
export const modelQuery = (input: Input): string =>
  input.expected === "sat" && input.integers.length + input.booleans.length
    ? `(get-value (${[...input.integers, ...input.booleans].map(quote).join(" ")}))`
    : "";
export const createNativeZ3 = (path: string) => {
  const client = processClient(path, ["-in", "-smt2"], false);
  const prepare = async (input: Input) => {
    const output = await client.request(
      `(reset)(set-option :produce-models true)${input.source}`,
    );
    assert.equal(output.trim(), "", output);
  };
  const check = async (input: Input): Promise<Answer> =>
    parseZ3(await client.request(`(check-sat)${modelQuery(input)}`), input);
  return {
    prepare,
    check,
    solve: async (input: Input): Promise<Answer> =>
      parseZ3(
        await client.request(
          `(reset)(set-option :produce-models true)${input.source}(check-sat)${modelQuery(input)}`,
        ),
        input,
      ),
    close: client.close,
  };
};
