# Rustで状態展開を再利用する

[計画](../plans/cached-expansions.md)を6e32f6237で固定し、task cacheのd329d47aaを基点に実装した。`rtlc::SafetyProgram::check_incremental`は、状態ごとの完全なNext展開だけを再利用する。TSの`checkSafetyIncremental`、native JSONL、Wasm、snapshotのmergeへ接続した。[APIと前提](../../docs/safety-expansions.md)を参照。

現在のInitから毎回BFSする。visited、propertyの合否、temporal graphは引き継がない。namespaceはbuild digest、IR version、変数順、全definitionとNext全体。候補はconstraintで除外する前に順序・重複・action indexを保って保存する。保持状態数を明示し、未完了の展開をcompleteとして保存しない。snapshotはportable値とSHA-256を持ち、別processのintern IDに依存しない。

## 証明と実装の対応

Lean 4.35.0 / Stdの証明へ9定理を追加し、合計26定理を監査・`leanchecker`で再検査した。追加したのはbuild/Next/状態/completeの再利用条件、保存関係が現在のNextと一致する条件、任意の現在Initからの到達関係の保存、現在のpropertyを再検査する場合の安全性。新しいpropertyが以前と同じであるという仮定は置かない。

実Rust/Wasmで16通りの条件を作り、Leanの実行可能な参照機へ照合した。全条件が揃う1件だけhitした。前段policyの37状態・23,606遷移の差分比較も維持する。fence、receipt、completeに加え、Next一致・完全展開・memo化の選択条件を壊した計6対照で証明が失敗した。

これらはpolicyと関係の証明で、Rustの全実装、LLVM、SHA-256やTLA+ evaluatorの機械証明ではない。正しい生成器が完全なNext列を保存したことと、意味に影響する依存をnamespaceへ含めたことは前提。SHA-256は生成者の署名ではない。

Rust APIの不在からRedを始めた。Rustの2回帰と、node:test/Vitestの共有9テストで次を確認した。

- 保存したQuint 6生成物のIR v1/v2でcold/warmの判定・状態数・生成数が一致する。正常版の計数は既存TLCオラクルと一致する。
- 12通りのproperty変更で反例を検出し、現在のuncached checkerの反例とwork以外が一致する。
- Init・state constraintを変更して到達状態を再計算する。除外境界のinvariantと、既訪問targetへのaction propertyも検査する。
- v2のNext自体を同じに保ち、参照definitionだけを変更すると全missになる。変数名、Nextとbuildの変更も失効する。
- state/work予算で未完了の展開を保存しない。depth-cutからの再利用、保持上限0/2、空Initと空successorを区別する。
- 現在のdeadlock、temporal/fairnessを検査する。7つのtemporal入力は現在のuncached結果と一致する。
- native processを再起動し、recordを含む状態を再利用する。独立したInit shardのsnapshotをmergeでき、別namespace・破損・同じ状態の異なる展開を拒否する。

Chromiumでも新しいWasm instanceへsnapshotを渡し、6件ともwarm missが0、property変更時は違反となった。状態展開を他workerへ分配するBFSや、探索queueのcheckpointは実装していない。共有storeへの公開とCASはホスト側の仕事として残す。

## 性能と採用範囲

Criterionは9入力×4区間の36測定。6入力は変更していないQuint出力。追加3入力はSetFoldから派生し、Nextで1..Nのfoldを毎回評価する。元の生成物を編集せず、派生条件とsourceをfixtureに保存した。

派生3入力はTLCでも検査した。Nextを変えず、期待する合計値だけをinvariantへ追加し、全件で2状態・3生成を確認した。Rust cacheの状態値も独立に`N*(N+1)/2`と照合した。最初のTLC実行は既定stackでStackOverflowとなったため、新しいrunではstackを64 MiB、heapを256 MiBと明示した。この参照実行は意味の照合用で、TLCとの速度比較には使わない。

Apple M5 / arm64、rustc 1.98.0、release opt-level 3 / LTO、各10 sample、warm-up 100 ms、measurement 300 ms。表は1 runの中央値、単位はµs。warmはnative cache objectのclone setupを除き、snapshotはJSONの復元・再検査・保存を含む。source compileとprocess起動は含まない。95%区間とsampleは[保存索引](../runs/cached-expansions/evidence.json)から参照する。

| 入力          | uncached | cold保存 |  warm | snapshot | uncached / warm |
| ------------- | -------: | -------: | ----: | -------: | --------------: |
| Counter       |     3.98 |    19.04 |  5.45 |    24.40 |            0.73 |
| SetFold       |     4.87 |     7.74 |  5.27 |    10.01 |            0.92 |
| SequenceFold  |     5.98 |     9.33 |  7.61 |    13.55 |            0.79 |
| Choice        |     3.81 |    14.57 |  4.80 |    20.04 |            0.79 |
| Match         |    25.08 |    82.49 | 32.02 |   106.41 |            0.78 |
| NestedFold    |    12.55 |    18.29 | 14.00 |    21.94 |            0.90 |
| 派生fold 16   |    16.40 |    25.60 | 11.58 |    25.23 |            1.42 |
| 派生fold 256  |    88.50 |   101.98 | 12.15 |    26.40 |            7.28 |
| 派生fold 4096 |  1225.33 |  1252.42 | 12.99 |    28.40 |           94.35 |

初版ではIR検証がcacheのbindingと通常探索で二重に走り、小さい6件のwarmは約2倍遅かった。[再測定前の計画](../plans/cached-expansions-tuning.md)をd276697e0へ記録し、既存のIR検証が通った直後にnamespaceを作るよう整理した。検証条件を維持し、9件のwarmは初版から約1.44〜1.79倍改善した。両runの全36区間とsourceを残した。

4096要素の派生foldでは約94倍、snapshot込みでは約43倍だった。一方、小さい生成物6件ではwarmもuncachedより約1.09〜1.37倍遅く、cold/snapshotの固定費はさらに大きい。この人工的な重いNextの結果を一般的なTLAモデルの高速化倍率へ広げない。Next評価の高い費用を繰り返し払う場合の任意APIとして採用し、通常の`check`/`check_parallel`はcacheを使わない。

## 実行の記録

初期のfixture field名とLeanの予約語を修正した。wireのaction indexをu32へ揃える途中でbuildが失敗し、直接実行したeval-01/replay-01が以前のbinaryを使った。その2 runは明示的に無効化し、採用する証拠から除いた。現在の証拠は成功したportable-buildに依存するeval-03/replay-03とproof-04。

途中でディスクが満杯になり、proof-02と最初のportable-testが停止した。生成済みのincremental cacheと過去のpackage検査用targetを整理し、新規runで再検査した。失敗ログを残し、未完了runを成功へ置き換えていない。これらを未知の実プロジェクト故障の追加検出とは数えない。

`check`、共有9件、前段cacheの共有11件、Chromium 7件、rtlcのfmt/clippy/all-targets、portableのRust/TS/Chromium検査、runtime、cross-language、配布consumerが成功した。既存mutationは95.4%で、109判定を再利用した。この値を新しいRust cacheのmutation coverageとして扱わない。評価ハーネス、既存のdevelopment/confirmation loopと10 witnessの再生も検査した。Jevの新しい2比較と全7 crateのlive/offline/staged検査を記録する。remote CIはこのローカルbranchでは未実行。

snapshotのbyte予算、namespace digestの事前計算、Nextの依存単位での部分失効、分散BFSの所有権・frontier・終了検出が次の候補。現在のNext全体一致という条件を緩める前に、それぞれの再利用条件を定義・検証する。
