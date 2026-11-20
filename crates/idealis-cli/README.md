# idealis-cli

モデルの許容観測と実装の観測を比較する Rust CLI。標準入力から1行ずつ JSON を読み、同じ `idealis_connect::Engine` に渡して応答を標準出力へ返す。各言語のホストは実装を駆動し、観測を CLI に渡す。

```sh
echo '{"version":1,"request":{"kind":"compare","oracle":{"kind":"allowed","values":[1,2]},"targets":[{"id":"counter","outcome":{"kind":"value","value":3}}]}}' | cargo run --quiet --manifest-path crates/idealis-cli/Cargo.toml
```

この例では counter の観測値 3 が許容集合から外れるため、応答の `result.status` は `mismatch` になる。生成・縮小・再生ではプロセスを開いたままにし、各 `Evaluate` に対応する ID と判定を `resume` で返す。

[ライブラリとプロトコル](../idealis-connect/README.md) · [対応範囲](SUPPORT.md)
