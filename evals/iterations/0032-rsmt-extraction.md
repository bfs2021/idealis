# 0032: rsmtを単独crateにする

SMT試作を[`crates/rsmt`](../../crates/rsmt)へ移した。parser、求解、JSON protocol、Wasm ABIの実装を保ち、crateとnative binaryを`rsmt`、Wasmを`rsmt.wasm`にした。`Problem::parse` / `solve`と`Engine`をmodeltの他crateへ依存せず使える。

## archiveからの利用

[計画](../plans/0032-rsmt-extraction.md)に従い、先にpackage consumerへrsmt archiveを要求する検査を追加した。archiveがないため失敗するRedを保存した。切り出し後はMIT license、README、example、Criterionとテストを含むcrate archiveを作り、そのarchiveだけを別ディレクトリへ展開してconsumerを実行した。

consumerは具体値7のSAT、負閉路のUNSAT、work 0のunknownとモデル不在、Realの拒否を検査する。cargo metadataでmodelt-core、rtla、rtla-compiler、rtlc、Z3へ依存しないことも確認した。既存のnpm、Rust、TLA parser/checkerの単独consumerも通過した。

## 移動前後の応答

移動前に固定10入力、既知の開発64入力と確認160入力、拒否対照12入力、予算・handle操作5リクエストを保存した。合計251リクエストについて、移動後native/Wasmの応答が元のnativeと完全一致した。stats、具体値、errorも比較した。215件のSATモデルを元の式へ固定し、Z3で再検査した。

parser、solver、protocol、Wasmのソースはbyte単位で同じ。Wasm binaryのSHA-256も切り出し前と一致した。既知入力の回帰検査であり、新しい独立した確認用seedの実験ではない。0030/0031のrunと速度比較は編集していない。移動による性能改善は主張しない。

## 検証と再実行

Rustのfmt/clippy、5テストと6 Criterion smoke、単独example、NodeのZ3 contract 2件、実Chromiumを検査した。packageは5つのcrate archiveとWasmを作り、rsmtの依存の独立性を検査する。check/browser、runtime-build/test、既存evaluate/replayも確認した。[失敗ログ、回帰応答、検証結果](../runs/0032-rsmt-extraction/result.json)を保存した。

```sh
just rsmt-build
just rsmt-test
just rsmt-example
just rsmt-compare .artifacts/rsmt-replay evals/runs/0032-rsmt-extraction/baseline
just smt-browser
just package
```

既存の`smt-build` / `smt-test` / `smt-criterion`は残し、rsmtを呼ぶ。modeltのpublic backendやZ3 adapterへは接続していない。registryへのpublishとmergeは行っていない。
