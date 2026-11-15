# 0026: TLA+構文解析をrtla crateへ分離する

[`rtla`](../../crates/rtla/README.md)を独立したRust parserとして実装した。TLA+／TLC設定から型付きASTを返し、nativeとhost import不要のWasmで動く。rtlcやmodelt-coreへの依存はない。既存の`compileTlaSafety`はtla-ts parserを継続利用し、ASTからIRへの変換はTS側に残した。

## crateの境界と採用

[計画](../plans/0026-rtla-parser.md)を先に固定した。既存parserはtla-ts 0.0.10のTS実装であり、Rustからのファイル移動ではない。ASTの契約と解析手順を参照してRustへ移植し、NOTICEとMIT licenseをcrateへ同梱した。SANYの文法・優先順位・offside・EXCEPT・fairnessはtlaplus/tlaplus v1.7.4のソースと照合した。

構文解析の成功は、名前解決、型/level検査、標準モジュールの読込やモデルの正しさを示さない。proof、INSTANCE、nested module、RECURSIVE宣言、高階引数等は対応外。既知の未対応構文は`unsupported`で返す。式/設定の解析とASTの深さは64段、ASTの整数は正確な`2^53-1`までで、全TLA+／SANYの互換性を主張しない。

parserを独立crateとして採用する。TS frontendの置き換えやRustでのIR生成は今回の採用範囲に含めない。速度比較は行っていない。

## ASTと構文の対照

collections/temporal、既存TLC差分のdevelopment/confirmation、単一モデルの保存入力から、重複したsource/configの組を除いた199組を使った。モジュール199件と設定199回をtla-ts 0.0.10のASTと比較した。metadataを除いた構造が一致し、operator、binding順序、値、prime、bulletの階層、assumption/theoremを維持した。

moduleではname/extends/constants/variables/definitions/assumptions/theoremsを比較した。location/rangeと関連する範囲、spelling、Number.text、rawConstants、宣言・定義を再掲するunitsは比較しない。moduleのmodules/instances/usesは空であることを要求した。Decimal.textは残す。source位置やmodule内の宣言順のTS互換性を、この比較から主張しない。

native／WasmはASTと診断を含めて421応答が完全一致した。内訳は199組×2、構文対照18件、深さ上限5件。構文対照では次を区別した。

| 対照                                                                                 | rtla        | SANYの構文解析のみ |
| ------------------------------------------------------------------------------------ | ----------- | ------------------ |
| 優先順位、alias、nested junction/comment、CASE、空tuple/setの7件                     | ASTを返す   | 受理               |
| 欠落delimiter、未終端string/comment、優先順位の曖昧さ、空call/index、offside等の10件 | syntax      | 拒否               |
| operator引数のLAMBDA 1件                                                             | unsupported | 受理               |

SANYは`-s`で意味解析を無効にした。これは文法の受理/拒否の検査であり、level検査、型検査やモデル検査ではない。[最終run](../runs/0026-rtla-final/result.json)に全入力、AST、診断、SANYログとdigestを保存した。固定corpusの回帰検査であり、新しい独立した未知故障発見の評価には数えない。

## 発見した境界と修正

初期の128段のguardは、深い括弧でRustのtest threadが先にstack overflowした。解析上限を64へ変更し、同じ130段の入力で`unsupported`を返すことを確認した。平坦なoperator/prime/field chainもASTを深くするため、解析の再帰深さとは別にASTの深さを制約した。式と設定を含む5件でnative/Wasmが同じ診断を返す。ブラウザでは深さの拒否後も次のmoduleを解析できる。

初回の[development run](../runs/0026-rtla-development/error.json)はSANYの未終端commentのログ判定で失敗した。SANYは`Lexical {error:`と出力するため、通常の`Lexical error`だけを探す判定では拾えなかった。parserの受理結果の差ではない。この形式を明示的に分類し、module読込失敗や空出力をinfrastructureの失敗として残す対照を追加した。

[confirmation](../runs/0026-rtla-confirmation/result.json)はその修正後の418応答。[最終run](../runs/0026-rtla-final/result.json)はoperator chainの上限対照も含む421応答。失敗runと元の結果は編集していない。

## ブラウザと配布

ChromiumでChannel、Unicode量化、10万文字のidentifier、構文エラーと深さ上限後の再利用を確認した。host importとcross-origin isolationは不要。既存のportable browser taskでもparserを実行する。

`just package`は3つのcrateを一時workspaceでpackage検証する。展開したrtlaだけを参照する別consumerがofflineで解析でき、依存graphにrtlc/modelt-coreがないことを確認した。npm consumerはsolverを入れずに`@mizchi/modelt/rtla.wasm`を解決し、ASTを得た。registryへのpublishは実行していない。

crate欠落、literalとoperatorの区別、空application、深さとAST chainのRedを保存した。Rust全target、fmt/clippy、AST差分、ブラウザ、既存のcheck/evaluate/replay、package consumerの結果は[検証記録](../runs/0026-rtla-verification/result.json)を参照。
