# 0056: 探索本体の関数値を圧縮する

基準は3c11b826。0054/0055のoracle費用ではなくrtlcの関数・sequence値を対象にする。公開IR/Value、完全探索の範囲、状態数・生成数、work、反例と予算の扱いを維持する。TLCのローカルclone d8fb231のFcnRcdValue.select/takeExceptを読み、EXCEPTのdomain外・入れ子・元の関数の不変性を確認した。

## 仮説と実装候補

現行Cell::FunctionはVec<(Scalar,Scalar)>を持ち、EXCEPTはkeyも含めて全cloneする。immutable domainを共有し、値を別に持つ。64要素以下の全Boolean関数はu64へ詰め、更新とhashを定数時間にする。sequenceを含むその他の値は汎用Scalar列へfallbackする。小整数のpackingは必要性が確認できた場合に別stageで検討する。

domainと値のhash衝突では全体を比較する。generic関数の既存衝突fixtureを維持する。record/tuple/mapの外部表現と正規化、typed model valueの比較エラーを維持する。nativeのoverlay/mergeでdomain・payload両方のRefをremapする。新しいcrate依存と公開APIは追加しない。

## 比較条件

変更前のnative/Wasm、Criterion executable、sourceとhashを.artifactsへ保存する。profileは通常測定から分け、既存のcounter M24/function N16/channels N5/ring N100000をprepared IRで16回完全探索する。Node inspectorまたはsamplyを使う。実装候補の採否はprofilerを外した測定で決める。

大規模比較は既存single-modelのfunction N20（1048576 states / 20971521 generated）とchannels N7（2097152 / 22020097）を固定。比較routeはtla-ts 1、TLC 1/8、before/after native 1/8、before/after Wasm 1。各1 warmupと3測定を交互・逆順に配置する。同じsource/config、heap設定、同じNode・arm64 Java・TLC jar、同じホスト。source-to-resultのwall時間を主値とし、API区間・sampled RSSを補助として保存する。全実装のcomplete・状態数・生成数を検算し、unknown/不完全/エラーを速度比へ入れない。TLC区間とRust API区間は異なるのでkernel比にはしない。大規模再測定中にbuild/test/profileを並行しない。

Criterionは既存14本を同じbench sourceで20 samples・warmup 0.5秒・measurement 1秒、before/after/after/before。counter/ring、unknown等の対照を残し、10%以上の後退を明記する。新しい幅1/16/32/64/65と更新・membershipのbenchを追加する場合、同じharnessを先に基準へbuildしてから比較する。

主目標はfunctionでtla-ts/TLCへの速度差を縮め、実際に上回った条件を報告すること。channelsも同じ条件で測り、追い越せなかった場合も結果を残す。固定倍率を仮定しない。既存の小さい正常/違反モデルを別途全routeへ通す。

## TDDと互換性

共有domain、64/65境界、Boolean→一般値→Booleanのcanonical identity、domain外/入れ子EXCEPT、collision、overlayのdomainとpayload Ref競合をテストする。内部表現のテストだけでなく既存213件等の有限モデル、native/Wasmの全応答・予算・反例を基準へ照合し、TLC差分と反例再生を行う。

終了前にfmt/clippy、rtlc/portable/safety/criterion/check/browser、runtime、evaluate/replay、cross/async、package consumer、SUPPORT/Jevを確認する。測定・失敗runは上書きしない。採用判断と残るTLCとの差をiterationへ記す。ローカルcommitまでとし、PR/push/mergeは含めない。
