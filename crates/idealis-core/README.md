# idealis-core

[対応範囲・非対応のチェックリスト](SUPPORT.md)を参照。

Rust native / Wasm core for finite state models, conformance checking, trace replay, and failure-preserving sequence shrinking.

The `compile-tla` and `check-tla` requests accept an optional `modules` object mapping module names to source strings. The compiler resolves the explicit bundle without filesystem or network access. Omitting `modules` preserves the existing single source request. See [module resolution and configuration overrides](../rtla-compiler/README.md#モジュールと設定置換) for the supported fragment.

`SafetyProgram::check()` is re-exported from the independent `rtlc` crate. It evaluates a versioned transition program and performs BFS, invariant/deadlock checking, temporal/fairness checking, and counterexample reconstruction inside Rust. It generates successors during exploration rather than accepting a precomputed graph. The IR supports a finite TLA+ fragment including integers, booleans, sets, functions, records and sequences; it is not a complete TLA+ interpreter. `compile_tla(source, config)` connects the `rtla` parser and `rtla-compiler` to produce that IR in native or Wasm. The checker itself lives in `rtlc` and has no parser or solver dependency. The existing TypeScript `compileTlaSafety` frontend remains a comparison path. See [the supported fragment and limits](https://github.com/mizchi/idealis/blob/main/docs/rust-safety.md).

`Session::new()` and `Session::replay()` return effects. The caller performs model evaluation or I/O and returns responses with the pending effect ID. `FiniteModel` loads a versioned, observation-bearing transition table without a model-language runtime. `FiniteModel::check_history()` validates saved observations against that table. No threads, clocks or host imports are required by the Wasm library.

The checker implementation and its Criterion benchmarks live in [rtlc](../rtlc). `idealis-core` retains the finite-model, conformance, history, host, shrink, JSONL and Wasm adapters. `Value` and `CoreError` are shared by re-export, with the existing wire format preserved.

See the [repository](https://github.com/mizchi/idealis) for TypeScript bindings, the shared value format, examples, and evaluations. The `idealis-core` binary accepts the same versioned JSON commands over stdin/stdout, one JSON object per line.

The release is experimental. A matched finite history covers the supplied actions and observations; it is not a refinement proof or a liveness guarantee.
