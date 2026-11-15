# 0034: 静的Alloyサブセットの検査

`ralloy`を独立したRust/native/Wasmの試作として追加した。静的なsigと二項field、fact、引数なしpred、assert、run/check、関係演算、atom量化を有限scope内で列挙する。SAT符号化やAlloy全体の移植ではない。[計画](../plans/0034-alloy-subset.md)と[対応範囲](../../crates/ralloy/README.md)を参照。

## 公式Analyzerとの結果

公式Alloy 6.2.0/SAT4J、Rust native/Wasmを同じ入力で比較した。生成グラフの期待値は独立したTSの集合演算とグラフ走査で計算する。各入力に別の`Feasible` commandを付け、factの充足可能性も検査した。

| run                                                      | 固定ケース | 生成ケース | instance | counterexample | no-counterexample | no-instance | 返却構造の公式受理 |
| -------------------------------------------------------- | ---------: | ---------: | -------: | -------------: | ----------------: | ----------: | -----------------: |
| [開発](../runs/0034-alloy-development-fixed/result.json) |         24 |         32 |       16 |              7 |                 8 |          25 |              23/23 |
| [確認](../runs/0034-alloy-confirmation/result.json)      |         24 |         64 |       21 |             11 |                13 |          43 |              32/32 |

全入力の判定が一致し、返却したsig/fieldを固定した追加commandも公式Analyzerが受理した。各runで21件の拒否対照とworkゼロのunknownを検査した。確認は別seed0x34c3で、開発seedは0x34a1。固定24件は両runで同じなので、144件を独立した入力数として扱わない。

両runのsourceVersionは`b6e0768c6355388557566d414d3a70aa0aad3654ac60b2d3cd696c5d71d9359f`。入力、応答、公式receipt、標準出力とエラー、native/Wasm/JARのhashを保存した。replayは開発77応答・確認109応答を再現し、証拠のdigestも照合した。replayで公式Analyzerは再実行していない。

## 比較で修正した誤り

[最初の開発run](../runs/0034-alloy-development/error.json)は24件目で失敗した。ralloyが`none`を任意の次数の空関係として扱い、二項fieldと`none + none`の等価を受理した。公式Analyzerはarity mismatchとして拒否した。

公式`ExprConstant`・`ExprBinary`・`ExprUnary`と個別入力で確認し、`none`を単項に固定した。二項の空関係は`none -> none`と書く。型エラーにする回帰テストを先に失敗させ、修正後に通した。最初のrunはerrorのまま残し、修正後は新しいrun IDを使った。

Intの既定4-bit、0 Int、one/lone sigの全体scopeゼロ、fieldの既定one、空domainの量化、ordered tupleとして数えるone/disj、join/closure/restrictionも固定ケースで照合した。Alloyの型検査全体や全構文との互換性を示す評価ではない。

## 実装への反例と配布

2-cycleの反例を実装へ渡すと、自己loopだけを拒否する故障版は受け入れ、循環を拒否する正常版は拒否した。既知の人工故障1種類の再現であり、未知バグ発見には数えない。

実Chromiumでホストimportとcross-origin isolationなしのWasmを実行した。crate archiveだけを展開する別consumerも通過し、依存graphに他のidealis crateやZ3がないことを確認した。生成したWasmは261,526 bytes。これはこの環境のrelease buildのサイズで、速度比較ではない。

fmt/clippyとRust 7 tests、native/Wasm契約、ブラウザ、check、runtime-build/test、evaluate/replay、packageのログは[検証run](../runs/0034-alloy-verification/result.json)へ保存する。CIにはcrate、契約、実Chromium、単独consumerの検査を追加した。

## 採用範囲

小さいscopeで制約モデルのinstanceや反例を生成する独立ライブラリとして採用する。既存backendとCLIへの統合は保留する。構造の列挙なので大きいscopeには組合せ爆発があり、work予算切れをunknownで返す。一般のAlloy Analyzerとの速度比較やSAT符号化への切り替えは、代表的な制約モデルを決めて別の計画で評価する。
