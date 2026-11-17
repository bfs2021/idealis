# 0047: 外部TLA+モジュールと設定置換

外部EXTENDSと引数なしINSTANCEを、明示したsource集合から解決する経路を追加した。Rustでは`rtla_compiler::compile_modules`、TSとJSONLでは既存source要求の`modules`を使う。ファイルシステムやネットワークをコアへ追加していない。

## 対応範囲と判断

非再帰EXTENDS、diamond import、named/unnamed INSTANCE、LOCAL、WITH、暗黙の変数置換を扱う。定義元のscopeを保持し、WITH右辺は呼び出し側で解決する。異なるinstanceの変数と定数を混同しない。反例のaction名は公開名・instanceの修飾名・LETの元のoperator名へ戻す。

設定の`N <- Bound`、同arityのoperator置換、module別の定義への値・operator・引数付きテーブル設定を扱う。operator宣言そのものへのテーブル設定は未対応。引数付きINSTANCE、LET内のINSTANCE、nested module、一般の高階・再帰operator、変数名以外への変数置換も対応外に残す。全SANY検査やlevel検査の代替ではない。

この範囲を採用する。量化fairnessと標準演算、coverage、SYMMETRYは次の段階に残す。今回は探索アルゴリズムを変更していない。

## TLC・native・Wasmの比較

[計画](../plans/0047-module-linking.md)に従い、[固定37ケース](../../crates/rtla-compiler/tests/modules-support.json)を同じ設定・source集合で実行した。比較元はTLC2 Version 2.19、revision `5a47802`。TLCの[SpecProcessor](https://github.com/tlaplus/tlaplus/blob/5a47802/tlatools/org.lamport.tlatools/src/tlc2/tool/impl/SpecProcessor.java)の設定置換、[Generator](https://github.com/tlaplus/tlaplus/blob/5a47802/tlatools/org.lamport.tlatools/src/tla2sany/semantic/Generator.java)のINSTANCE処理を参照した。

- [最終実行](../runs/0047-module-bounded/result.json)と[保存結果の再生](../runs/0047-module-bounded-replay/result.json)で、全37件の報告が一致した。
- native/Wasmの生成IR、検査結果とエラー応答を照合した。19件がcomplete、3件が不変条件違反、1件がモデル評価エラー、14件が未対応拒否になった。
- 受理した正常・違反入力はTLCと判定・状態数・生成数を比較した。3件の反例を元の仕様へ固定してTLCで再生した。
- 拒否した14件はTLCの受理・拒否と分けて記録した。TLCが存在しないmoduleへの値設定を無視するケースや、設定の不正を評価エラーに分類するケースも含む。同じエラーphaseや文言までは保証しない。
- Chromiumでも同じ37件を検査した。nativeの同じEngineに内容の異なる同名moduleを続けて渡し、要求間の混同がないことを確認した。

最終実行と再生のsourceVersionは`ddeb950873340296318fefe5f21347ff4c1e50c9f0c03b1c213d4dd8a0618398`。manifestにsourceのdigest、入力、Java/TLC/native/Wasmのbinary digestを保存した。人工的な小規模仕様の回帰であり、実システムの未知バグ発見や全TLA+互換を示すものではない。

## 最初に失敗したもの

[開発実行1](../runs/module-linking-dev1/error.json)では、テストのLET引数が外側の引数を再宣言しており、SANYが不正な仕様として拒否した。有効なscope検査へ修正し、再宣言の拒否も別ケースに残した。parserへの構文追加で既存の深さ制限テストがstack overflowになったため、名前付き式の解析を独立した処理へ分け、上限64を維持して回帰を通した。

[開発実行2](../runs/module-linking-dev2/error.json)では、operator宣言へ設定したテーブルをTLCがoperator値のままstateへ格納し、評価エラーにした。定義済みoperatorへのテーブル設定と、宣言への設定を分け、後者を未対応拒否にした。[開発実行3](../runs/module-linking-dev3/result.json)で35件が通った。

[最初の保存結果](../runs/0047-module-final/extends-counter-fault/report.json)にはaction名の内部識別子が残っていた。状態・反例の検査だけではこの問題を見逃した。公開名・修飾名・LET名のassertionを追加し、37件の最終実行で確認した。以前のrunは編集していない。

## コンパイル費用

[Criterionの17本](../runs/0047-module-criterion-bounded/result.json)を20 sample、warmup 0.5 s、measurement 1 sで測定した。計測中はこの作業のbuild・test・TLC実行を並走させていない。通常の開発用Macでの測定で、専用機の測定ではない。AST深さ制限の追加前の[初回計測](../runs/0047-module-criterion/result.json)も保存した。下表は制限追加後の値で、両runの差を高速化とは判断しない。

| ASTからIRへの処理      |      平均 |    平均の95%区間 |
| ---------------------- | --------: | ---------------: |
| 単一ファイルのcounter  | 17.317 µs | 17.097-17.593 µs |
| 同等のcounterをEXTENDS | 27.398 µs | 27.306-27.510 µs |
| diamond import         | 18.715 µs | 18.661-18.766 µs |
| 独立した2つのINSTANCE  | 75.746 µs | 75.191-76.468 µs |
| module別operator置換   | 23.674 µs | 23.574-23.767 µs |

counterのmodule解決込みcompileは、単一ファイルに対して約1.58倍の費用だった。構文解析、Wasm起動、IPC、process起動は測定区間に含まない。IRを再利用する探索の速度やTLCとの速度比には読み替えない。この測定をmodule解決の今後の最適化の基準とする。

既存のcompile/check 12本も残した。保存した72 artifactをdigestとestimatesへ照合し、以前の12本の保存runも同じ検証器で読めることを確認した。検査実行とbenchmarkの共通sourceのdigestに差はなかった。

## 回帰と配布

crateのfmt/clippy/test、`just check`、`just browser`、portableのnative/Wasm契約とChromium、runtimeのnative/Wasm/browserを通した。既存評価の実行・再生、mutationの回帰、npmと7 crateの単独consumerも通した。mutationは既存の109件の結果を再利用しており、新機能の検出力の評価ではない。

各crateのSUPPORT.mdを更新し、module解決と名前の書き換えをJevの比較元へ追加した。29比較で矛盾・未評価は0件。オフラインの鮮度検査と、古い・欠落・破損した記録を拒否する8テストも確認した。

ModuleをRustから直接構築する場合も、ASTの書き換え深さを64段で検査する。未使用の定義を80段へ構築した対照で、制限追加前の受理と追加後の拒否を確認した。
