# 0046: TLCの有限モデル対応を増やす

[計画](../plans/0046-tlc-finite-support.md)に沿って、最初の5分野を実装した。model value、探索制約、有限CHOOSE、整数のdivision/modulo/power、PROPERTY内の定数有限domain上の時間論理量化を対象とする。SYMMETRY、VIEW、checkpoint、標準moduleの残り、simulation/DFID、coverage、ALIASはcrateのチェックリストへ未対応として分けた。

## 意味論と比較

比較元はTLCの`5a47802`。保存ログのバナーはTLC2 Version 2.19で、JARのdigestもmanifestへ記録する。ローカルcheckoutのHEADだけでなく、同revisionのModelValue.java、Tool.java、Liveness.javaを確認した。typed model valueの正規化時の比較と等価比較は、このrevisionで扱いが異なる。

先に5分野の回帰を追加し、旧実装では未対応として5件とも失敗した。実装後はbudget・scopeの検査を加えた8テストと、逐次/並列の制約境界テストを通した。初期状態も含め、制約で除外する候補のinvariant/propertyを検査する。除外した候補を探索queueや活性グラフへ入れず、候補の生成自体はdeadlockに数えない。

[pilot](../runs/0046-finite-pilot/error.json)では5種類の差を保存した。最小整数を直接負のリテラルとして書いたfixtureは、TLCが正の2147483648を先に拒否したため、計算式へ直した。Rust frontendに既存の例外受理がある点は未対応として記録した。Specの定義順もTLCの名前解決に従って直した。混在したtyped model valueは集合の正規化では比較でき、等価比較時に型エラーとなることを確認した。

空domainの全称時間論理量化は、比較元TLCのtautology診断に合わせる。空の存在量化の反例をTLCへ返すと、検査対象の否定が恒真になり、TLCが探索前に同じ診断を返す。この場合は別の検査で保存したtraceの最終位置への到達と状態数を確認する。恒真の診断だけでtraceが実行可能と判定しない。

[third](../runs/0046-finite-third/error.json)では、untyped model valueのSUBSETへの所属で差を検出した。TLCはこの入力を型エラーとするため、式の所属を単なるsubset-eqへ変換する最適化をやめ、既存のmembership descriptorへ変換した。TS/Rustの両frontendを直し、既存の237入力でもIRと挙動を照合した。

[final](../runs/0046-finite-portable/result.json)と[再生](../runs/0046-portable-replay/result.json)は59入力と反例7件をTLC/native/Wasmで照合した。scope・budgetを含む回帰や別の有限モデルまでを含む結果であり、全TLA+互換性や実システムの未知故障発見の証拠ではない。

JSのmodel valueは文字列と区別できるimmutableなobjectへ戻し、toPortableValue/fromPortableValueの往復でもtagを保持する。通常のアプリケーションrecordにtype/valueというfieldがある場合はmodel valueへ読み替えない。共有fixtureと往復の失敗する回帰を追加して確認した。

## ベンチマーク

[変更前の14本](../runs/0046-criterion-before/result.json)を保存した。既存の4モデルの探索、値のintern/EXCEPT、型所属を測る。追加した6モデルはASTからのcompileと生成済みIRのcheckを別々に測る。いずれもCriterionの20sample、warmup 0.5秒、measurement 1秒を使い、parser、IPC、process起動を計測から除く。

最初の変更後測定では後退があったため、通常の整数/Boolean比較をmodel value判定から分けた。続けて、検査すべきpredicateのない既訪問候補ではproperty検査を呼ばず、action propertyがない場合は親stateの参照とnextの構築を省くようにした。predicateがある候補の検査順序は保つ。制約で除外する候補もmaxDepthを超えないことを、失敗する回帰を追加して修正した。

