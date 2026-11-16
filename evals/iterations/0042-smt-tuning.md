# 0042: SMTのCriterionと解析費用の改善

スクリプトの各checkが同じ制約を再解析していた処理を、不変な解析結果の共有へ変えた。nativeの解析時間は再確認で、反復checkが約4.0倍、scope操作付きが約1.7倍、整数iteが約6.7倍になった。既存6ケースを残し、Criterionを14ケースへ増やした。

## 条件と測定順

[plan](../plans/0042-smt-tuning.md)を先に固定した。改善前のライブラリはd5fb2ffを使い、両版で同じbenchと4つのSMT-LIB fixtureをビルドした。Apple M5、macOS arm64、Criterion 0.8.2、20 samples、warmup 1秒、measurement 2秒。strip=noneの保存bench binaryを全測定で再利用した。ソース、依存lock、binary、CPUとOS、負荷、raw samplesを各runへ保存した。

14ケースは整数チェーンのparse+solve/prepared、整数iteのparse+solve、3種類のscriptのparse/prepared solve、Solver作成と32組のpush/assert/popを含む。生成や正しさ確認はtimerの外。parseの値は解析結果の構築・破棄を含み、JSONL/Wasmの輸送、JVMやZ3との比較ではない。

baseline→candidate、candidate→baselineの順に測った。[最初の比較](../runs/0042-smt-comparison/result.json)は中央値の条件を満たしたが、逆順のbaseline後半でばらつきが大きかった。同じbinaryのscript-prepared-repeat-64が0.944 msから2.608 msへ変わり、script-parse-scopes-32も0.416 msから0.837 msになった。

[追記plan](../plans/0042a-smt-measurement-recheck.md)に理由を残し、同じ条件でbaseline→candidateをもう1組測った。元の測定は保持し、[再確認の比較](../runs/0042-smt-comparison-recheck/result.json)も採用条件を満たした。以下の時間はこの再確認の中央値。倍率は初回と再確認を併記する。解析済み求解の高速化は主張しない。

| ケース                      | 改善前 µs | 改善後 µs | 初回倍率 | 再確認倍率 |
| --------------------------- | --------: | --------: | -------: | ---------: |
| `parse-solve-4`             |      6.50 |      6.60 |     1.02 |       0.98 |
| `prepared-4`                |      0.66 |      0.66 |     0.99 |       1.00 |
| `parse-solve-32`            |     69.53 |     68.24 |     1.01 |       1.02 |
| `prepared-32`               |     15.16 |     14.42 |     1.03 |       1.05 |
| `parse-solve-128`           |    424.38 |    417.52 |     1.01 |       1.02 |
| `prepared-128`              |    200.31 |    188.43 |     0.94 |       1.06 |
| `parse-solve-ite`           |      3.03 |      3.07 |     1.03 |       0.99 |
| `script-parse-repeat-64`    |   4701.18 |   1182.84 |     4.19 |       3.97 |
| `script-prepared-repeat-64` |    965.07 |    947.22 |     0.97 |       1.02 |
| `script-parse-scopes-32`    |    415.17 |    239.00 |     1.77 |       1.74 |
| `script-prepared-scopes-32` |     20.08 |     20.00 |     0.98 |       1.00 |
| `script-parse-ite-64`       |    212.74 |     31.78 |     6.68 |       6.70 |
| `script-prepared-ite-64`    |     23.74 |     24.00 |     1.03 |       0.99 |
| `context-scopes-32`         |   3758.30 |   3712.21 |     0.98 |       1.01 |

反復checkのparseは20%以上、scope scriptのparseは10%以上という目標を満たした。3つのparseケースは再確認の95%区間も重ならなかった。既存ケースと解析済み求解の中央値の後退は、全組で基準の10%以内だった。専用ホストではないため、この倍率を全入力へ外挿しない。

## プロファイルと変更

samply 1000Hzで反復checkのparseを各5秒計測した。改善前の4,991サンプル中4,656でProblem::parseが呼び出しスタックに現れた。制約を変えないcheckでもSolver::snapshotが再解析していることを、ソースとこのprofileから改善対象にした。

SolverとScriptのqueryがArc<Problem>を保持する。snapshotは不変な解析結果への参照を共有する。assert/popは引き続き再解析し、成功時だけ新しいProblemに置き換える。各checkのtodo、整数制約、Bool代入とstatsは新しく作る。探索・学習状態や回答のcacheは追加していない。

改善後は、残るSolver::assertが4,900サンプル中4,518のスタックに現れた。追加assertによる全体の再解析は次の候補として残す。inclusiveなサンプル数は重複し、固定時間のprofileであるため、サンプル数の比を速度倍率として使わない。[profile run](../runs/0042-smt-profile/result.json)にraw profile・symbol sidecar・集計を保存した。

## 正しさと再生

開発seed 0x42a1で35 scriptの300 check、確認seed 0x42c3で67 scriptの588 checkを検査した。別に4つのCriterion fixtureの193 queryを、Z3の逐次実行とSATモデルの固定で照合した。scope fixtureは[SMT-LIB 2.7 Figure 3.11](https://smt-lib.org/papers/smt-lib-reference-v2.7-r2025-07-07.pdf)の差分制約サイクルを基に、未対応の観測コマンドを除いた構造を繰り返す。

確認runの145リクエストは、旧版と新版のnative/Wasmで値・判定・work/stats・error応答が一致した。22種類の不正・未対応入力、work 0、空結果と既存handleの独立性も維持した。workにparserの費用を含めない既存契約は変えていない。

- [正しさの開発run](../runs/0042-smt-correctness-development/result.json)と[確認run](../runs/0042-smt-correctness-confirmation/result.json)は、同じcandidate source/binaryを使った。入力、全応答、旧版の応答、snapshot、Z3出力と返却モデルの検査を保存し、両runを再生した。
- 正しさ確認のsource fingerprintは`fc65b016406925eb630fc78459ae7db38e22a72a8c6018cd120853afda12b646`。[数値比較の再生](../runs/0042-smt-comparison-replay/result.json)も保存した。
- [最終検証](../runs/0042-smt-verification/result.json)にはfmt/clippy/Rust tests、node/Vitest、Chromium、check/browser、runtime-build/test、evaluate/replay、packageの独立consumerと、評価器の型検査失敗を記録した。

## 再実行と採用範囲

```sh
just rsmt-criterion
just smt-tuning-build
just smt-tuning-capture <saved-bench-binary> <new-provenance.json>
just smt-tuning-criterion <stage> <provenance.json> <new-run>
just smt-tuning-eval development <new-run> <old-native-binary>
just smt-tuning-eval confirmation <new-confirmation> <old-native-binary> <development-run>
just smt-tuning-replay <saved-run>
```

baselineを作り直す場合はd5fb2ffのライブラリと、今回と同じbenches/fixturesを組み合わせる。各runのbenchmarkSourcesのhashで入力とbenchを照合する。bench binaryの保存先は書き換えず、出力には新しいディレクトリを使う。

不変な解析結果の共有を採用する。一般LIA、CDCL、理論伝播、学習状態の再利用、assert/popの増分解析は追加していない。TLA+ checker、公開SMT adapter、WasmやZ3との新しい速度倍率は今回の対象に含めない。
