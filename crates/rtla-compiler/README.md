# rtla-compiler

rtlaの型付きModule/Configを、rtlcのSafetyProgramへ変換するRust frontend。parserとcheckerを別crateに保ち、両者の公開契約をつなぐ。Nodeやsolverには依存しない。

`compile(&module, &config)`は名前解決、定数とoperatorの展開、action列挙、temporal/fairnessを変換する。対象はidealisの既存TS frontendと同じ有限fragment。INSTANCE、ユーザーモジュール、高階・再帰operator、無限集合の列挙、symmetry、state/action constraintは扱わない。解析はrtla、実行時の型・IR検査と探索はrtlcが担当する。

内部ではJSONのIR nodeを組み立て、公開境界で型付きSafetyProgramへ変換する。frontendの性能改善を主張する実装ではない。既存TS frontendは独立した差分比較対象として残す。

```sh
just compiler-test
```
