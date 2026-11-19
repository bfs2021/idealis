# Quint生成物のIR v1/v2回帰と測定

[計画](../plans/quint-sharing-regression.md)に従い、main d38fdf5f1を基点に実装した。実測sourceはd275e560f。raw入力・stdout/stderr・Criterion sample・TLCログは[保存索引](../runs/quint-sharing-regression/evidence.json)へ残す。生成TLA+と既存TLCオラクルは変更しない。

通常の回帰を38テストへ広げた。6モデルの正常/違反を省略v1・明示v1・明示v2のnative/Wasmで実行する。実際のIR version、compileと直接check、コンパイル済みIRの検査、v1の既定/明示の完全応答を照合する。TLCに計数がある場合だけ、その状態数・生成数にも照合する。workはv1/v2間の完全一致を要求しない。

ブラウザは3通りのversionで計36入力を検査する。最初のRedは、追加したbench用fixtureがない1件と、browser entryが実際のversionを返さない3件。fixtureを追加し、browserではcompileされたversionと直接source検査/IR検査の一致を確認する。既存のモデル検査の意味論的不具合を見つけたとは数えない。

Quint 0.33.1 / Apalache 0.63.0で6入力を新しいdirectoryへ再生成した。TLA+、stdout、Quint入力と設定は保存fixtureとバイト単位で一致した。新しいTLC比較は12設定×2 IRの24件がnative/Wasmと一致し、v1/v2合わせて反例12件をTLCへ再生した。同じsource/binaryでの再実行でも24結果と反例12件が一致した。初期違反でTLCが出さない計数はnullを保つ。

Criterionは6モデルのparseと、v1/v2のcompile・check・parse+compile+checkの計42区間。各caseの正常判定・状態数・生成数を検査してから測る。parseはmoduleとconfig、compileは解析済みAST、checkはcompile済みIRを入力にする。check内のIR検証・metadata作成と探索は分離していない。結果の破棄を含み、Quint生成・JVM起動・JSONL輸送は含まない。

測定環境はApple M5 / arm64、rustc 1.98.0、bench profileのopt-level 3。各区間20 sample、warm-up 500 ms、measurement 1 s。以下は1回の測定の中央値で、単位はµs。各セルはv1 / v2。95%区間とraw sampleは[構造化結果](../runs/quint-sharing-regression/benchmark-summary.json)とarchiveに残す。

| モデル       | compile v1 / v2 | check v1 / v2 |    全体 v1 / v2 | 全体 v2/v1 |
| ------------ | --------------: | ------------: | --------------: | ---------: |
| Counter      |   26.17 / 25.04 |   1.03 / 3.91 |   45.36 / 48.18 |       1.06 |
| SetFold      |   21.14 / 31.59 |   0.72 / 4.70 |   44.35 / 60.24 |       1.36 |
| SequenceFold |   24.82 / 36.11 |   0.79 / 5.89 |   48.87 / 66.08 |       1.35 |
| Choice       |   26.31 / 25.70 |   1.20 / 3.75 |   46.80 / 49.44 |       1.06 |
| Match        | 101.66 / 144.57 |  3.95 / 24.20 | 180.25 / 241.03 |       1.34 |
| NestedFold   |   35.11 / 71.47 |  1.24 / 12.09 |  68.79 / 115.19 |       1.67 |

この小さい6モデルではv2の全体が約1.06〜1.67倍、checkが約3.1〜9.7倍遅い。正常版のIR bytesも全件で増えた。例えばNestedFoldは1,146→1,723 bytes、workは19→40、checkは1.24→12.09 µs。共有IRが常に速いとは判断しない。固定費の内訳と大きいモデルでの分岐点は、この測定からは特定できない。

次の測定候補は、v2の検証/metadata作成と探索をprofileで分けること、call/frameを多く生成するQuintモデルの規模別測定、同じcompiled IRを再利用するprepared APIの費用である。callの課金や遅延引数の意味を変えた高速化は今回行わない。

再生は42区間の欠落、20 sample、正の推定値、sourceとrawファイルのdigestを検査する。実データをコピーし、完了ファイル欠落・推定値ファイル欠落・sampleの1 byte改変・同名runへの再実行の4対照を拒否した。元runの再生はその後も成功する。

check（Vitest/node各77件）、Quint 38件、compilerのfmt/clippy/all-targets、42 benchのsmoke、Chromium 5件、mutation 95.4%、配布consumer、評価とwitness再生、Jev live/offlineと自己テスト8件が成功した。CIには既存のQuintテストとcompilerのall-targets経路で接続し、時間の閾値を設けない。このブランチをGitHubへpushした後のCI実行は今回行っていない。

再実行は[利用手順](../../docs/quint-generated.md)を参照。固定corpusの互換性・性能回帰であり、Quint全体、Quint Connect、未知故障、全モデルにおける高速化の証明ではない。
