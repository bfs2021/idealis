# 0041: SMT-LIB scriptの問い合わせ列

SMT-LIB入力のpush/pop、複数check-sat、Bool literalのcheck-sat-assumingを追加した。RustのScriptとJSONのrun-scriptがcheck順に結果を返す。既存のcontext APIで行える操作を、SMT-LIB入力から使うための拡張。[対応チェックリスト](../../docs/smt-checklist.md)に制約と残りを記録した。

## 条件と結果

[plan](../plans/0041-smt-scripts.md)を実装前にコミットした。回帰seed 0x4101で実装と評価器を確認し、予約した開発・確認seedは同じsourceとbinaryで実行した。旧版8650134のnative/Wasmをbaselineとして保存した。

[SMT-LIB 2.7の参照文書](https://smt-lib.org/papers/smt-lib-reference-v2.7-r2025-07-07.pdf)でstackとassumptionsを確認した。Z3 5.4.1のeval_smtlib2_stringへscriptをそのまま渡して逐次判定を得た。別にTSがassertion stackとassumptionsから作るsnapshotをZ3で解き、各SATモデルをsnapshotへ代入して再検査した。

| 評価        | script | check | SAT | UNSAT | 旧版の単発solveが受理したsource |
| ----------- | -----: | ----: | --: | ----: | ------------------------------: |
| 開発 0x41a1 |     35 |   300 | 200 |   100 |                               1 |
| 確認 0x41c3 |     67 |   588 | 392 |   196 |                               1 |

全checkで期待判定、Z3の逐次判定、独立snapshotの判定が一致した。native/Wasmの判定・値・workも一致した。手書き3 scriptと生成32/64 scriptを含み、既存の単発checkを正常対照として残した。

旧版には同じsourceをJSON solveへ渡した。単発checkの対照以外は未対応で拒否した。人工入力の対応率であり、実用入力全体の対応率や速度向上の主張ではない。

## 予算・エラー・状態の境界

各check時点の式をparse中に保持し、solve時に順番に求解する。maxWorkはスクリプト全体で共有する。最初のcheckが予算を使い切る場合、結果がSAT、unknown、unknownになることを検査した。unknownはwork理由と空モデルを持つ。再求解は予算と探索を初めから始める。

全コマンドを事前に検査し、後続の不正なpopなどがあれば全体をerrorにする。部分的なSATは返さない。22種類の不正・未対応入力をwork 0と通常予算の両方で拒否し、同じEngineの既存handleが影響を受けないことを検査した。checkのない入力は空の結果列で、検査成功には数えない。

push/popの0、入れ子scope、pop後の定数代入の復元、一時assumptionsの非持ち越しを含む。Rustテストで引用識別子、u32 numeralの境界、256 scope、256 check、件数超過も検査した。Node/Vitest、実Chromiumでスクリプトと共有予算を実行した。

## 保存と再生

- [開発run](../runs/0041-smt-script-development/result.json)と[確認run](../runs/0041-smt-script-confirmation/result.json)にsource、手順、snapshot、全応答、Z3の出力とモデルを保存した。入力と応答はJSONLで記録する。
- source fingerprintは`29a33bb74d3d23549f30e57cde0539e46ea4005ba73e85987dbbc6f9396e46d6`。実装、評価器、plan、lock、task、CI、consumerのスクリプトを含み、READMEと結果文書は除外する。
- 両runを再生し、結果列・モデル・workと保存した手順が一致した。Z3のモデル選択は固定せず、SATモデルを再検査する。
- [検証記録](../runs/0041-smt-script-verification/result.json)にTDDのRed、型検査での失敗と修正後のログ、fmt/clippy/Rust tests、node/Vitest、Chromium、check/browser、runtime-build/test、evaluate/replay、packageの単独consumerを保存した。

```sh
just smt-script-test
just smt-browser
just smt-script-eval development <new-run> <old-native-binary>
just smt-script-eval confirmation <new-confirmation> <old-native-binary> <development-run>
just smt-script-replay <saved-run>
```

baselineのWasmはold-native-binaryと同じディレクトリのidealis_smt.wasmを使う。古いrunを上書きせず、結果を見たseedを次の独立した確認へ再利用しない。

## 採用する範囲

宣言・logic・optionを冒頭に固定するbatch APIとして採用する。公開する型はScriptだけを追加し、結果には既存のSolutionを使う。各assertは追加時点で対応する差分制約へ正規化できる必要がある。各checkの探索・学習状態は再利用しない。

checkは最大256回。scopeとassumptionsは各256件。source/token/深さは既存の上限を保つ。SMT-LIB 2.7の任意Bool assumptions、scope内の宣言、define-fun、get-model/get-value、reset/exit、CLIのSMT-LIBストリーム入出力は今後の対象。一般LIAや他の理論の対応範囲は変えていない。
