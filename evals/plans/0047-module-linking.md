# 0047: ポータブルなTLA+モジュール解決

## 目的と順序

既存仕様を変更せず取り込める範囲を広げる。モジュール読み込み、INSTANCE、TLC設定の定数・operator置換を先に実装する。次の段階で量化fairnessと標準演算、coverage、SYMMETRYを扱う。

## 契約

モジュールの名前とソースをホストから渡す。compilerはファイルシステムやネットワークを読まない。既存の単一source APIとJSON要求は維持する。parserはINSTANCEと修飾名をASTへ保持し、名前解決と置換はcompilerへ分離する。

まず非再帰な外部EXTENDSと引数なしINSTANCEを扱う。WITHの変数置換は変数名に限定し、定数置換は対応する有限式を扱う。LOCALの非公開性、定義元のscope、同名の衝突、暗黙の置換を検査する。一般の高階引数、引数付きINSTANCE、nested module、再帰importは拒否する。未対応を全モジュール互換へ読み替えない。

## 判定と対照

- TLCのSpecProcessorの設定置換、SANYのgenerateInstance/processSubstを読む。曖昧な挙動は固定したTLC jarで確認する。
- 正常なEXTENDS、推移importとdiamond、LOCAL、named/unnamed INSTANCE、WITHのscope、複数instanceを固定fixtureへ追加する。
- 定数とoperatorの置換、引数数の不一致、不明な置換先、重複・循環置換、欠落module、循環import、名前衝突を検査する。
- native/Wasm/TLCに同じsource/config/module集合を渡し、判定と状態を比較する。違反は保存した反例の値・遷移を確認する。
- source集合だけを変えた対照でモジュールの混同やcache漏れがないことを確認する。未対応入力の拒否はTLCの拒否と区別する。

## 費用と回帰

既存の単一module Criterionを対照に、parseを除いた複数moduleのcompileを測る。探索の高速化とは解釈しない。計測中は他のbuild/testを並走させない。runは新しいパスへ保存し、過去runを編集しない。

crateのcargo fmt/clippy/test、portableのnative/Wasm契約、Chromium、just checkとjust browser、配布consumerを検査する。変更したcrateのSUPPORT.mdとJevの比較を同期する。
