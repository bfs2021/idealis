# idealis-core

`check-safety-coverage`は逐次のSafetyProgram検査へcoverageを付ける。応答は`{kind:"safety-coverage",report:{result,coverage}}`。通常の`check-safety`の結果形は変えない。branchのattempts/生成・新規state/workとunattributedWorkは[rtlcの契約](../rtlc/README.md#アクションのcoverage)に従う。

[対応範囲・非対応のチェックリスト](SUPPORT.md)を参照。

Rust native / Wasm core for finite state models, conformance checking, trace replay, and failure-preserving sequence shrinking.

The `compile-tla` and `check-tla` requests accept an optional `modules` object mapping module names to source strings. The compiler resolves the explicit bundle without filesystem or network access. Omitting `modules` preserves the existing single source request. See [module resolution and configuration overrides](../rtla-compiler/README.md#モジュールと設定置換) for the supported fragment.

`SafetyProgram::check()` is re-exported from the independent `rtlc` crate. It evaluates a versioned transition program and performs BFS, invariant/deadlock checking, temporal/fairness checking, and counterexample reconstruction inside Rust. It generates successors during exploration rather than accepting a precomputed graph. The IR supports a finite TLA+ fragment including integers, booleans, sets, functions, records and sequences; it is not a complete TLA+ interpreter. `compile_tla(source, config)` connects the `rtla` parser and `rtla-compiler` to produce that IR in native or Wasm. The checker itself lives in `rtlc` and has no parser or solver dependency. The existing TypeScript `compileTlaSafety` frontend remains a comparison path. See [the supported fragment and limits](https://github.com/mizchi/idealis/blob/main/docs/rust-safety.md).

`Session::new()` and `Session::replay()` return effects. The caller performs model evaluation or I/O and returns responses with the pending effect ID. `FiniteModel` loads a versioned, observation-bearing transition table without a model-language runtime. `FiniteModel::check_history()` validates saved observations against that table. No threads, clocks or host imports are required by the Wasm library.

The checker implementation and its Criterion benchmarks live in [rtlc](../rtlc). `idealis-core` retains the finite-model, conformance, history, host, shrink, JSONL and Wasm adapters. `Value` and `CoreError` are shared by re-export, with the existing wire format preserved. `Value` implements `Drop` to release arbitrarily deep values with bounded stack, so its fields cannot be moved out by pattern; match by reference and take fields with `std::mem::take` or `std::mem::replace`.

See the [repository](https://github.com/mizchi/idealis) for TypeScript bindings, the shared value format, examples, and evaluations. The `idealis-core` binary accepts the same versioned JSON commands over stdin/stdout, one JSON object per line.

The release is experimental. A matched finite history covers the supplied actions and observations; it is not a refinement proof or a liveness guarantee.

`compile_tla_with_options(source, config, CompileOptions { irVersion: IrVersion::V1 })` explicitly selects safety IR v1. `compile_tla` and omitted `irVersion` in native/Wasm `compile-tla` / `check-tla` requests retain v1 behavior. A request with `irVersion: 2` emits safety IR v2 with shared expression definitions and call-by-name arguments. Calls and argument demands consume work separately; values are not memoized. Action operators remain inlined and recursion is refused. When the pre-checks shared with v1 (linking and validation) pass, a v2 request that needs module linking or configuration overrides (a user-module `EXTENDS` or `INSTANCE`, an operator substitution, a constant table or a definition override) is refused with a `protocol` error whose message is `Unsupported TLA input: safety IR v2 does not support module linking or configuration operator substitution`; v1 and omitted `irVersion` accept the same input as before. Extra modules in `modules` that the root does not need do not change this: when the input needs no linking or configuration override and bundle validation passes, v2 lowers the same root as a single-module v2 request, and when that lowering succeeds the program is the same as without them. The request envelope and Wasm ABI stay at version 1; existing frontend and checker limits remain in force.

明示v2のaction/temporal helperはcall siteごとの非共有anonymous frameでcallerの遅延引数を保持する。binder・EXCEPT oldと要求時のprimeを分離し、入口と引数要求へ各1 workを課金する。LET本体の自由な`@`はtyped protocol拒否とし、action template共有・再帰・全TLA+互換を保証しない。
