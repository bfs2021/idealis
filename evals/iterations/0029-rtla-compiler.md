# 0029: RustでTLA+ソースから検査まで実行する

[Rust frontend](../../docs/rust-frontend.md)を実装した。rtlaはparser、rtlcはIRとcheckerを保ち、rtla-compilerがASTの名前解決とIR生成を担当する。[計画](../plans/0029-rtla-compiler.md)を先にコミットし、Rustのcompile APIがないRedと、coreがsource requestを拒否するRedから進めた。

## 実装した経路

rtla-compilerの`compile(&Module, &Config)`は、定数、非再帰operator、lexicalなLET、有限量化、集合・関数・record・sequence、action列挙、temporal/fairnessを変換する。modelt-coreの`compile_tla`は解析を含める。native JSONLとWasmに`compile-tla` / `check-tla`を追加し、既存requestとABI v1を維持した。

TS SDKは`@mizchi/modelt/portable`の`compileTlaSource` / `checkTlaSource`。生成IRを保存して既存の並列APIやbackend契約へ渡せる。公開境界では型付きSafetyProgramを使い、Wasm応答のIRもfield・式・domain・temporalの構造を検査する。内部のIR nodeはJSONとして組み立てるため、この実装から速度向上を主張しない。

## 固定199入力の差分

[開発run](../runs/0029-compiler-development/result.json)と[最終run](../runs/0029-compiler-final/result.json)は、既存のparser評価と同じ199入力を使う。197件のIRがTS frontendと一致し、Rust native/Wasmのsource requestとTS生成IRの検査結果も一致した。2件は両frontendが拒否する既存の非BMP文字入力だった。

IRの比較では、optionalな空のproperty/fairness配列とENABLEDのnull subscriptだけを正規化する。式、branch名、branch順序、stepは一致を要求する。実行上限はdepth 6、states 1000、work 100000、temporal nodes 10000に固定した。完全探索を要求する性能実験ではない。

| 結果                | 件数 |
| ------------------- | ---: |
| complete            |   47 |
| invariant-violation |   58 |
| property-violation  |    4 |
| liveness-violation  |   40 |
| deadlock            |    9 |
| bounded             |    6 |
| empty               |    1 |
| 評価エラー          |   32 |
| compile時の拒否     |    2 |

予算切れ・空集合・評価エラーを成功へ変換していない。同じ既存入力の再検査であり、未知故障発見や独立した確認用seedの実験とは数えない。

## TLCとの直接比較

[TLC対照](../runs/0029-compiler-tlc-final/result.json)は、既存collections-temporalから実行前に選んだ20件。Rust frontendを比較器へ渡し、集合・EXCEPT・sequence・量化・lexical scope・初期/action property・WF/SF・deadlockの判定を照合した。20件すべてが一致し、8件の有限反例をTLCへ戻す検査も通過した。

比較器のfilename契約に合わせ、module headerだけをDifferentialへ変え、同じsourceを両者へ渡した。初回はcase directoryの作成漏れで停止した。[失敗run](../runs/0029-compiler-tlc/error.json)を保持し、修正後のrunは別IDにした。個々のsource/configとTLC logも保存する。

## 配布・ブラウザと制約

共通77テスト、Rust compiler 5件、coreのsource protocol、cargo fmt/clippy/test、実Chromium、既存evaluate/replayを検査した。単独consumerはtla-ts/Z3/fast-checkを入れずにsourceをWasmで検査し、Rust consumerもsourceからcompile/checkする。4つのcrateのarchiveを検査し、parser/checker単独consumerの依存分離も維持した。[検証ログ](../runs/0029-compiler-verification/result.json)を保存した。

配布consumerの初回は、fixtureを共通化した後も古いmodule名の期待値が残っていたため失敗した。sourceのmodule名に合わせて修正し、同じ反例長4の検査を再実行した。失敗ログも検証記録に残す。

既存mutationのscoreは95.4%、104 killed / 5 survived / 133 compile errors。対象はconformanceとdurable-counterで、Rust compilerのmutation coverageではない。全TLA+、SANY、SMTによるモデル検査、Apalache adapter、ITF importerは追加していない。
