# 0023: Criterionで値保持・評価・並列探索を改善する

対象は採用した6項目。順にinternの二重保持、EXCEPTのコピー/再ソート、型所属のprepare/cache、候補buffer、重複検査、batch/worker選択を改善する。公開IRと結果の意味を保ち、実装・結果・採否を段階ごとに記録する。速さを理由にhash衝突の完全比較、短絡・エラー順序、EXCEPTの@、最短safety反例、global budgetを弱めない。

## ベンチと固定入力

Criterion 0.8.2をnativeのdev dependencyとして追加する。ベンチは実際のsrcをincludeし、公開APIへ計測用exportを追加しない。iter_batchedのsetupとdropを区別し、std::hint::black_boxで入力と結果を消去させない。native APIの計測はIR deserializeとTS frontend/IPC/起動を除く。Wasmと起動込みは別のend-to-end測定で扱う。

microは4,096個の幅12の関数intern、同じ関数のlookup、幅20のEXCEPT、4要素のnested record EXCEPT、4,096値の関数/record型所属。探索はcounter M=12 (20,736状態/82,945生成)、function N=12 (4,096/49,153)、channels N=4 (4,096/24,577)、ring N=10,000 (10,000/10,001)を1/8 workerで完全探索する。入力のsource/config/IRと解析上の計数を固定し、結果を見て入力サイズを変えない。

Criterionはsample size 20、warmup 0.5 s、measurement 1 s、noise threshold 3%。これはsampling期間でtimeoutではなく、実行回数に応じて延長される。各段階に一意なsave-baseline名を使い、target/criterionのsample・estimatesを新規runへコピーしてdigestを保存する。before / intern / except / membership / buffer / dedup / adaptiveを比較する。測定中はbuild/test/replayを並走させない。CIはsmokeのみとし、共有runnerの時間で採否を判定しない。

## TDDと意味論

各項目はRedを先に保存する。既存のHashMap collision、異なるworker local Ref、tuple/function/record等価性、逐次EXCEPT/@/missing path、動的boundに依存する型、未到達の定数エラー、空Init、既訪問target、最短trace、work/state/depth境界、ordered model errorsを対照にする。TLC v1.7.4のFcnRcdValue/RecordValue/Workerを読み、曖昧な点はsmallのTLC差分へ追加する。

cacheはcontextから独立した型だけを到達時にprepareし、成功した評価だけ再利用する。invariantや遷移の短絡/エラーを先へ移動しない。並列の重複検査はcanonical Refへ統合後に一意な候補へ検査を割り当て、commit順と予算切れを保つ。仕様全体を推測して検査を省略しない。

## 採否と最終比較

項目ごとに対象microと探索を測り、改善しない変更も記録して採用しない。work単位は実際の処理削減に応じて変わるので、状態数・生成数・違反traceを独立に照合する。最終比較では保存した変更前のnative/Wasm binaryと変更後を、0022と同じlarge 4入力で交互に実行する。native 1/8、Wasm 1、各1 warmup + 3測定。計96 sampleを保存し、source解析・起動込み/API区間・RSSを分ける。TLC/tla-tsの過去値を同時再測定と称さない。

just check/browser、portable-build/test、runtime-build/test、safety-test、frontier-safety-test、TLC正常/故障対照、cargo fmt/clippy、evaluate/replay-evaluationを実行する。全runは上書きせずledgerへ追記する。replayは保存した入力・binary・sourceと結果を検証し、smallの正常/故障を再実行する。large性能の独立した再測定とは区別する。
