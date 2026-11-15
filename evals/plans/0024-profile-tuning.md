# 0024: プロファイルに基づくmicro tuning

0023後の実装をbeforeとして固定する。Wasm scalarの後退と複合値探索をCPU sampleで調べ、時間を使っている処理から順に小さい変更を試す。公開IR/API、error順序、完全比較、予算、反例の意味を保つ。

## 探索と計測条件

- nativeはsamply（1,000 Hz）でRust releaseを記録する。別targetにsymbolを残したreleaseを作る。wasmはNode v24のinspector CPU profiler（1,000 µs）を用い、frontendとengine準備後に開始する。名前付きWasmはproductionと非custom sectionが同一か検査する。profile付き時間を改善率に使わない。
- 入力は既存4モデル。profileではcounter M=24、function N=16、channels N=5、ring N=100,000を使い、各16回完全探索する。状態数と生成数を各回検査し、raw profile、input、source/binary hashとself/inclusive集計を保存する。nativeは1/8 workerを区別する。Wasmは逐次だけ。
- samplyの既存binaryはlibiconvへのNix store参照が欠損していた。今回の起動だけDYLD_LIBRARY_PATH=/usr/libを指定する。グローバルな環境やtool installationは変更しない。
- Criterionは0023の固定14本、sample size 20、warmup 0.5秒、measurement 1秒、noise threshold 3%を維持する。fresh before、候補ごと、finalを別名で保存する。profile/build/testと速度計測を並走させない。
- 最終の独立した時間比較では、counter M=48とring N=1,000,000のWasmをbefore/after交互に各1 warmup + 3測定（16 sample）で測る。0023で後退した条件を同じ入力で確認する。複合値とnativeの対照はCriterion14本を使い、large全経路の再測定とは称さない。

## 改善の進め方

最初のprofileを保存した後に、候補と対象費用を追記して実装前にcommitする。既存の意味が変わらない対照を通し、必要な挙動/resource回帰をRed→Greenにする。性能だけの変更はbeforeとの差を性能の基準とし、元から正しい意味を無理に失敗させない。候補が効果を示さない場合もprofileと測定を残し、戻す。

runは追記のみ。source変更、例外、未完了の探索、プロフィールの取得失敗を成功へ置き換えない。最終のprofileも取り、割合だけでなくabsolute sample、探索work、結果の一致を確認する。profile用buildがproductionと異なる場合はその差を記録する。

just check/browser、portable-build/test、runtime-build/test、fmt/clippy、safety/frontierの差分、TLC正常/故障、evaluate/replayを実行する。変更前binaryと各sourceはtagで保持する。

## 名前付与の補正

最初のWasm profileは名前付き別buildの非custom section不一致でerrorになった。関数code/type/function sectionの一致を検査し、production binaryの末尾へname custom sectionだけを追加する。これにより記録対象の実行sectionはproductionと完全一致する。元の失敗runを残し、修正後は別runへ保存する。

CARGO_PROFILE_RELEASE_STRIP=noneによる別buildは関数順も変わり、name付与の検査が拒否した。cargo rustc --release -- -C strip=noneでCargoの通常profile metadataを保つと、code/type/function sectionが一致した。計測はそのnameだけを元binaryへ追加したものを使う。

## profile後の候補（実装前）

Wasm counterのself sampleはExpression::eval 55.5%、visit_branch 29.7%。native 1のcounterでもevalが41.3%。Wasm channelsはArena::selectが17.2%、functionはArena::findが28.0%。native 8の全thread sampleは待機が多いため、その割合をCPU使用率と解釈しない。

1. eval-leaves: literal/read/bound/@を小さいdispatchへ切り出し、残るoperator本体をinlineしない関数へ分離する。式の計算順・error・workは変更せず、大きい評価器へ入るleaf呼び出しを減らす。Criterionと最終Wasm scalar比較で採否を決める。
2. indexed-lookups: 正の整数keyについて、key-1番目にそのkeyが存在することを検証してから直接参照する。selectとEXCEPTのpath探索で共通に使う。それ以外のdomain/keyは元の検索へ戻る。疎な整数domain、0/負数、record、missing key、衝突を対照にする。TLC v1.7.4 FcnRcdValue.select/takeExceptと照合済み。

既存の意味を変えないため、追加する意味論の対照はbeforeでも通す。新しい失敗を作るために期待値を変えない。性能の基準はfresh before。各候補を別Criterion runへ保存し、final profileでsample/CPU deltaと結果を再照合する。
