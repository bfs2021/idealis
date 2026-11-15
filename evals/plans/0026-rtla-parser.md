# 0026: TLA+構文解析をrtlaへ分離する

rtlcへIRを渡す前の構文解析を、modelt/rtlcに依存しないRust crate `rtla`として実装する。既存parserはtla-ts 0.0.10のTypeScriptであり、Rustの移動元はない。TLA+とTLC設定のASTを返すpureなparserを作り、IR生成・名前解決・モデル検査とは分ける。既存TS経路を維持し、parser単独のnative/Wasm利用を可能にする。

## 境界と対応範囲

有限fragmentの既存入力を対象にする。モジュール、宣言、operator/function定義、LET、量化、集合・関数・record・sequence、EXCEPT、CASE、action/temporal/fairness構文を扱う。コメント・文字列・括弧・bulleted junctionのoffside規則、優先順位の重なりと非結合operatorを検査する。SANY全体、proof、INSTANCE、nested module等を黙って捨てず、未対応として診断する。AST解析の成功を名前解決や意味論の検証と解釈しない。

契約はtyped AST、位置付きのsyntax/unsupported診断、parse_module/parse_expression/parse_config。JSON経由のnative検査driverとhost-importなしWasm adapterも同じparserを使う。rtlcへの依存とASTからIRへのloweringは含めない。

## 実装前に固定する対照

tla-ts 0.0.10のparser/lexerと、tlaplus/tlaplus v1.7.4 (5a47802b5c391f59ecdd44117981f4ff8c0656ba)のSANY grammar、ConfigConstants、ModelConfigを参照する。Channel/DieHard等の既存fixtureと、collections/temporal、TLC差分、既存benchmarkの保存source/configを固定した回帰corpusとして使う。重複source/configを除き、対応範囲外とsyntax失敗を成功件数へ混ぜない。

先に外部consumerのASTテストを書き、crate欠落をRedとして保存する。Green後はnative/Wasmが返すAST、tla-ts ASTの意味を持つfieldを比較する。location/range、proof用metadata等を比較対象外にする場合は一覧で記録し、意味を持つfieldを落とさない。故障入力には文字列/コメント未終端、欠落delimiter、offside、曖昧な優先順位、未対応構文を用意する。正常AST、明示的な拒否、parser crashを区別する。

## 検証と配布

crate単独の全target、fmt/clippy、wasm32 build、CLI、実ブラウザと差分corpusを確認する。just check/browser、evaluate/replay、packageの単独consumerまで実行する。新crateとWasm artifactを配布手順へ追加する。runは上書きせずevalsへ入力・結果・digest・採否を保存する。性能改善や全TLA+互換の主張はしない。