[最初の変更後](../runs/0046-criterion-after/result.json)、[比較の調整後](../runs/0046-criterion-tuned/result.json)、[候補検査の調整後](../runs/0046-criterion-lazy/result.json)を保存した。旧版も`d76cd20`のGit archiveから再ビルドして[対照を再測定](../runs/0046-criterion-control/result.json)した。同じ旧版でも初回と差があり、時間の変化を全て追加機能の費用とは解釈できない。archive自体もdigest付きで保存し、現在のsourceを旧版と取り違えない。

以下は平均時間のms表示。差は再測定した旧版に対する値で、負は短縮、正は遅延。confidence intervalと全sampleは各runのCriterion JSONを参照する。計測中はbuild/test/replayを並走させていない。単一端末上で順に測った結果であり、一律の高速化やJVMとの倍率を示さない。

| benchmark                        | 旧版の初回 | 旧版の再測定 | 最終版 | 再測定した旧版との差 |
| -------------------------------- | ---------: | -----------: | -----: | -------------------: |
| membership/function-cube         |      2.066 |        2.425 |  2.360 |                -2.7% |
| membership/parallel-channels     |      0.902 |        1.095 |  0.969 |               -11.5% |
| search/counter-product/1         |      9.201 |       10.156 | 10.674 |                +5.1% |
| search/counter-product/8         |      9.225 |       10.227 | 10.471 |                +2.4% |
| search/function-cube/1           |      9.963 |       10.658 | 10.622 |                -0.3% |
| search/function-cube/8           |      8.978 |        9.439 |  8.790 |                -6.9% |
| search/linear-ring/1             |      1.284 |        1.184 |  1.175 |                -0.7% |
| search/linear-ring/8             |      1.055 |        1.181 |  1.173 |                -0.7% |
| search/parallel-channels/1       |     12.823 |       14.864 | 14.088 |                -5.2% |
| search/parallel-channels/8       |      8.583 |        8.846 |  8.661 |                -2.1% |
| values/except-function           |      0.470 |        0.536 |  0.522 |                -2.6% |
| values/except-record             |      3.941 |        4.542 |  4.291 |                -5.5% |
| values/intern-existing-functions |      0.447 |        0.522 |  0.503 |                -3.5% |
| values/intern-new-functions      |      0.498 |        0.564 |  0.533 |                -5.5% |

scalar counterには後退が残る。他の探索とmicroも含めて速さの全面的な改善とは判断しない。有限fragmentの追加対応を採用し、この測定を次のprofileによる改善の基準にする。

[追加機能の最終12本](../runs/0046-compiler-final/result.json)は次の平均時間だった。単位はµs。小さい固定モデルなので、大きい分散モデルの探索時間には一般化しない。

| 入力              | ASTからcompile | IRをcheck |
| ----------------- | -------------: | --------: |
| model-untyped     |         16.462 |     1.201 |
| state-constraint  |         11.697 |     0.531 |
| action-constraint |         12.237 |     0.499 |
| choose-order      |         19.229 |     0.558 |
| power-min         |         15.690 |     0.481 |
| temporal-fair     |         45.789 |    12.064 |

7つのCriterion runは`verify.ts`でmanifest・入力・ログ・全sample/推定値・旧版archiveのdigestを再検査した。これは保存データの再生確認で、新しい独立した性能測定ではない。

## 検証と残り

check、browser、portable-test、safety-test、GoのCGO無効/race/vetと独立consumer、配布packageの独立consumer、evaluate/replay-evaluationを確認した。mutationの既存対象は104 killed / 5 survived / 133 compile errorで95.4%。Rust検査器のmutation scoreとは区別する。

Jevの28比較と保存記録を同期した。Rust sourceだけでなく、同梱JSON/TLA/CFG fixtureも判定の鮮度へ結び付ける。staged fixtureの変更で保存した判定を拒否する回帰を追加した。モデルのsupports判定はこの宣言範囲の同期確認であり、テストや形式検証の代用ではない。

次はSYMMETRY/VIEWによる探索削減を別の計画で評価する。公平性と反例の復元を保持できる条件を定め、削減前の同じグラフと比較する。checkpointやdiskへの退避は、これらと別の探索・永続化契約として扱う。
